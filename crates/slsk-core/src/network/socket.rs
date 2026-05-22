//! Socket configuration for different connection types.
//!
//! Per spec §3.2: File connections (F) are left uncapped for maximum throughput.
//! Distributed connections (D) are capped at 16 KB. Peer connections (P) at 208 KB.

use slsk_proto::types::ConnectionType;
use tokio::net::TcpStream;

/// Configure socket receive buffer size based on connection type.
///
/// - `File`: uncapped (OS default, typically 128 KB+ with auto-tuning)
/// - `Distributed`: 16 KB (prevents flooding)
/// - `Peer`: 208 KB (moderate buffering)
pub fn configure_socket(socket: &TcpStream, conn_type: ConnectionType) -> std::io::Result<()> {
    let sock_ref = socket2::SockRef::from(socket);
    match conn_type {
        ConnectionType::FileTransfer => {
            // Intentionally uncapped — OS default allows maximum throughput.
            // Linux default: 128 KB (net.core.rmem_default), can be higher
            // if kernel auto-tuning is enabled (net.ipv4.tcp_moderate_rcvbuf).
        }
        ConnectionType::Distributed => {
            sock_ref.set_recv_buffer_size(16_384)?;
        }
        ConnectionType::PeerToPeer => {
            sock_ref.set_recv_buffer_size(208_896)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_type_to_string() {
        assert_eq!(format!("{:?}", ConnectionType::FileTransfer), "FileTransfer");
        assert_eq!(format!("{:?}", ConnectionType::PeerToPeer), "PeerToPeer");
        assert_eq!(format!("{:?}", ConnectionType::Distributed), "Distributed");
    }
}
