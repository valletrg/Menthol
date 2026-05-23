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
    /// Room search per SEARCH_SYSTEM.md §2
    SearchRoom {
        room: String,
        query: String,
        token: u32,
    },
    /// User/buddy search per SEARCH_SYSTEM.md §2
    SearchUser {
        username: String,
        query: String,
        token: u32,
    },
    /// Wishlist search per SEARCH_SYSTEM.md §2
    SearchWishlist {
        wish_id: usize,
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
    /// Add a term to the wishlist
    AddWishlist(String),
    /// Remove a term from the wishlist by index
    RemoveWishlist(usize),
    /// Clear all allowed tokens (called on disconnect per spec §3.3)
    ClearSearchTokens,
}
