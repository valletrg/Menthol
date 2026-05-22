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
    /// Incoming search result from a peer (via server relay).
    /// Note: `filename` and `size` require peer-level response handling (code 9).
    /// This event is emitted when the server relays a peer's FileSearchResponse.
    SearchResult {
        token: u32,
        username: String,
        filename: String,
        size: u64,
    },
    /// Our search was acknowledged and is now active.
    SearchStarted {
        token: u32,
        query: String,
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
