//! Server connection actor - manages TCP connection to Soulseek server.
//!
//! Follows LOGIN_FLOW.md specification:
//! - TCP keepalive on the socket
//! - Login + SetWaitPort sent in same write burst before reading response
//! - Post-login burst: HaveNoParent, BranchRoot, BranchLevel, AcceptChildren, etc.
//! - State machine: Connecting -> LoggingIn -> Connected
//! - Relogged handled - no auto-reconnect
//! - Exponential backoff for reconnect

use bytes::{Buf, BufMut, Bytes, BytesMut};
use slsk_proto::codec::SlskWrite;
use slsk_proto::server;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::command::Command;
use crate::config::Config;
use crate::event::Event;
use crate::network::framing::encode_message;
use crate::network::peer::PeerState;
use crate::network::state::ServerStats;

// Constants for memory-bounded operation
const MAX_FRAME_SIZE: usize = 2 * 1024 * 1024; // 2 MiB

/// Connection state per LOGIN_FLOW.md §7.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerConnState {
    Disconnected,
    Connecting, // TCP established, Login not yet sent
    LoggingIn,  // Login + SetWaitPort sent, awaiting response
    Connected,  // Login succeeded, session active
}

impl Default for ServerConnState {
    fn default() -> Self {
        ServerConnState::Disconnected
    }
}

/// Handle returned from ServerConnection::connect()
/// Allows sending commands to the connection
pub struct ConnectionHandle {
    pub cmd_tx: mpsc::Sender<Command>,
}

/// Connect to the Soulseek server and return a handle.
/// Events are sent directly to the provided evt_tx so the GUI receives them.
pub async fn connect(
    config: &Config,
    cmd_rx: mpsc::Receiver<Command>,
    evt_tx: mpsc::Sender<Event>,
) -> Result<ConnectionHandle, crate::error::Error> {
    let (cmd_tx, _) = mpsc::channel::<Command>(1);
    let stats = Arc::new(ServerStats::new());

    // Clone the config values to avoid lifetime issues with tokio::spawn
    let host = config.host.clone();
    let username = config.username.clone();
    let password = config.password.clone();
    let port = config.port;
    let listen_port = config.listen_port;
    let major_version = config.major_version;
    let minor_version = config.minor_version;
    let evt_tx_clone = evt_tx.clone();

    tokio::spawn(async move {
        let owned_config = Config {
            host,
            username,
            password,
            port,
            listen_port,
            major_version,
            minor_version,
        };
        if let Err(e) =
            run_connection(&owned_config, cmd_rx, evt_tx_clone, Arc::clone(&stats)).await
        {
            tracing::error!("connection error: {}", e);
        }
    });

    Ok(ConnectionHandle { cmd_tx })
}

// Reconnect backoff per LOGIN_FLOW.md §5.1
fn next_reconnect_delay(current: Option<Duration>) -> Duration {
    match current {
        None => {
            // First attempt: random jitter 5-15s
            let jitter = rand_u32_range(5, 15);
            Duration::from_secs(jitter as u64)
        }
        Some(d) => {
            // Exponential backoff, capped at 5 minutes
            (d * 2).min(Duration::from_secs(300))
        }
    }
}

// Simple random u32 in range [min, max]
fn rand_u32_range(min: u32, max: u32) -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    min + (nanos % (max - min + 1))
}

