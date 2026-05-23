//! Core search state, token management, and search modes per SEARCH_SYSTEM.md.
//!
//! # Token Management (§3)
//!
//! Tokens are `u32` values that identify searches. The allow/disallow gate
//! prevents unnecessary zlib decompression of stale `FileSearchResponse` messages.
//!
//! # Search Modes (§2)
//!
//! | Mode      | Server Message  | Code | Description                    |
//! |-----------|-----------------|------|--------------------------------|
//! | `global`  | `FileSearch`    | 26   | Entire distributed network     |
//! | `rooms`   | `RoomSearch`    | 120  | All users in a specific room    |
//! | `buddies` | `UserSearch`×N  | 42   | Each buddy individually        |
//! | `user`    | `UserSearch`    | 42   | A specific username            |
//! | `wishlist`| `WishlistSearch`| 103  | Same as global, rate-limited   |

use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};

/// Characters that must be excluded from search responses per server code 160.
#[derive(Debug, Clone, Default)]
pub struct ExcludedPhrases {
    phrases: Vec<String>,
}

impl ExcludedPhrases {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the excluded phrases list (received from server via code 160)
    pub fn set(&mut self, phrases: Vec<String>) {
        tracing::debug!("excluded search phrases updated: {:?}", &phrases);
        self.phrases = phrases;
    }

    /// Check if a file path contains any excluded phrase
    pub fn is_excluded(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();
        self.phrases
            .iter()
            .any(|p| path_lower.contains(&p.to_lowercase()))
    }

    pub fn is_empty(&self) -> bool {
        self.phrases.is_empty()
    }
}

/// A file search result as parsed from FileSearchResponse
#[derive(Debug, Clone)]
pub struct FileSearchResult {
    /// Virtual path (backslash-separated)
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Whether peer has a free upload slot right now
    pub free_upload_slots: bool,
    /// Peer's average upload speed in bytes/s
    pub upload_speed: u32,
    /// Number of files ahead of us in their queue
    pub queue_size: u32,
}

/// A search request issued by us
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub token: u32,
    pub mode: SearchMode,
    /// Sanitized search term (per SEARCH_SYSTEM.md §4)
    pub sanitized: super::SanitizedSearch,
}

/// Search mode per SEARCH_SYSTEM.md §2
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    /// Global distributed network search (code 26)
    Global,
    /// Room search (code 120)
    Room { room: String },
    /// User/buddy search (code 42)
    User { username: String },
    /// Wishlist search (code 103)
    Wishlist { wish_id: usize },
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::Global => write!(f, "global"),
            SearchMode::Room { room } => write!(f, "room:{room}"),
            SearchMode::User { username } => write!(f, "user:{username}"),
            SearchMode::Wishlist { wish_id } => write!(f, "wishlist:{wish_id}"),
        }
    }
}

/// A wishlist item per SEARCH_SYSTEM.md §9
#[derive(Debug, Clone)]
pub struct WishlistItem {
    /// The search term
    pub term: String,
    /// Whether auto-search is enabled for this item
    pub auto_search: bool,
    /// Whether this item is currently being searched (token live)
    pub is_ignored: bool,
    /// Custom filters (display only, not fully implemented)
    pub custom_filters: Vec<String>,
    /// Users to ignore for this wishlist search
    pub ignored_users: Vec<String>,
    /// When added (unix timestamp)
    pub time_added: u64,
}

