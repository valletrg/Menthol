use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Disconnecting,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Authenticating => write!(f, "Authenticating"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Disconnecting => write!(f, "Disconnecting"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    UserRequested,
    ServerClosed,
    ConnectionFailed,
    LoginRejected,
    Timeout,
}

pub struct ServerStats {
    pub bytes_sent: AtomicU64,
    pub bytes_recv: AtomicU64,
    pub messages_sent: AtomicU64,
    pub messages_recv: AtomicU64,
}

impl ServerStats {
    pub const fn new() -> Self {
        Self {
            bytes_sent: AtomicU64::new(0),
            bytes_recv: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_recv: AtomicU64::new(0),
        }
    }

    pub fn reset(&self) {
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.bytes_recv.store(0, Ordering::Relaxed);
        self.messages_sent.store(0, Ordering::Relaxed);
        self.messages_recv.store(0, Ordering::Relaxed);
    }
}

impl Default for ServerStats {
    fn default() -> Self {
        Self::new()
    }
}