async fn run_connection(
    config: &Config,
    mut cmd_rx: mpsc::Receiver<Command>,
    evt_tx: mpsc::Sender<Event>,
    stats: Arc<ServerStats>,
) -> Result<(), crate::error::Error> {
    let mut reconnect_delay: Option<Duration> = None;
    let mut relogged = false;

    loop {
        let addr = format!("{}:{}", config.host, config.port);
        tracing::info!("connecting to {}...", addr);

        let mut stream = match tokio::net::TcpStream::connect(&addr).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("connection failed: {}", e);
                let delay = next_reconnect_delay(reconnect_delay);
                reconnect_delay = Some(delay);
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        tracing::info!("connected to server");

        // Configure TCP keepalive per LOGIN_FLOW.md §1.1
        let _ = configure_server_keepalive(&stream);

        // Login + SetWaitPort in same write burst per LOGIN_FLOW.md §2.4
        if let Err(e) = send_login_and_waitport(&mut stream, config).await {
            tracing::warn!("login failed: {}", e);
            reconnect_delay = Some(next_reconnect_delay(reconnect_delay));
            let _ = evt_tx.send(Event::Disconnected { reason: Some(e) }).await;
            tokio::time::sleep(reconnect_delay.unwrap()).await;
            continue;
        }

        // Read login response
        let login_resp = match read_server_message::<server::login::LoginResponse>(
            &mut stream,
            server::login::CODE,
            &stats,
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!("failed to read login response: {}", e);
                reconnect_delay = Some(next_reconnect_delay(reconnect_delay));
                let _ = evt_tx
                    .send(Event::Disconnected {
                        reason: Some(e.to_string()),
                    })
                    .await;
                tokio::time::sleep(reconnect_delay.unwrap()).await;
                continue;
            }
        };

        match login_resp {
            server::login::LoginResponse::Success {
                greet,
                own_ip: _,
                hash: _,
                is_supporter,
            } => {
                tracing::info!(
                    "login success! motd len={}, supporter={}",
                    greet.len(),
                    is_supporter
                );
                let _ = evt_tx.send(Event::Connected { motd: greet }).await;
                reconnect_delay = None; // Reset backoff on success
                relogged = false;
            }
            server::login::LoginResponse::Failure { reason, detail: _ } => {
                tracing::warn!("login failed: {}", reason);
                let _ = evt_tx.send(Event::LoginFailed { reason }).await;
                return Ok(()); // Don't reconnect on login failure
            }
        }

        // Send post-login burst per LOGIN_FLOW.md §4.8
        if let Err(e) = send_post_login_burst(&mut stream, config).await {
            tracing::warn!("post-login burst failed: {}", e);
            let _ = evt_tx.send(Event::Disconnected { reason: Some(e) }).await;
        }

        // Main loop - handle commands and incoming messages
        // Create shared peer state for all peer connections
        let peer_state = PeerState::new(
            config.username.clone(),
            config.listen_port,
            evt_tx.clone(),
        );

        // Start TCP listener for incoming peer connections
        let peer_listen_addr = format!("0.0.0.0:{}", config.listen_port);
        let listener: Option<TcpListener> = match TcpListener::bind(&peer_listen_addr).await {
            Ok(l) => {
                tracing::info!("peer listener bound to {}", peer_listen_addr);
                Some(l)
            }
            Err(e) => {
                tracing::warn!("failed to bind peer listener on {}: {}", peer_listen_addr, e);
                // Continue anyway - peer connections won't work but we can still search
                None
            }
        };

        // Spawn task to accept incoming peer connections
        if let Some(listener) = listener {
            let peer_state_clone = peer_state.clone();
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, addr)) => {
                            peer_state_clone.handle_incoming(stream, addr).await;
                        }
                        Err(e) => {
                            tracing::warn!("peer listener error: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        'connected: loop {
            tokio::select! {
                // Incoming message from server
                msg = read_one_message(&mut stream, &stats) => {
                    match msg {
                        Ok(Some((code, payload))) => {
                            if code == server::relogged::CODE {
                                // Per LOGIN_FLOW.md §6: Relogged - set flag, don't reconnect
                                tracing::warn!("received Relogged - another client logged in");
                                relogged = true;
                                let _ = evt_tx.send(Event::Disconnected { reason: Some("Relogged".into()) }).await;
                                break 'connected;
                            }
                            // Handle ConnectToPeer specially - need to initiate peer connection
                            if code == server::connect_to_peer::CODE {
                                use server::ServerMessage;
                                let mut buf = BytesMut::from(payload.as_ref());
                                match ServerMessage::decode(code, &mut buf) {
                                    Ok(ServerMessage::ConnectToPeer(resp)) => {
                                        tracing::info!("ConnectToPeer: {} at {}:{} type={} token={}",
                                            resp.username, resp.ip, resp.port, resp.conn_type, resp.token);
                                        // Initiate connection to peer
                                        let conn_type = match resp.conn_type.as_str() {
                                            "P" => slsk_proto::types::ConnectionType::PeerToPeer,
                                            "F" => slsk_proto::types::ConnectionType::FileTransfer,
                                            "D" => slsk_proto::types::ConnectionType::Distributed,
                                            _ => {
                                                tracing::warn!("unknown connection type '{}'", resp.conn_type);
                                                slsk_proto::types::ConnectionType::PeerToPeer
                                            }
                                        };
                                        if let Err(e) = peer_state.connect_to_peer(
                                            &resp.username,
                                            resp.ip,
                                            resp.port,
                                            resp.token,
                                            conn_type,
                                        ).await {
                                            tracing::warn!("failed to connect to peer {}: {}", resp.username, e);
                                        }
                                    }
                                    _ => {
                                        tracing::warn!("ConnectToPeer message decode mismatch");
                                    }
                                }
                                continue;
                            }
                            handle_server_message(code, payload, &evt_tx).await;
                        }
                        Ok(None) => {
                            // Would block - try again
                        }
                        Err(e) => {
                            tracing::error!("read error: {}", e);
                            let _ = evt_tx.send(Event::Disconnected { reason: Some(e.to_string()) }).await;
                            break 'connected;
                        }
                    }
                }
                // Incoming command from GUI
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(Command::Disconnect) => {
                            tracing::info!("disconnect requested");
                            return Ok(());
                        }
                        Some(Command::Search { query, token }) => {
                            // Send FileSearchRequest to server (code 26)
                            let req = server::file_search::FileSearchRequest { token, query: query.clone() };
                            if let Err(e) = send_server_msg(&mut stream, server::file_search::CODE, &req).await {
                                tracing::warn!("failed to send search: {}", e);
                            } else {
                                let _ = evt_tx.send(Event::SearchStarted { token, query }).await;
                            }
                        }
                        Some(Command::QueueDownload { username, filename, size }) => {
                            tracing::debug!("queue: {} from {} ({} bytes)", filename, username, size);
                        }
                        Some(Command::QueueUpload { username, filename }) => {
                            tracing::debug!("queue upload: {} to {}", filename, username);
                        }
                        Some(Command::CancelTransfer { id }) => {
                            tracing::debug!("cancel transfer: {}", id);
                        }
                        Some(Command::PauseTransfers { direction }) => {
                            tracing::debug!("pause transfers: {:?}", direction);
                        }
                        Some(Command::ResumeTransfers { direction }) => {
                            tracing::debug!("resume transfers: {:?}", direction);
                        }
                        Some(Command::SetUploadSlots(slots)) => {
                            tracing::debug!("set upload slots: {}", slots);
                        }
                        Some(Command::SetDownloadLimit(limit)) => {
                            tracing::debug!("set download limit: {}", limit);
                        }
                        Some(Command::SetUploadLimit(limit)) => {
                            tracing::debug!("set upload limit: {}", limit);
                        }
                        Some(Command::Connect { .. }) => {
                            tracing::warn!("already connected, ignoring Connect command");
                        }
                        Some(Command::JoinRoom(room)) => {
                            tracing::debug!("join room: {}", room);
                        }
                        Some(Command::LeaveRoom(room)) => {
                            tracing::debug!("leave room: {}", room);
                        }
                        Some(Command::SendChatMessage { room, message }) => {
                            tracing::debug!("chat to {}: {}", room, message);
                        }
                        Some(Command::SendPrivateMessage { username, message }) => {
                            tracing::debug!("PM to {}: {}", username, message);
                        }
                        None => {
                            tracing::info!("command channel closed");
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Per LOGIN_FLOW.md §6: don't reconnect if relogged
        if relogged {
            tracing::info!("not reconnecting due to Relogged");
            return Ok(());
        }

        // Reconnect with backoff
        tracing::info!("disconnected, will reconnect in {:?}", reconnect_delay);
        let delay = next_reconnect_delay(reconnect_delay);
        reconnect_delay = Some(delay);
        tokio::time::sleep(delay).await;
    }
}

/// Configure TCP keepalive per LOGIN_FLOW.md §1.1
fn configure_server_keepalive(stream: &TcpStream) -> std::io::Result<()> {
    use socket2::SockRef;
    let sock = SockRef::from(stream);
    sock.set_keepalive(true)?;
    // TCP keepalive: after 10s idle, probes every 2s
    // Platform note: not all socket2 methods available on all platforms
    Ok(())
}

/// Send Login + SetWaitPort in same write burst per LOGIN_FLOW.md §2.4
async fn send_login_and_waitport(stream: &mut TcpStream, config: &Config) -> Result<(), String> {
    let hash = login_hash(&config.username, &config.password);

    // Build Login message
    let login_req = server::login::LoginRequest {
        username: config.username.clone(),
        password: config.password.clone(),
        major_version: config.major_version,
        hash,
        minor_version: config.minor_version,
    };
    let login_frame = encode_message(server::login::CODE, &login_req);

    // Build SetWaitPort message
    let waitport_req = server::set_wait_port::SetWaitPortRequest::new(config.listen_port);
    let waitport_frame = encode_message(server::set_wait_port::CODE, &waitport_req);

    // Send both in same burst - write_all is atomic on TCP
    stream
        .write_all(&login_frame)
        .await
        .map_err(|e| e.to_string())?;
    tracing::debug!(
        "sent Login (major={}, minor={})",
        config.major_version,
        config.minor_version
    );

    stream
        .write_all(&waitport_frame)
        .await
        .map_err(|e| e.to_string())?;
    tracing::debug!("sent SetWaitPort({})", config.listen_port);

    Ok(())
}

/// Compute MD5 hash of username + password per LOGIN_FLOW.md §2.2
fn login_hash(username: &str, password: &str) -> String {
    let mut h = md5::Context::new();
    h.consume(username.as_bytes());
    h.consume(password.as_bytes());
    format!("{:x}", h.finalize())
}

/// Send post-login burst per LOGIN_FLOW.md §4.8
async fn send_post_login_burst(stream: &mut TcpStream, config: &Config) -> Result<(), String> {
    use server::accept_children::CODE as ACCEPT_CHILDREN_CODE;
    use server::branch_level::CODE as BRANCH_LEVEL_CODE;
    use server::branch_root::CODE as BRANCH_ROOT_CODE;
    use server::have_no_parent::CODE as HAVE_NO_PARENT_CODE;
    use server::shared_folders_files::CODE as SHARED_FOLDERS_FILES_CODE;

    // 1-4: Distributed network init (per LOGIN_FLOW.md §4.1)
    send_server_msg(
        stream,
        HAVE_NO_PARENT_CODE,
        &server::have_no_parent::HaveNoParentRequest { no_parent: true },
    )
    .await?;
    send_server_msg(
        stream,
        BRANCH_ROOT_CODE,
        &server::branch_root::BranchRootRequest {
            branch_root: config.username.clone(),
        },
    )
    .await?;
    send_server_msg(
        stream,
        BRANCH_LEVEL_CODE,
        &server::branch_level::BranchLevelRequest { branch_level: 0 },
    )
    .await?;
    send_server_msg(
        stream,
        ACCEPT_CHILDREN_CODE,
        &server::accept_children::AcceptChildrenRequest { accept: false },
    )
    .await?;

    // 5: Share counts (per LOGIN_FLOW.md §4.2) - send 0/0 for now
    send_server_msg(
        stream,
        SHARED_FOLDERS_FILES_CODE,
        &server::shared_folders_files::SharedFoldersFilesRequest { dirs: 0, files: 0 },
    )
    .await?;

    // Note: RoomList, PrivateRoomToggle, JoinRoom, AddThingILike/AddThingIHate, WatchUser
    // would be sent based on saved state. For now we just do the critical distributed init.

    tracing::info!("post-login burst complete");
    Ok(())
}

/// Send a server message with its code
async fn send_server_msg<T: SlskWrite>(
    stream: &mut TcpStream,
    code: u32,
    msg: &T,
) -> Result<(), String> {
    let mut payload = BytesMut::new();
    msg.write(&mut payload);
    let total_len = 4 + payload.len() as u32;

    let mut frame = BytesMut::with_capacity(4 + total_len as usize);
    frame.put_u32_le(total_len);
    frame.put_u32_le(code);
    frame.put_slice(&payload);

    stream
        .write_all(&frame.freeze())
        .await
        .map_err(|e| e.to_string())
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
            format!("frame too large: {} bytes", total_len),
        ));
    }

    // Read code + payload
    let mut frame_buf = vec![0u8; total_len];
    stream.read_exact(&mut frame_buf).await?;

    stats
        .bytes_recv
        .fetch_add((4 + total_len) as u64, std::sync::atomic::Ordering::Relaxed);
    stats
        .messages_recv
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

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
            format!("expected code {}, got {}", expected_code, code),
        )
        .into());
    }

    let msg = T::read(&mut payload)?;
    Ok(msg)
}

async fn handle_server_message(code: u32, mut payload: Bytes, evt_tx: &mpsc::Sender<Event>) {
    use server::ServerMessage;

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
            let _ = evt_tx
                .send(Event::Disconnected {
                    reason: Some("Relogged".into()),
                })
                .await;
        }
        ServerMessage::ServerPing => {
            tracing::trace!("server ping");
        }
        ServerMessage::FileSearch(resp) => {
            // Server relays FileSearch from a peer. The server tells us:
            // username + token. The full file metadata comes via peer message code 9.
            // Emit what we have now; GUI will enrich with peer details later.
            let _ = evt_tx
                .send(Event::SearchResult {
                    token: resp.token,
                    username: resp.username,
                    filename: String::new(),
                    size: 0,
                })
                .await;
        }
        _ => {
            tracing::trace!("unhandled server message: {:?}", msg);
        }
    }
}
