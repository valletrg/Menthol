//! Disk reader task for file uploads with read-ahead.
//!
//! Reads from disk and sends chunks to SocketWriter via a bounded channel.
//! Chunk size grows proportionally to last send (1.25×), keeping the socket
//! send buffer pre-filled so the kernel always has data ready to transmit.

use bytes::{Bytes, BytesMut};
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use tokio::sync::mpsc;

/// Initial chunk size (4 KB)
const CHUNK_INITIAL: usize = 4_096;
/// Maximum chunk size (512 KB)
const CHUNK_MAX: usize = 512 * 1024;
/// Channel capacity (16 chunks = 8 MB max pre-read)
const CHANNEL_CAPACITY: usize = 16;

/// Start a disk reader task for an upload.
///
/// Reads from `file` starting at `offset`, sending chunks to `tx`.
/// Chunk size adapts based on last send (1.25× growth).
pub async fn disk_reader_task<R>(
    mut file: R,
    tx: mpsc::Sender<Bytes>,
    offset: u64,
) -> std::io::Result<u64>
where
    R: AsyncRead + AsyncSeekExt + Unpin,
{
    file.seek(std::io::SeekFrom::Start(offset)).await?;

    let mut chunk_size = CHUNK_INITIAL;
    let mut last_sent = CHUNK_INITIAL;
    let mut total_read: u64 = 0;

    loop {
        let mut buf = BytesMut::with_capacity(chunk_size);
        let n = file.read_buf(&mut buf).await?;

        if n == 0 {
            break;
        }

        total_read += n as u64;

        // Grow chunk size proportionally to last send (per spec §3.4)
        chunk_size = ((last_sent as f64 * 1.25) as usize).clamp(CHUNK_INITIAL, CHUNK_MAX);
        last_sent = n;

        tx.send(buf.freeze())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))?;
    }

    Ok(total_read)
}

/// Open a file for upload reading at the given offset.
pub async fn open_for_upload(path: &Path, _offset: u64) -> std::io::Result<tokio::fs::File> {
    let file = OpenOptions::new().read(true).open(path).await?;
    Ok(file)
}

/// Create a bounded channel for disk-reader to socket-writer communication.
pub fn channel() -> (mpsc::Sender<Bytes>, mpsc::Receiver<Bytes>) {
    mpsc::channel(CHANNEL_CAPACITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(CHUNK_INITIAL, 4_096);
        assert_eq!(CHUNK_MAX, 512 * 1024);
        assert_eq!(CHANNEL_CAPACITY, 16);
    }
}
