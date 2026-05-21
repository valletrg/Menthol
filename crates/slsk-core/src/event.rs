#[derive(Debug)]
pub enum Event {
    Connected { motd: String },
    LoginFailed { reason: String },
    Disconnected { reason: Option<String> },
    SearchResult { token: u32, username: String },
    TransferProgress { id: u64, bytes_done: u64, total: u64 },
}
