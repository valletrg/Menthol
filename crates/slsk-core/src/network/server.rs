//! Server connection actor - manages TCP connection to Soulseek server.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use slsk_proto::codec::SlskWrite;
use slsk_proto::server::{self, ServerMessage};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::command::Command;
use crate::config::Config;
use crate::event::Event;
use crate::network::framing::{read_frame, write_frame, encode_message};
use crate::network::state::{ConnectionState, ServerStats};

// Constants for memory-bounded operation
const INITIAL_READ_BUF: usize = 16 * 1024; // 16 KiB
const MAX_FRAME_SIZE: usize = 2 * 1024 * 1024; // 2 MiB

/// Handle returned from ServerConnection::connect()
/// Allows sending commands and receiving events
pub struct ConnectionHandle {
    pub cmd_tx: mpsc::Sender<Command>,
    pub evt_rx: mpsc::Receiver<Event>,
}

/// Connect to the Soulseek server and return a handle
pub async fn connect(config: &Config) -> Result<ConnectionHandle, crate::error::Error> {
    let (cmd_tx, cmd_rx) = mpsc::channel(256);
    let (evt_tx, evt_rx) = mpsc::channel(256);
    let stats = Arc::new(ServerStats::new());

    // Clone the config values to avoid lifetime issues with tokio::spawn
    let host = config.host.clone();
    let username = config.username.clone();
    let password = config.password.clone();
    let port = config.port;

    tokio::spawn(async move {
        let owned_config = Config {
            host,
            username,
            password,
            port,
        };
        if let Err(e) = run_connection(&owned_config, cmd_rx, evt_tx, Arc::clone(&stats)).await {
            tracing::error!("connection error: {}", e);
        }
    });

    Ok(ConnectionHandle { cmd_tx, evt_rx })
}

async fn run_connection(
    config: &Config,
    mut cmd_rx: mpsc::Receiver<Command>,
    evt_tx: mpsc::Sender<Event>,
    stats: Arc<ServerStats>,
) -> Result<(), crate::error::Error> {
    // Connect to server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("connecting to {}...", addr);

    let mut stream = tokio::net::TcpStream::connect(&addr).await?;
    tracing::info!("connected to server");

    // Send login
    send_login(&mut stream, &config.username, &config.password, 160, 2).await?;

    // Read login response
    let login_resp = read_server_message::<server::login::LoginResponse>(
        &mut stream, server::login::CODE, &stats
    ).await?;

    match login_resp {
        server::login::LoginResponse::Success { greet, own_ip, hash, is_supporter } => {
            tracing::info!("login success! motd len={}, supporter={}", greet.len(), is_supporter);
            let _ = evt_tx.send(Event::Connected { motd: greet }).await;
        }
        server::login::LoginResponse::Failure { reason, detail } => {
            tracing::warn!("login failed: {} / {:?}", reason, detail);
            let _ = evt_tx.send(Event::LoginFailed { reason }).await;
            return Ok(());
        }
    }

    // Send SetWaitPort
    let frame = make_set_wait_port(2234);
    stream.write_all(&frame).await?;
    tracing::debug!("sent SetWaitPort(2234)");

    // Main loop - handle commands and incoming messages
    loop {
        tokio::select! {
            // Incoming message from server
            msg = read_one_message(&mut stream, &stats) => {
                match msg {
                    Ok(Some((code, payload))) => {
                        handle_server_message(code, payload, &evt_tx).await;
                    }
                    Ok(None) => {
                        // Would block - try again
                    }
                    Err(e) => {
                        tracing::error!("read error: {}", e);
                        let _ = evt_tx.send(Event::Disconnected { reason: Some(e.to_string()) }).await;
                        break;
                    }
                }
            }
            // Incoming command from GUI
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(Command::Disconnect) => {
                        tracing::info!("disconnect requested");
                        break;
                    }
                    Some(Command::Search { query, token }) => {
                        tracing::debug!("search: token={} query={}", token, query);
                    }
                    Some(Command::QueueDownload { username, filename, size }) => {
                        tracing::debug!("queue: {} from {} ({} bytes)", filename, username, size);
                    }
                    Some(Command::Connect { .. }) => {
                        tracing::warn!("already connected, ignoring Connect command");
                    }
                    None => break,
                }
            }
        }
    }

    let _ = evt_tx.send(Event::Disconnected { reason: None }).await;
    Ok(())
}

async fn send_login(
    stream: &mut TcpStream,
    username: &str,
    password: &str,
    major_version: u32,
    minor_version: u32,
) -> Result<(), std::io::Error> {
    let md5_input = format!("{}{}", username, password);
    let md5_hash = md5::compute(md5_input.as_bytes());
    let hash_hex = format!("{:x}", md5_hash);

    let req = server::login::LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
        major_version,
        hash: hash_hex,
        minor_version,
    };

    let frame = encode_message(server::login::CODE, &req);
    stream.write_all(&frame).await?;
    Ok(())
}

/// Read and decode one server message from the stream
async fn read_one_message(
    stream: &mut TcpStream,
    stats: &Arc<ServerStats>,
) -> Result<Option<(u32, Bytes)>, std::io::Error> {
    // Read length prefix
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let total_len = u32::from_le_bytes(len_buf) as usize;

    if total_len > MAX_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frame too large: {} bytes", total_len)
        ));
    }

    // Read code + payload
    let mut frame_buf = vec![0u8; total_len];
    stream.read_exact(&mut frame_buf).await?;

    stats.bytes_recv.fetch_add((4 + total_len) as u64, std::sync::atomic::Ordering::Relaxed);
    stats.messages_recv.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mut buf = BytesMut::with_capacity(total_len);
    buf.put_slice(&frame_buf);
    let code = buf.get_u32_le();
    let payload = buf.freeze();

    Ok(Some((code, payload)))
}

/// Read a specific message type by code
async fn read_server_message<T: slsk_proto::codec::SlskRead>(
    stream: &mut TcpStream,
    expected_code: u32,
    stats: &Arc<ServerStats>,
) -> Result<T, crate::error::Error> {
    let (code, mut payload) = read_one_message(stream, stats)
        .await?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no message"))?;

    if code != expected_code {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("expected code {}, got {}", expected_code, code)
        ).into());
    }

    let msg = T::read(&mut payload)?;
    Ok(msg)
}

async fn handle_server_message(code: u32, mut payload: Bytes, evt_tx: &mpsc::Sender<Event>) {
    let msg = match ServerMessage::decode(code, &mut payload) {
        Ok(msg) => msg,
        Err(e) => {
            tracing::warn!("failed to decode server message code {}: {}", code, e);
            return;
        }
    };

    match msg {
        ServerMessage::Login(_) => {
            // Already handled during login sequence
        }
        ServerMessage::Relogged(_) => {
            tracing::warn!("received Relogged - closing connection");
            let _ = evt_tx.send(Event::Disconnected { reason: Some("Relogged".into()) }).await;
        }
        ServerMessage::ServerPing => {
            tracing::trace!("server ping");
        }
        ServerMessage::FileSearch(resp) => {
            let _ = evt_tx.send(Event::SearchResult { token: resp.token, username: resp.username }).await;
        }
        _ => {
            tracing::trace!("unhandled server message: {:?}", msg);
        }
    }
}

/// Create a SetWaitPort frame
fn make_set_wait_port(port: u32) -> Bytes {
    encode_message(server::set_wait_port::CODE, &server::set_wait_port::SetWaitPortRequest::new(port))
}