impl WishlistItem {
    pub fn new(term: String) -> Self {
        Self {
            term,
            auto_search: true,
            is_ignored: true,
            custom_filters: Vec::new(),
            ignored_users: Vec::new(),
            time_added: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// The main search state manager
#[derive(Debug, Clone)]
pub struct SearchState {
    /// Token counter — monotonically increasing with wrap
    token_counter: u32,
    /// Allowed tokens: set of active search tokens awaiting responses.
    /// Used as the allow/disallow gate for FileSearchResponse decompression.
    allowed_tokens: HashSet<u32>,
    /// Active searches by token
    searches: HashMap<u32, SearchRequest>,
    /// Own tokens: for self-search detection (SEARCH_SYSTEM.md §5.2)
    own_tokens: HashSet<u32>,
    /// Search history (max 200 entries, per §10)
    search_history: Vec<String>,
    /// The wishlist (round-robin queue, per §9)
    wishlist: VecDeque<WishlistItem>,
    /// Wishlist polling interval in seconds (received from server as WishlistInterval)
    wishlist_interval_secs: u32,
    /// Excluded search phrases (server code 160, per §7.4)
    excluded_phrases: ExcludedPhrases,
    /// Maximum results to return per single-word search
    max_results_per_search: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    /// Create a new search state
    pub fn new() -> Self {
        Self {
            // Start with a random token to prevent collisions between sessions
            token_counter: rand::thread_rng().gen_range(0..=u32::MAX / 1000),
            allowed_tokens: HashSet::new(),
            searches: HashMap::new(),
            own_tokens: HashSet::new(),
            search_history: Vec::new(),
            wishlist: VecDeque::new(),
            wishlist_interval_secs: 600, // 10 minutes default until server sends WishlistInterval
            excluded_phrases: ExcludedPhrases::new(),
            max_results_per_search: 1000,
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Token management (§3)
    // ─────────────────────────────────────────────────────────────────

    /// Get the next token and increment the counter (with wrap)
    pub fn next_token(&mut self) -> u32 {
        if self.token_counter >= u32::MAX {
            self.token_counter = 0;
        } else {
            self.token_counter += 1;
        }
        self.token_counter
    }

    /// Add a token to the allowed set (open the gate when search starts)
    pub fn allow_token(&mut self, token: u32) {
        self.allowed_tokens.insert(token);
    }

    /// Remove a token from the allowed set (close the gate when search ends)
    pub fn disallow_token(&mut self, token: u32) {
        self.allowed_tokens.remove(&token);
        self.searches.remove(&token);
    }

    /// Check if a token is in the allowed set (the allow/disallow gate)
    pub fn is_token_allowed(&self, token: u32) -> bool {
        self.allowed_tokens.contains(&token)
    }

    /// Clear all allowed tokens (must be called on disconnect per spec §3.3)
    pub fn clear_allowed_tokens(&mut self) {
        self.allowed_tokens.clear();
    }

    /// Mark a token as "own" (for self-search detection)
    pub fn add_own_token(&mut self, token: u32) {
        self.own_tokens.insert(token);
    }

    /// Check if token is in own_tokens set, and if so remove and return true.
    /// Used for self-search detection (spec §5.2).
    pub fn check_and_remove_own_token(&mut self, token: u32) -> bool {
        self.own_tokens.remove(&token)
    }

    /// Get all allowed tokens (for lazy decompression stage 1)
    pub fn allowed_tokens(&self) -> &HashSet<u32> {
        &self.allowed_tokens
    }

    // ─────────────────────────────────────────────────────────────────
    // Search registration
    // ─────────────────────────────────────────────────────────────────

    /// Register a new search and return its token.
    /// Opens the allow gate for the token.
    pub fn start_search(&mut self, mode: SearchMode, sanitized: super::SanitizedSearch) -> u32 {
        let token = self.next_token();
        self.allow_token(token);
        self.searches.insert(
            token,
            SearchRequest {
                token,
                mode: mode.clone(),
                sanitized: sanitized.clone(),
            },
        );
        tracing::debug!("started search token={} mode={}", token, mode);
        token
    }

    /// Register a new search with an already-existing token (provided by GUI).
    /// Opens the allow gate for the token.
    pub fn start_search_with_token(&mut self, token: u32, mode: SearchMode, sanitized: super::SanitizedSearch) {
        self.allow_token(token);
        self.searches.insert(
            token,
            SearchRequest {
                token,
                mode: mode.clone(),
                sanitized: sanitized.clone(),
            },
        );
        tracing::debug!("started search with existing token={} mode={}", token, mode);
    }

    /// Get a search request by token
    pub fn get_search(&self, token: u32) -> Option<&SearchRequest> {
        self.searches.get(&token)
    }

    /// Remove a search and close its token gate
    pub fn remove_search(&mut self, token: u32) {
        self.disallow_token(token);
        tracing::debug!("removed search token={}", token);
    }

    /// Check if a token corresponds to a wishlist search
    pub fn is_wishlist_token(&self, token: u32) -> bool {
        matches!(
            self.searches.get(&token),
            Some(SearchRequest {
                mode: SearchMode::Wishlist { .. },
                ..
            })
        )
    }

    /// Mark a wishlist search as ignored but don't remove it (per spec §3.3)
    pub fn ignore_wishlist_search(&mut self, token: u32) {
        if let Some(req) = self.searches.get_mut(&token) {
            if let SearchMode::Wishlist { wish_id } = req.mode {
                if let Some(item) = self.wishlist.get_mut(wish_id) {
                    item.is_ignored = true;
                }
            }
        }
        self.allowed_tokens.remove(&token);
    }

    // ─────────────────────────────────────────────────────────────────
    // Search history (§10)
    // ─────────────────────────────────────────────────────────────────

    /// Add a term to search history (max 200 entries, dedup, move to front)
    pub fn add_to_history(&mut self, term: &str) {
        self.search_history.retain(|t| t != term);
        self.search_history.insert(0, term.to_string());
        self.search_history.truncate(200);
    }

    /// Get search history
    pub fn search_history(&self) -> &[String] {
        &self.search_history
    }

    /// Clear search history
    pub fn clear_history(&mut self) {
        self.search_history.clear();
    }

    // ─────────────────────────────────────────────────────────────────
    // Wishlist (§9)
    // ─────────────────────────────────────────────────────────────────

    /// Set the wishlist polling interval (from server WishlistInterval message)
    pub fn set_wishlist_interval(&mut self, secs: u32) {
        tracing::info!("wishlist interval set to {} seconds", secs);
        self.wishlist_interval_secs = secs;
    }

    /// Get the wishlist polling interval
    pub fn wishlist_interval(&self) -> u32 {
        self.wishlist_interval_secs
    }

    /// Add an item to the wishlist
    pub fn add_to_wishlist(&mut self, term: String) {
        // Avoid duplicates
        if self.wishlist.iter().any(|i| i.term == term) {
            return;
        }
        self.wishlist.push_back(WishlistItem::new(term));
        tracing::debug!("added to wishlist, total items: {}", self.wishlist.len());
    }

    /// Remove an item from the wishlist by index
    pub fn remove_from_wishlist(&mut self, idx: usize) {
        if idx < self.wishlist.len() {
            self.wishlist.remove(idx);
        }
    }

    /// Get the wishlist
    pub fn wishlist(&self) -> &VecDeque<WishlistItem> {
        &self.wishlist
    }

    /// Get the next wishlist item to search (round-robin).
    /// Returns the item index and sets it as not-ignored (active).
    pub fn next_wishlist_item(&mut self) -> Option<(usize, &mut WishlistItem)> {
        if self.wishlist.is_empty() {
            return None;
        }

        // Rotate: pop front, push back until we find an auto_search item
        let len = self.wishlist.len();
        for _ in 0..len {
            if let Some(mut item) = self.wishlist.pop_front() {
                if item.auto_search {
                    item.is_ignored = false;
                    let idx = self.wishlist.len();
                    self.wishlist.push_back(item);
                    return self.wishlist.get_mut(idx).map(|i| (idx, i));
                }
                self.wishlist.push_back(item);
            }
        }
        None
    }

    /// Get mutable wishlist item by index
    pub fn wishlist_item_mut(&mut self, idx: usize) -> Option<&mut WishlistItem> {
        self.wishlist.get_mut(idx)
    }

    // ─────────────────────────────────────────────────────────────────
    // Excluded phrases (§7.4)
    // ─────────────────────────────────────────────────────────────────

    /// Set excluded search phrases (from server code 160)
    pub fn set_excluded_phrases(&mut self, phrases: Vec<String>) {
        self.excluded_phrases.set(phrases);
    }

    /// Check if a file path should be excluded from search responses
    pub fn is_path_excluded(&self, path: &str) -> bool {
        self.excluded_phrases.is_excluded(path)
    }

    // ─────────────────────────────────────────────────────────────────
    // Configuration
    // ─────────────────────────────────────────────────────────────────

    /// Set max results per single-word search
    pub fn set_max_results(&mut self, max: usize) {
        self.max_results_per_search = max;
    }

    /// Get max results per search
    pub fn max_results(&self) -> usize {
        self.max_results_per_search
    }
}

/// Parse the SoulseekNS 2GB bug file size (spec §6.2).
///
/// SoulseekNS sends >2GiB files with garbage in the high 4 bytes (0xFFFFFFFF).
/// Detect this and only use the low 4 bytes.
#[inline]
pub fn parse_file_size_lo_hi(lo: u32, hi: u32) -> u64 {
    if hi == 0xFFFFFFFF {
        lo as u64 // SoulseekNS bug — discard hi
    } else {
        lo as u64 | ((hi as u64) << 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_wrap() {
        let mut state = SearchState::new();
        state.token_counter = u32::MAX - 1;
        let t1 = state.next_token();
        assert_eq!(t1, u32::MAX); // incremented from MAX-1 to MAX
        let t2 = state.next_token();
        assert_eq!(t2, 0); // wrapped from MAX to 0
    }

    #[test]
    fn test_token_allow_disallow() {
        let mut state = SearchState::new();
        let token = state.next_token();
        state.allow_token(token);
        assert!(state.is_token_allowed(token));
        state.disallow_token(token);
        assert!(!state.is_token_allowed(token));
    }

    #[test]
    fn test_search_history_dedup() {
        let mut state = SearchState::new();
        state.add_to_history("flac");
        state.add_to_history("mp3");
        state.add_to_history("flac"); // duplicate, moves to front
        assert_eq!(
            state.search_history(),
            &["flac".to_string(), "mp3".to_string()]
        );
    }

    #[test]
    fn test_search_history_max() {
        let mut state = SearchState::new();
        for i in 0..250 {
            state.add_to_history(&format!("term{}", i));
        }
        assert_eq!(state.search_history().len(), 200);
    }

    #[test]
    fn test_excluded_phrases() {
        let mut phrases = ExcludedPhrases::new();
        phrases.set(vec!["spam".into(), "advertisement".into()]);
        assert!(phrases.is_excluded("download this spam file"));
        assert!(phrases.is_excluded("ADVERTISEMENT"));
        assert!(!phrases.is_excluded("normal file"));
    }

    #[test]
    fn test_parse_file_size_soulseekns_bug() {
        // SoulseekNS bug: hi = 0xFFFFFFFF
        let size = parse_file_size_lo_hi(3_000_000_000u32, 0xFFFFFFFF);
        assert_eq!(size, 3_000_000_000u64);
    }

    #[test]
    fn test_parse_file_size_normal() {
        // Normal 64-bit size
        let lo = 0x12345678u32;
        let hi = 0x00000001u32; // = 0x12345678_00000000
        let size = parse_file_size_lo_hi(lo, hi);
        assert_eq!(size, 0x00000001_12345678u64);
    }
}
