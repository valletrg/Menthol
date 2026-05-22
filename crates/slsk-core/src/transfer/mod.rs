//! Transfer state machine and core types.

use std::time::Instant;

use slsk_proto::types::TransferDirection;

/// Transfer identifier (unique within the transfers subsystem)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferId(pub u64);

/// State machine per transfer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferState {
    Queued,
    GettingStatus,
    Connecting,
    Handshaking,
    Transferring,
    Finished,
    Failed(TransferFailure),
    Paused { offset: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferFailure {
    ConnectionTimeout,
    ConnectionRefused,
    ConnectionClosed,
    LocalFileError,
    RemoteError,
    UserCancelled,
}

/// A single file transfer
#[derive(Debug)]
pub struct Transfer {
    pub id: TransferId,
    pub direction: TransferDirection,
    pub username: String,
    pub virtual_path: String,
    pub file_size: u64,
    pub token: u32,
    pub state: TransferState,
    pub bytes_transferred: u64,
    pub offset: u64,
    pub queued_at: Instant,
    pub started_at: Option<Instant>,
}

impl Transfer {
    pub fn new(
        id: TransferId,
        direction: TransferDirection,
        username: String,
        virtual_path: String,
        file_size: u64,
        token: u32,
    ) -> Self {
        Self {
            id,
            direction,
            username,
            virtual_path,
            file_size,
            token,
            state: TransferState::Queued,
            bytes_transferred: 0,
            offset: 0,
            queued_at: Instant::now(),
            started_at: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        (self.bytes_transferred + self.offset) >= self.file_size
    }

    pub fn bytes_remaining(&self) -> u64 {
        self.file_size
            .saturating_sub(self.bytes_transferred + self.offset)
    }
}

/// Tracks download-specific state including 2GB bug handling
#[derive(Debug)]
pub struct DownloadState {
    pub expected_size: u64,
    pub bytes_received: u64,
    pub resume_offset: u64,
    pub cached_size: Option<u64>,
}

impl DownloadState {
    pub fn new(file_size: u64, resume_offset: u64) -> Self {
        Self {
            expected_size: file_size,
            bytes_received: 0,
            resume_offset,
            cached_size: None,
        }
    }

    /// Apply 2GB bug fix: if remote sends size=0 and we have a cached size, use it
    pub fn apply_size(&mut self, file_size: u64) {
        if file_size == 0 && self.cached_size.is_some_and(|s| s > 0) {
            self.expected_size = self.cached_size.unwrap();
        } else {
            self.expected_size = file_size;
        }
    }

    pub fn is_complete(&self) -> bool {
        (self.bytes_received + self.resume_offset) >= self.expected_size
    }

    pub fn bytes_remaining(&self) -> u64 {
        self.expected_size
            .saturating_sub(self.bytes_received + self.resume_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_complete_detection() {
        let mut state = DownloadState::new(1_000_000, 0);
        assert!(!state.is_complete());
        state.bytes_received = 1_000_000;
        assert!(state.is_complete());
    }

    #[test]
    fn download_2gb_bug() {
        let mut state = DownloadState::new(0, 0);
        state.cached_size = Some(3_000_000_000);
        state.apply_size(0);
        assert_eq!(state.expected_size, 3_000_000_000);
    }

    #[test]
    fn transfer_bytes_remaining() {
        let transfer = Transfer::new(
            TransferId(1),
            TransferDirection::Download,
            "alice".into(),
            "/music/song.mp3".into(),
            1_000_000,
            123,
        );
        assert_eq!(transfer.bytes_remaining(), 1_000_000);
    }
}
