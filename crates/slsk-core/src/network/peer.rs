//! Peer-to-peer connection handling.
//!
//! Soulseek search results arrive via peer connections. When we send a
//! FileSearchRequest (code 26), peers with matching files initiate P connections
//! to us and send FileSearchResponse (code 9) messages containing the actual
//! filename and size data.
//!
//! Additionally, searches are propagated through the distributed network via
//! parent/child D connections using DistribSearch messages.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::read::ZlibDecoder;
use slsk_proto::codec::{SlskRead, SlskWrite};
use slsk_proto::peer;
use slsk_proto::peer_init::pierce_firewall::PierceFirewallRequest;
use slsk_proto::peer_init::req::PeerInitRequest;
use slsk_proto::types::ConnectionType;
use std::collections::HashMap;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

use crate::event::Event;

// Maximum frame size for peer messages
const MAX_PEER_FRAME_SIZE: usize = 2 * 1024 * 1024; // 2 MiB

/// Shared peer state across all connections
#[derive(Clone)]
pub struct PeerState {
    /// Our username, used in peer init messages
    pub username: String,
    /// Our listen port
    pub listen_port: u32,
    /// Active peer connections, keyed by peer username
    pub connections: Arc<Mutex<HashMap<String, PeerConnection>>>,
    /// Sender to emit events back to core
    evt_tx: mpsc::Sender<Event>,
}

impl PeerState {
    pub fn new(username: String, listen_port: u32, evt_tx: mpsc::Sender<Event>) -> Self {
        Self {
            username,
            listen_port,
            connections: Arc::new(Mutex::new(HashMap::new())),
            evt_tx,
        }
    }

