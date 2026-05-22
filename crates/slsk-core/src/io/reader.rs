//! Adaptive receive buffer for file transfers.
//!
//! Starts at 50 KB and grows/shrinks dynamically based on how much data
//! each recv call returns. This is the primary throughput optimization
//! described in the transfer pipeline spec.

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::mpsc;

/// Initial receive buffer size (50 KB)
const RECV_INITIAL: usize = 51_200;
/// Maximum receive buffer size (4 MiB)
const RECV_MAX: usize = 4 * 1024 * 1024;
/// Channel capacity for SocketReader → DiskWriter
const CHANNEL_CAPACITY: usize = 16;

/// Start a socket reader task for a file download.
///
/// Reads from the socket with an adaptive buffer, sending chunks to the
/// DiskWriter via a bounded mpsc channel. Provides natural backpressure:
/// if disk IO is slow, the channel fills and recv pauses.
pub async fn socket_reader_task<R>(
    reader: R,
    tx: mpsc::Sender<Bytes>,
    mut recv_size: usize,
) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
{
    let mut reader = reader;
    let mut total_bytes: u64 = 0;
    let mut buf = bytes::BytesMut::with_capacity(recv_size);

    loop {
        buf.reserve(recv_size);
        let n = reader.read_buf(&mut buf).await?;

        if n == 0 {
            break;
        }

        total_bytes += n as u64;

        // Adaptive buffer sizing per spec §3.1
        if n >= recv_size / 2 {
            recv_size = (recv_size * 2).min(RECV_MAX);
        } else if n <= recv_size / 6 {
            recv_size = (recv_size / 2).max(RECV_INITIAL);
        }

        tx.send(buf.split().freeze())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))?;
    }

    Ok(total_bytes)
}

/// Create a bounded channel for socket-reader to disk-writer communication.
pub fn channel() -> (mpsc::Sender<Bytes>, mpsc::Receiver<Bytes>) {
    mpsc::channel(CHANNEL_CAPACITY)
}

/// Initial receive buffer size constant
pub const INITIAL_RECV_SIZE: usize = RECV_INITIAL;
/// Maximum receive buffer size constant
pub const MAX_RECV_SIZE: usize = RECV_MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(RECV_INITIAL, 51_200);
        assert_eq!(RECV_MAX, 4 * 1024 * 1024);
        assert_eq!(CHANNEL_CAPACITY, 16);
    }
}
