//! Socket writer task for uploads.
//!
//! Receives chunks from DiskReader and writes to the socket.
//! Backpressure works in reverse: if the socket is slow, the channel fills
//! and DiskReader pauses.

use bytes::Bytes;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;

/// Write uploaded data to socket, sending all bytes.
pub async fn socket_writer_task<W>(
    mut writer: W,
    mut rx: mpsc::Receiver<Bytes>,
    file_size: u64,
) -> std::io::Result<u64>
where
    W: AsyncWrite + Unpin,
{
    let mut bytes_sent: u64 = 0;

    while let Some(chunk) = rx.recv().await {
        writer.write_all(&chunk).await?;
        bytes_sent += chunk.len() as u64;
        if bytes_sent >= file_size {
            break;
        }
    }

    Ok(bytes_sent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn socket_writer_tracks_bytes() {
        let (tx, rx) = mpsc::channel(4);

        // Write chunks then drop sender
        let _ = tx.send(bytes::Bytes::from_static(b"hello")).await;
        let _ = tx.send(bytes::Bytes::from_static(b" world")).await;
        drop(tx);

        // Use Cursor as writer
        let mut out = Cursor::new(Vec::new());
        let result = socket_writer_task(&mut out, rx, 11).await.unwrap();
        assert_eq!(result, 11);
        assert_eq!(out.into_inner(), b"hello world");
    }
}