    /// Handle an incoming peer connection (peer initiated to us)
    pub async fn handle_incoming(&self, mut stream: TcpStream, addr: SocketAddr) {
        tracing::info!("incoming peer connection from {}", addr);
        // Read peer init message to identify the peer
        match read_peer_init_frame(&mut stream).await {
            Ok((code, payload)) => {
                match code {
                    0 => {
                        // PierceFirewall - peer is connecting indirectly via server relay
                        let mut buf = payload.clone();
                        match PierceFirewallRequest::read(&mut buf) {
                            Ok(pf) => {
                                tracing::info!("PierceFirewall from {} token={}", addr, pf.token);
                                // TODO: respond appropriately for indirect connections
                            }
                            Err(e) => {
                                tracing::warn!("failed to parse PierceFirewall: {}", e);
                            }
                        }
                    }
                    1 => {
                        // PeerInit - direct connection
                        let mut buf = payload.clone();
                        match PeerInitRequest::read(&mut buf) {
                            Ok(init) => {
                                tracing::info!(
                                    "peer init: {} type={} version={} from {}",
                                    init.username,
                                    init.conn_type,
                                    init.version,
                                    addr
                                );
                                let _conn_type = if init.conn_type == "P" {
                                    ConnectionType::PeerToPeer
                                } else if init.conn_type == "F" {
                                    ConnectionType::FileTransfer
                                } else {
                                    ConnectionType::Distributed
                                };
                                let (reader, _writer) = tokio::io::split(stream);
                                let conn = PeerConnection::new(init.username.clone(), addr);
                                // Store connection and spawn read loop
                                let username_clone = conn.username.clone();
                                self.connections
                                    .lock()
                                    .await
                                    .insert(username_clone.clone(), conn);
                                tokio::spawn(peer_read_loop(
                                    username_clone,
                                    reader,
                                    self.evt_tx.clone(),
                                ));
                            }
                            Err(e) => {
                                tracing::warn!("failed to parse PeerInit: {}", e);
                            }
                        }
                    }
                    other => {
                        tracing::warn!("unknown peer init code {} from {}", other, addr);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("failed to read peer init from {}: {}", addr, e);
            }
        }
    }

    /// Connect to a peer given address info from ConnectToPeer server message
    pub async fn connect_to_peer(
        &self,
        username: &str,
        ip: u32,
        port: u32,
        token: u32,
        conn_type: slsk_proto::types::ConnectionType,
    ) -> Result<(), std::io::Error> {
        let addr = SocketAddr::from((IpAddr::V4(Ipv4Addr::from(u32::from_be(ip))), port as u16));
        tracing::info!(
            "connecting to peer {} at {} (token={}, type={:?})",
            username,
            addr,
            token,
            conn_type
        );

        let stream = match TcpStream::connect(addr).await {
            Ok(s) => {
                tracing::info!("TCP connected to peer {}, sending PeerInit...", username);
                s
            }
            Err(e) => {
                tracing::warn!("TCP connection to peer {} failed: {}", username, e);
                return Err(e);
            }
        };

        let conn = match PeerConnection::new_outgoing(
            username.to_string(),
            &self.username,
            self.listen_port,
            conn_type,
            stream,
            self.evt_tx.clone(),
        )
        .await
        {
            Ok(c) => {
                tracing::info!("PeerInit sent to {}, read loop spawned", username);
                c
            }
            Err(e) => {
                tracing::warn!("PeerInit to {} failed: {}", username, e);
                return Err(e);
            }
        };

        // Store connection so it's not dropped
        self.connections
            .lock()
            .await
            .insert(username.to_string(), conn);
        Ok(())
    }
}

/// A single peer connection
pub struct PeerConnection {
    pub username: String,
    pub addr: SocketAddr,
}

impl PeerConnection {
    /// Create for incoming connection (we are the passive side)
    pub fn new(username: String, addr: SocketAddr) -> Self {
        Self { username, addr }
    }

    /// Create outgoing connection (we are the active side)
    /// Sends PeerInit message after connecting
    pub async fn new_outgoing(
        username: String,
        our_username: &str,
        _our_port: u32,
        conn_type: ConnectionType,
        stream: TcpStream,
        evt_tx: mpsc::Sender<Event>,
    ) -> Result<Self, std::io::Error> {
        let peer_addr = stream.peer_addr()?;
        let (reader, mut writer) = tokio::io::split(stream);
        // Send PeerInit message
        let conn_type_str = match conn_type {
            ConnectionType::PeerToPeer => "P",
            ConnectionType::FileTransfer => "F",
            ConnectionType::Distributed => "D",
        };
        let init = PeerInitRequest {
            username: our_username.to_string(),
            conn_type: conn_type_str.to_string(),
            version: 182, // Menthol's major version
        };
        let mut buf = BytesMut::new();
        init.write(&mut buf);
        let frame = encode_peer_init_frame(1, &buf.freeze()); // code 1 = PeerInit
        writer.write_all(&frame).await?;

        tracing::info!(
            "sent PeerInit to {} as {} type={}",
            peer_addr,
            our_username,
            conn_type_str
        );

        let conn = Self {
            username,
            addr: peer_addr,
        };
        // Spawn read loop
        let username_clone = conn.username.clone();
        tokio::spawn(peer_read_loop(username_clone, reader, evt_tx));

        Ok(conn)
    }
}

/// Encode a peer init message frame: [u32 len][u8 code][payload]
/// Note: peer init uses u8 code, not u32 like regular peer messages
fn encode_peer_init_frame(code: u8, payload: &Bytes) -> BytesMut {
    let total_len = 1 + payload.len() as u32;
    let mut frame = BytesMut::with_capacity(4 + total_len as usize);
    frame.put_u32_le(total_len);
    frame.put_u8(code);
    frame.put_slice(payload);
    frame
}

/// Read a peer init frame (first message on an incoming connection)
async fn read_peer_init_frame(stream: &mut TcpStream) -> Result<(u8, Bytes), std::io::Error> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let total_len = u32::from_le_bytes(len_buf) as usize;

    if total_len > MAX_PEER_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("peer init frame too large: {} bytes", total_len),
        ));
    }

    let mut frame_buf = vec![0u8; total_len];
    stream.read_exact(&mut frame_buf).await?;

    let mut buf = BytesMut::with_capacity(total_len);
    buf.put_slice(&frame_buf);
    let code = buf.get_u8();
    let payload = buf.freeze();

    Ok((code, payload))
}

/// Encode a peer message frame: [u32 len][u32 code][payload]
fn encode_peer_frame(code: u32, payload: &Bytes) -> BytesMut {
    let total_len = 4 + payload.len() as u32;
    let mut frame = BytesMut::with_capacity(4 + total_len as usize);
    frame.put_u32_le(total_len);
    frame.put_u32_le(code);
    frame.put_slice(payload);
    frame
}

/// Read a peer message frame
async fn read_peer_frame(stream: &mut TcpStream) -> Result<(u32, Bytes), std::io::Error> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let total_len = u32::from_le_bytes(len_buf) as usize;

    if total_len > MAX_PEER_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("peer frame too large: {} bytes", total_len),
        ));
    }

    let mut frame_buf = vec![0u8; total_len];
    stream.read_exact(&mut frame_buf).await?;

    let mut buf = BytesMut::with_capacity(total_len);
    buf.put_slice(&frame_buf);
    let code = buf.get_u32_le();
    let payload = buf.freeze();

    Ok((code, payload))
}

/// Read and dispatch peer messages from an established connection
pub async fn peer_read_loop(
    username: String,
    mut reader: tokio::io::ReadHalf<TcpStream>,
    evt_tx: mpsc::Sender<Event>,
) {
    tracing::info!(
        "peer_read_loop started for {}, waiting for messages...",
        username
    );
    loop {
        match read_peer_frame_raw(&mut reader).await {
            Ok((code, payload)) => {
                tracing::info!(
                    "peer {} received message code={} len={}",
                    username,
                    code,
                    payload.len()
                );
                let mut payload = payload;
                if let Err(e) = handle_peer_message(code, &mut payload, &username, &evt_tx).await {
                    tracing::warn!("error handling peer message from {}: {}", username, e);
                }
            }
            Err(e) => {
                tracing::info!("peer {} disconnected: {}", username, e);
                break;
            }
        }
    }
    tracing::info!("peer_read_loop ended for {}", username);
}

