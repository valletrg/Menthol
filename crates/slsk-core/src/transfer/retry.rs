//! Retry timers for transfer failures.
//!
//! Per spec §5:
//! - Connection failures: retry after 3 minutes
//! - Disk/IO failures: retry after 15 minutes
//! - F-connection wait timeout: 45 seconds from TransferResponse to connection arrival
//! - PlaceInQueue poll: every 5 minutes

use std::time::Duration;

/// Retry intervals for different failure types.
#[derive(Debug, Clone, Copy)]
pub enum RetryKind {
    Connection,
    Io,
    FConnectionWait,
    PlaceInQueuePoll,
}

impl RetryKind {
    pub fn duration(&self) -> Duration {
        match self {
            RetryKind::Connection => Duration::from_secs(3 * 60),
            RetryKind::Io => Duration::from_secs(15 * 60),
            RetryKind::FConnectionWait => Duration::from_secs(45),
            RetryKind::PlaceInQueuePoll => Duration::from_secs(5 * 60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_durations() {
        assert_eq!(RetryKind::Connection.duration().as_secs(), 180);
        assert_eq!(RetryKind::Io.duration().as_secs(), 900);
        assert_eq!(RetryKind::FConnectionWait.duration().as_secs(), 45);
        assert_eq!(RetryKind::PlaceInQueuePoll.duration().as_secs(), 300);
    }
}
