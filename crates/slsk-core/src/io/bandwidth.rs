//! Token bucket rate limiter for bandwidth control.
//!
//! Optionally limits download and upload speed across all concurrent transfers.
//! Nicotine+ behavior: split the limit evenly across active transfers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Token bucket for bandwidth limiting.
#[derive(Debug)]
pub struct TokenBucket {
    /// Available bytes in the current window
    tokens: AtomicU64,
    /// Limit in bytes per second (0 = unlimited)
    limit_bps: u64,
    /// Window start time
    window_start: std::sync::Mutex<Instant>,
}

impl TokenBucket {
    pub fn new(limit_bps: u64) -> Self {
        Self {
            tokens: AtomicU64::new(0),
            limit_bps,
            window_start: std::sync::Mutex::new(Instant::now()),
        }
    }

    /// Try to acquire `want` bytes from the bucket.
    /// Returns the number of bytes actually available (may be less than `want`).
    pub async fn acquire(&self, want: usize) -> usize {
        if self.limit_bps == 0 {
            return want; // unlimited
        }

        let mut deadline = *self.window_start.lock().unwrap() + Duration::from_secs(1);

        loop {
            let now = Instant::now();
            if now >= deadline {
                // Refill for new second
                *self.window_start.lock().unwrap() = now;
                self.tokens.store(self.limit_bps, Ordering::Relaxed);
                deadline = now + Duration::from_secs(1);
            }

            let available = self.tokens.load(Ordering::Relaxed) as usize;

            if available >= want {
                // Try to claim `want` bytes
                let new_val = self.tokens.fetch_sub(want as u64, Ordering::AcqRel);
                return (new_val as usize).min(want);
            }

            // Not enough, yield and retry after refill
            let sleep_time = deadline.saturating_duration_since(now);
            tokio::time::sleep(sleep_time).await;
        }
    }

    /// Split a global limit evenly across `n` active transfers.
    pub fn per_transfer_limit(global_limit: u64, n: usize) -> u64 {
        if n == 0 || global_limit == 0 {
            0
        } else {
            global_limit / n as u64
        }
    }
}

impl Clone for TokenBucket {
    fn clone(&self) -> Self {
        Self {
            tokens: AtomicU64::new(self.tokens.load(Ordering::Relaxed)),
            limit_bps: self.limit_bps,
            window_start: std::sync::Mutex::new(*self.window_start.lock().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_transfer_limit_splits_evenly() {
        assert_eq!(TokenBucket::per_transfer_limit(1_000_000, 4), 250_000);
        assert_eq!(TokenBucket::per_transfer_limit(1_000_000, 0), 0);
        assert_eq!(TokenBucket::per_transfer_limit(0, 4), 0);
    }

    #[tokio::test]
    async fn unlimited_bucket_returns_full_request() {
        let bucket = TokenBucket::new(0);
        let got = bucket.acquire(50_000).await;
        assert_eq!(got, 50_000);
    }
}
