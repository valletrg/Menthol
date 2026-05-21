#[derive(Debug)]
pub enum Command {
    Connect { username: String, password: String },
    Disconnect,
    Search { query: String, token: u32 },
    QueueDownload { username: String, filename: String, size: u64 },
}
