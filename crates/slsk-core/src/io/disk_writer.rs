//! Disk writer task for file downloads.
//!
//! Receives chunks from the SocketReader via channel and writes to disk.
//! Chunk is freed (Arc refcount decremented) immediately after write.

use bytes::Bytes;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;

/// Write downloaded file data to disk, with completion detection.
///
/// Continues until the channel is closed (sender dropped) or the expected
/// number of bytes has been written. Returns total bytes written.
pub async fn disk_writer_task<W>(
    mut file: W,
    mut rx: mpsc::Receiver<Bytes>,
    expected_size: u64,
    mut bytes_written: u64,
) -> std::io::Result<u64>
where
    W: AsyncWrite + Unpin,
{
    while let Some(chunk) = rx.recv().await {
        file.write_all(&chunk).await?;
        bytes_written += chunk.len() as u64;
        if bytes_written >= expected_size {
            break;
        }
    }
    Ok(bytes_written)
}

/// Open incomplete file for append (resume support).
/// The file offset at open time becomes our resume position.
pub async fn open_incomplete(path: &Path) -> std::io::Result<(tokio::fs::File, u64)> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
        .await?;

    let offset = file.metadata().await?.len();
    Ok((file, offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn disk_writer_reports_bytes_written() {
        let (tx, rx) = mpsc::channel(4);

        // Send chunks then drop sender
        let _ = tx.send(bytes::Bytes::from_static(b"hello")).await;
        let _ = tx.send(bytes::Bytes::from_static(b" world")).await;
        drop(tx);

        // Use Cursor which implements AsyncWrite
        let mut out = Cursor::new(Vec::new());
        let result = disk_writer_task(&mut out, rx, 11, 0).await.unwrap();
        assert_eq!(result, 11);
        assert_eq!(out.into_inner(), b"hello world");
    }
}
