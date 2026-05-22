#[derive(Debug)]
pub enum Command {
    Connect {
        username: String,
        password: String,
    },
    Disconnect,
    Search {
        query: String,
        token: u32,
    },
    QueueDownload {
        username: String,
        filename: String,
        size: u64,
    },
    QueueUpload {
        username: String,
        filename: String,
    },
    CancelTransfer {
        id: u64,
    },
    PauseTransfers {
        direction: slsk_proto::types::TransferDirection,
    },
    ResumeTransfers {
        direction: slsk_proto::types::TransferDirection,
    },
    SetUploadSlots(u8),
    SetDownloadLimit(u64),
    SetUploadLimit(u64),
    JoinRoom(String),
    LeaveRoom(String),
    SendChatMessage {
        room: String,
        message: String,
    },
    SendPrivateMessage {
        username: String,
        message: String,
    },
}
