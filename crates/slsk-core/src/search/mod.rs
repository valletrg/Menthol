//! Search subsystem — implements outgoing and incoming search handling per
//! SEARCH_SYSTEM.md specification.
//!
//! # Overview
//!
//! Search operates on two axes:
//! - **Outgoing (we search):** We send a query to the server. Matching peers
//!   find us via the distributed search network and respond with
//!   `FileSearchResponse` over a P connection.
//! - **Incoming (others search us):** We receive search queries from the server
//!   or distributed network. We look up matches in our local word index and
//!   send `FileSearchResponse` back.
//!
//! # Token Management
//!
//! Every search gets a unique `u32` token. Incoming `FileSearchResponse`
//! messages are zlib-compressed and checked against the allowed token set
//! before full decompression (the "allow/disallow gate").
//!
//! # Search Modes
//!
//! | Mode      | Server Message  | Code | Description                    |
//! |-----------|-----------------|------|--------------------------------|
//! | `global`  | `FileSearch`    | 26   | Entire distributed network     |
//! | `rooms`   | `RoomSearch`    | 120  | All users in a specific room    |
//! | `buddies` | `UserSearch`×N  | 42   | Each buddy individually        |
//! | `user`    | `UserSearch`    | 42   | A specific username            |
//! | `wishlist`| `WishlistSearch`| 103  | Same as global, rate-limited   |

pub mod search;
pub mod sanitize;
pub mod word_index;

pub use search::{SearchMode, SearchState};
pub use sanitize::SanitizedSearch;
pub use word_index::WordIndex;
