#[derive(Debug, Clone)]
pub enum Event {
    Connected {
        motd: String,
    },
    LoginFailed {
        reason: String,
    },
    Disconnected {
        reason: Option<String>,
    },
    SearchResult {
        token: u32,
        username: String,
    },
    TransferProgress {
        id: u64,
        bytes_done: u64,
        total: u64,
        direction: slsk_proto::types::TransferDirection,
    },
    TransferRequest {
        username: String,
        filename: String,
        size: u64,
        token: u32,
    },
    TransferComplete {
        id: u64,
        success: bool,
    },
    RoomJoined {
        room: String,
    },
    RoomLeft {
        room: String,
    },
    RoomMessage {
        room: String,
        username: String,
        message: String,
    },
    PrivateMessage {
        username: String,
        message: String,
        timestamp: u32,
    },
    UserStatusChanged {
        username: String,
        status: u32,
    },
    UploadFailed {
        username: String,
        filename: String,
    },
    UploadDenied {
        username: String,
        filename: String,
        reason: String,
    },
    PlaceInQueue {
        username: String,
        filename: String,
        position: u32,
    },
}