/// Read a peer message frame from a split stream
async fn read_peer_frame_raw(
    reader: &mut tokio::io::ReadHalf<TcpStream>,
) -> Result<(u32, Bytes), std::io::Error> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let total_len = u32::from_le_bytes(len_buf) as usize;

    if total_len > MAX_PEER_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("peer frame too large: {} bytes", total_len),
        ));
    }

    let mut frame_buf = vec![0u8; total_len];
    reader.read_exact(&mut frame_buf).await?;

    let mut buf = BytesMut::with_capacity(total_len);
    buf.put_slice(&frame_buf);
    let code = buf.get_u32_le();
    let payload = buf.freeze();

    Ok((code, payload))
}

/// Handle a single peer message
async fn handle_peer_message(
    code: u32,
    payload: &mut Bytes,
    from_username: &str,
    evt_tx: &mpsc::Sender<Event>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match code {
        peer::file_search_response::CODE => {
            // FileSearchResponse payload is zlib-compressed per SEARCH_SYSTEM.md §6
            // Max uncompressed size: 128 MiB per spec
            tracing::info!(
                "peer {} FileSearchResponse RECEIVED, compressed len={}",
                from_username,
                payload.len()
            );
            const MAX_UNCOMPRESSED: usize = 128 * 1024 * 1024;
            let decompressed = match decompress_zlib(payload, MAX_UNCOMPRESSED) {
                Ok(d) => {
                    tracing::info!(
                        "peer {} FileSearchResponse decompressed, {} bytes",
                        from_username,
                        d.len()
                    );
                    d
                }
                Err(e) => {
                    tracing::warn!(
                        "peer {} FileSearchResponse zlib decompression failed: {}",
                        from_username,
                        e
                    );
                    return Ok(());
                }
            };
            let mut decomp = decompressed;
            match peer::file_search_response::FileSearchResponse::read(&mut decomp) {
                Ok(resp) => {
                    tracing::info!(
                        "peer {} FileSearchResponse PARSED: token={} file={} size={}",
                        from_username,
                        resp.token,
                        resp.result.filename,
                        resp.result.size
                    );
                    let send_result = evt_tx
                        .send(Event::SearchResult {
                            token: resp.token,
                            username: from_username.to_string(),
                            filename: resp.result.filename,
                            size: resp.result.size,
                        })
                        .await;
                    if let Err(e) = send_result {
                        tracing::warn!(
                            "peer {} failed to send SearchResult event to GUI: {}",
                            from_username,
                            e
                        );
                    } else {
                        tracing::info!(
                            "peer {} SearchResult emitted to GUI: token={}",
                            from_username,
                            resp.token
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "peer {} failed to parse FileSearchResponse: {}",
                        from_username,
                        e
                    );
                }
            }
        }
        peer::transfer_request::CODE => {
            let resp = peer::transfer_request::TransferRequest::read(payload)?;
            tracing::info!(
                "TransferRequest from {}: file={} size={:?} token={}",
                from_username,
                resp.filename,
                resp.file_size,
                resp.token
            );
            let _ = evt_tx
                .send(Event::TransferRequest {
                    username: from_username.to_string(),
                    filename: resp.filename,
                    size: resp.file_size.unwrap_or(0),
                    token: resp.token,
                })
                .await;
        }
        peer::queue_upload::CODE => {
            let resp = peer::queue_upload::QueueUpload::read(payload)?;
            tracing::debug!("peer {} QueueUpload: file={}", from_username, resp.filename);
        }
        _ => {
            tracing::info!(
                "peer {} unhandled message code {} (len={})",
                from_username,
                code,
                payload.len()
            );
        }
    }
    Ok(())
}

/// Decompress zlib-encoded payload with size limit per SEARCH_SYSTEM.md §6.4
fn decompress_zlib(input: &Bytes, max_size: usize) -> Result<Bytes, std::io::Error> {
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(&input[..]);
    let mut decompressed = Vec::new();

    // Limit decompressed output to prevent memory exhaustion
    let mut buf = vec![0u8; 8192];
    loop {
        if decompressed.len() > max_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("decompressed size exceeds {} bytes", max_size),
            ));
        }
        let n = decoder.read(&mut buf)?;
        if n == 0 {
            break;
        }
        decompressed.extend_from_slice(&buf[..n]);
    }

    Ok(Bytes::from(decompressed))
}
