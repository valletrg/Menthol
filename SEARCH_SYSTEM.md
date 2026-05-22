# Menthol — Search System Specification

Full specification of how search works in Soulseek: outgoing queries, incoming
results, responding to others' queries, the wishlist, and the word index.

Reference: `pynicotine/search.py`, `pynicotine/slskmessages.py`

---

## 1. Overview

Search in Soulseek operates on two distinct axes:

**Outgoing (we search):** We send a query to the server or directly to peers.
Matching peers find us via the distributed search network and respond directly
with `FileSearchResponse` over a P connection.

**Incoming (others search us):** We receive search queries from the server
(relayed) or from the distributed network. We look up matches in our local
word index and send `FileSearchResponse` directly back to the searcher.

Both axes share infrastructure but are otherwise independent. A client with
sharing disabled still receives and displays search results. A client with no
active search token still must respond to incoming queries.

---

## 2. Search Modes

There are five outgoing search modes:

| Mode | Server Message | Code | Who receives it |
|------|---------------|------|-----------------|
| `global` | `FileSearch` | 26 | Entire distributed network |
| `rooms` | `RoomSearch` | 120 | All users in a specific room |
| `buddies` | `UserSearch` × N | 42 | Each buddy individually |
| `user` | `UserSearch` | 42 | A specific username |
| `wishlist` | `WishlistSearch` | 103 | Same as global, rate-limited |

`WishlistSearch` is a subclass of `FileSearch` — identical wire format,
different server code. The server applies rate-limiting to wishlist searches
that it does not apply to regular `FileSearch`.

---

## 3. Token Management

### 3.1 Generation

Every search gets a unique `u32` token. The token namespace is shared across
searches, transfers, and connection requests — they all draw from the same
incrementing counter.

```rust
// Starting token: random value in [0, UINT32_MAX / 1000]
// This prevents collisions between sessions.
fn initial_token() -> u32 {
    rand::thread_rng().gen_range(0..=u32::MAX / 1000)
}

// Increment with wraparound
fn increment_token(token: u32) -> u32 {
    if token >= u32::MAX { 0 } else { token + 1 }
}
```

### 3.2 The Allow/Disallow Gate

This is a critical performance mechanism. Incoming `FileSearchResponse` messages
are **zlib-compressed** and can be large. Before doing any decompression work,
the network thread checks the token against a set of "allowed" tokens.

When you start a search: add the token to the allowed set.
When you close/remove a search: remove the token from the allowed set.

```rust
// In the network layer, before decompressing FileSearchResponse:
fn should_parse_search_response(token: u32, allowed: &HashSet<u32>) -> bool {
    allowed.contains(&token)
}
```

Without this gate, every incoming `FileSearchResponse` from every peer (even
from searches you've closed) would be fully decompressed. On an active
Soulseek session this would waste significant CPU.

The token is extracted **before** decompression — the wire format is:

```
[zlib compressed payload]:
  [ u32 username_len ][ username bytes ]
  [ u32 token ]       ← extract this first, then decide whether to continue
  ...rest of message
```

Nicotine+ decompresses just enough bytes to read the token, then either
decompresses the rest or discards.

### 3.3 Token Lifecycle

```
do_search()
  → increment token
  → add_allowed_token(token)   ← gate opened
  → send FileSearch to server

[results arrive as FileSearchResponse]

remove_search(token)
  → remove_allowed_token(token) ← gate closed
  → if wishlist: mark is_ignored=true, keep token in searches map
  → else: remove from searches map entirely
```

Wishlist tokens are never fully removed from `searches` — they stay alive
so the periodic wishlist search can reuse them. They are just gated by
`is_ignored = true`.

---

## 4. Search Term Sanitization

Before a search term is transmitted, it goes through two transformations:
**sanitization** (for local filtering) and **transmission cleaning** (for the
wire). These produce different strings that serve different purposes.

### 4.1 Special Syntax

The search term parser supports three special prefixes:

| Prefix | Example | Meaning |
|--------|---------|---------|
| `-word` | `-live` | Exclude results containing this word |
| `*word` | `*ello` | Partial match — result must contain a word ending in `ello` |
| `"phrase"` | `"dark side"` | Phrase match — result must contain this exact phrase |

These are parsed with `shlex` tokenization so quoted phrases work correctly.

### 4.2 Removed Characters

A large set of punctuation and special characters is **stripped** from the
transmitted search term. This is because SoulseekQt returns no results if the
search term contains these characters:

```
! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~ – — ‐ ' " " …
```

They are replaced with spaces. The original (unsanitized) term is kept for
display purposes. The cleaned term is what goes on the wire.

### 4.3 Rust Implementation

```rust
const REMOVED_SEARCH_CHARS: &[char] = &[
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|',
    '}', '~',
    // Unicode
    '\u{2013}', // –
    '\u{2014}', // —
    '\u{2010}', // ‐
    '\u{2018}', // '
    '\u{201C}', // "
    '\u{201D}', // "
    '\u{2026}', // …
];

pub struct SanitizedSearch {
    /// Original term as entered by user (for display)
    pub term: String,
    /// Cleaned term for display in history (punctuation stripped)
    pub term_sanitized: String,
    /// What actually goes on the wire
    pub term_transmitted: String,
    /// Words that must appear in results (for local filtering)
    pub included_words: Vec<String>,
    /// Words that must NOT appear in results (for local filtering)
    pub excluded_words: Vec<String>,
}

pub fn sanitize_search_term(input: &str) -> SanitizedSearch {
    let input = input.trim();
    let mut included_words = Vec::new();
    let mut excluded_words = Vec::new();
    let mut transmitted_words = Vec::new();

    // Tokenize respecting quoted phrases
    for token in tokenize_search(input) {
        match token {
            Token::Excluded(word) => {
                excluded_words.push(word.to_lowercase());
                // Excluded words are not transmitted
            }
            Token::Partial(word) => {
                included_words.push(word.to_lowercase());
                transmitted_words.push(format!("*{}", word));
            }
            Token::Phrase(phrase) => {
                included_words.push(phrase.to_lowercase());
                // Transmit each word in the phrase separately (stripped)
                for w in strip_removed_chars(&phrase).split_whitespace() {
                    transmitted_words.push(w.to_string());
                }
            }
            Token::Word(word) => {
                let stripped = strip_removed_chars(&word);
                if !stripped.is_empty() {
                    for w in stripped.split_whitespace() {
                        included_words.push(w.to_lowercase());
                        transmitted_words.push(w.to_string());
                    }
                }
            }
        }
    }

    SanitizedSearch {
        term: input.to_string(),
        term_sanitized: transmitted_words.join(" "),
        term_transmitted: transmitted_words.join(" "),
        included_words,
        excluded_words,
    }
}
```

---

## 5. Outgoing Wire Formats

### 5.1 FileSearch (Server Code 26)

```
[ u32 token ]
[ u32 term_len ][ term bytes ]
```

Note: standalone `-` words are stripped from the transmitted term.
`"hello -world"` is transmitted as `"hello"`. The `-world` exclusion filter
is applied locally on results.

### 5.2 UserSearch (Server Code 42)

```
[ u32 username_len ][ username bytes ]
[ u32 token ]
[ u32 term_len ][ term bytes ]
```

For buddies search, send one `UserSearch` per buddy. All use the same token.

For self-search (searching your own username), add the token to `own_tokens`
set — this lets you receive and respond to your own search, which is normally
suppressed.

### 5.3 RoomSearch (Server Code 120)

```
[ u32 room_len ][ room bytes ]
[ u32 token ]
[ u32 term_len ][ term bytes ]
```

Standalone `-` words stripped from term, same as `FileSearch`.

### 5.4 WishlistSearch (Server Code 103)

Identical wire format to `FileSearch` (code 26). Different code only.

---

## 6. Incoming Results — FileSearchResponse (Peer Code 9)

### 6.1 Wire Format

The entire payload is **zlib compressed**. Decompress with `flate2` or
the `zlib` crate. Max uncompressed size: 128 MiB.

**Decompressed payload:**

```
[ u32 username_len ][ username bytes ]
[ u32 token ]
[ u32 num_files ]
  × num_files:
    [ u8  code ]            ← always 1
    [ u32 name_len ][ name bytes ]   ← virtual path, forward slashes normalized to backslash
    [ u64 size ]            ← see SoulseekNS 2GB bug below
    [ u32 ext_len ]         ← OBSOLETE, always 0, skip this many bytes
    [ u32 num_attrs ]
      × num_attrs:
        [ u32 attr_type ]
        [ u32 attr_value ]
[ u8  free_upload_slots ]  ← bool: peer has a free upload slot right now
[ u32 upload_speed ]       ← peer's average upload speed in B/s
[ u32 queue_size ]         ← number of files ahead of us in their queue

[ u32 unknown ]            ← optional, read if bytes remain
[ u32 num_private_files ]  ← optional private share results
  × num_private_files: (same structure as public files)
```

### 6.2 The SoulseekNS 2GB Bug (File Size)

When reading the `u64` file size, inspect byte 8 (index 7, 0-based):

```rust
fn read_file_size(buf: &mut impl Buf) -> u64 {
    // SoulseekNS sends >2GiB files with garbage in the high 4 bytes
    // (value = 0xFFFFFFFF). Detect and read only the low 4 bytes.
    let lo = buf.get_u32_le();
    let hi = buf.get_u32_le();

    if hi == 0xFFFFFFFF {
        lo as u64  // SoulseekNS bug — discard hi
    } else {
        lo as u64 | ((hi as u64) << 32)
    }
}
```

### 6.3 File Attributes

Attributes are key-value pairs where the key is a `u32` type code:

| Code | Meaning | Notes |
|------|---------|-------|
| 0 | Bitrate (kbps) | Lossy audio |
| 1 | Duration (seconds) | All audio |
| 2 | VBR flag | 1 = variable bitrate, 0 = CBR |
| 3 | Encoder | Obsolete, ignore |
| 4 | Sample rate (Hz) | Lossless audio |
| 5 | Bit depth | Lossless audio |

Attributes are optional and may not be present. A file with no attributes is
valid. Lossless files (FLAC, WAV) send sample rate + bit depth instead of
bitrate + VBR.

**Inferring missing values:**

If bitrate is missing but sample rate and bit depth are present:
```
bitrate_kbps = (sample_rate * bit_depth * 2) / 1000
```

If duration is missing but bitrate is present:
```
duration_secs = file_size_bytes / (bitrate_kbps * 125)
```

### 6.4 Lazy Decompression (Performance)

Nicotine+ decompresses in two stages to avoid unnecessary work:

**Stage 1:** Decompress just enough to read `username_len + username + token`.
Then check if `token` is in the allowed set. If not, discard the compressed
blob entirely — no further decompression.

**Stage 2:** If allowed, decompress the remaining payload.

In Rust with `flate2`:

```rust
use flate2::read::ZlibDecoder;
use std::io::Read;

fn parse_search_response(
    compressed: &[u8],
    allowed_tokens: &HashSet<u32>,
) -> Option<FileSearchResponse> {

    let mut decoder = ZlibDecoder::new(compressed);

    // Stage 1: read username + token only
    let username_len = read_u32_le(&mut decoder)?;
    let username = read_string(&mut decoder, username_len as usize)?;
    let token = read_u32_le(&mut decoder)?;

    if !allowed_tokens.contains(&token) {
        return None; // discard, no further decompression
    }

    // Stage 2: read the rest
    let num_files = read_u32_le(&mut decoder)?;
    let files = (0..num_files)
        .map(|_| parse_file_entry(&mut decoder))
        .collect::<Option<Vec<_>>>()?;

    let free_slots = read_u8(&mut decoder)? != 0;
    let upload_speed = read_u32_le(&mut decoder)?;
    let queue_size = read_u32_le(&mut decoder)?;

    // Optional fields
    // ...

    Some(FileSearchResponse { username, token, files, free_slots, upload_speed, queue_size })
}
```

### 6.5 Result Filtering (Local)

After receiving a result, apply local filters before emitting to the UI.
Nicotine+ filters at result receipt time:

```rust
fn should_accept_result(
    result: &FileSearchResponse,
    search: &SearchRequest,
    network_filter: &NetworkFilter,
) -> bool {

    // 1. User is on ignore list
    if network_filter.is_user_ignored(&result.username) {
        return false;
    }

    // 2. User's IP is on the IP ignore list
    if network_filter.is_ip_ignored(&result.username, &result.addr) {
        return false;
    }

    // 3. Wishlist: user is in the wish's ignored_users set
    if let SearchMode::Wishlist { ignored_users, .. } = &search.mode {
        if ignored_users.contains(&result.username) {
            return false;
        }
    }

    true
}
```

Advanced UI filters (bitrate, size, free slot, country, file type) are applied
at the display layer, not at receipt. Received results are stored and re-filtered
when filter settings change.

---

## 7. Responding to Others' Searches

When we receive a search query (from server relay or distributed network), we
look it up in our word index and respond if we have matches.

### 7.1 Entry Points

Search queries arrive via two paths:

**From server** (`FileSearch`, server code 26, inbound):
```
[ u32 username_len ][ username bytes ]   ← who is searching
[ u32 token ]
[ u32 term_len ][ term bytes ]
```
This is the same message code as the outbound FileSearch, but when **received**
it contains the searcher's username. The server relays these for `UserSearch`
and `RoomSearch` requests.

**From distributed network** (Distrib code 3, `DistribSearch`):
```
[ u32 unknown ]
[ u32 username_len ][ username bytes ]
[ u32 token ]
[ u32 term_len ][ term bytes ]
```

Both paths call the same `_process_search_request(term, username, token)`.

### 7.2 Guard Conditions

Before touching the word index, reject early if:

```rust
fn should_process_incoming_search(
    term: &str,
    username: &str,
    token: u32,
    config: &Config,
    own_username: &str,
    own_tokens: &mut HashSet<u32>,
) -> bool {

    // 1. Empty term
    if term.is_empty() { return false; }

    // 2. Search responses disabled in settings
    if !config.search_results_enabled { return false; }

    // 3. Pending shutdown (finishing uploads before quit)
    if config.pending_shutdown { return false; }

    // 4. Query from ourselves
    if username == own_username {
        // Exception: if we sent a user-search targeting ourselves,
        // the token is in own_tokens — allow it and remove from set
        if own_tokens.remove(&token) {
            return true;
        }
        return false;
    }

    // 5. Max results set to 0
    if config.max_search_results == 0 { return false; }

    // 6. Term too short
    if term.chars().count() < config.min_search_chars { return false; }

    // 7. User is banned
    if is_banned(username) { return false; }

    true
}
```

### 7.3 Word Index Lookup

The word index maps lowercase words to lists of file indices:

```
HashMap<String, Vec<usize>>
```

The lookup algorithm:

```rust
fn search_word_index(
    term: &str,
    included: &HashSet<String>,
    excluded: &HashSet<String>,
    partial: &HashSet<String>,
    max_results: usize,
    word_index: &HashMap<String, Vec<usize>>,
) -> Option<HashSet<usize>> {

    // All included words must exist in the index
    for word in included.iter() {
        if !word_index.contains_key(word) {
            return None;
        }
    }

    let start_word = included.iter().next()?;
    let is_single_word = included.len() + excluded.len() + partial.len() == 1;

    // Start with the indices for the first included word
    let start_results = word_index.get(start_word)?;
    let start_results: Vec<usize> = if is_single_word {
        // Single-word search: cap at max_results to avoid huge allocations
        // (e.g. "flac" could match millions of files)
        start_results.iter().copied().take(max_results).collect()
    } else {
        start_results.to_vec()
    };

    let mut results: HashSet<usize> = start_results.into_iter().collect();

    // Intersect with remaining included words
    for word in included.iter().skip(1) {
        let word_results: HashSet<usize> = word_index.get(word)
            .map(|v| v.iter().copied().collect())
            .unwrap_or_default();
        results = results.intersection(&word_results).copied().collect();
        if results.is_empty() { return None; }
    }

    // Partial words (*ello): find all index words ending in the partial word
    for partial_word in partial.iter() {
        let partial_results: HashSet<usize> = word_index.iter()
            .filter(|(k, _)| k.ends_with(partial_word.as_str()))
            .flat_map(|(_, v)| v.iter().copied())
            .filter(|i| results.contains(i))
            .collect();

        if partial_results.is_empty() { return None; }
        results = partial_results;
    }

    // Subtract excluded words
    for word in excluded.iter() {
        if let Some(excluded_results) = word_index.get(word) {
            for i in excluded_results {
                results.remove(i);
            }
        }
        if results.is_empty() { return None; }
    }

    if results.is_empty() { None } else { Some(results) }
}
```

**At least one complete included word is required.** A search for `*ello`
alone returns nothing — you must have at least one non-partial, non-excluded
word. This matches official client behavior.

### 7.4 Excluded Search Phrases (Server Code 160)

The server sends a list of phrases that must be excluded from all search
responses. Store this list and check every result file path against it before
including it in a response:

```rust
fn is_file_path_excluded(path: &str, excluded_phrases: &[String]) -> bool {
    let path_lower = path.to_lowercase();
    excluded_phrases.iter().any(|phrase| path_lower.contains(phrase.as_str()))
}
```

This is received once after login via `ExcludedSearchPhrases` (server code 160)
and cleared on disconnect. Log it at debug level — it changes occasionally.

### 7.5 FileSearchResponse Construction

```rust
core.send_to_peer(username, FileSearchResponse {
    search_username: own_username.to_string(),
    token,
    files:          public_fileinfos,       // public share results
    private_files:  private_fileinfos,      // buddy/trusted results (may be empty)
    free_slots:     uploads.has_free_slot(),
    upload_speed:   uploads.average_speed(),
    queue_size:     uploads.queue_size_for(username),
})
```

`private_files` is only included if non-empty. Buddies see their results in
`files` (not `private_files`). Non-buddies see buddy files in `private_files`
if `reveal_buddy_shares` is enabled — they appear grayed out in Nicotine+'s UI
but cannot be downloaded.

---

## 8. Search Result Data Model

Each result is a tuple:

```rust
pub struct SearchResult {
    pub code:     u8,          // always 1
    pub path:     String,      // virtual path, backslash separated
    pub size:     u64,         // bytes
    pub attrs:    FileAttrs,
}

pub struct FileAttrs {
    pub bitrate:     Option<u32>,   // kbps
    pub duration:    Option<u32>,   // seconds
    pub vbr:         Option<bool>,
    pub sample_rate: Option<u32>,   // Hz
    pub bit_depth:   Option<u32>,   // bits
}
```

A complete search result as seen by the UI combines the above with peer metadata:

```rust
pub struct SearchResultEntry {
    pub username:    String,
    pub file:        SearchResult,
    pub free_slots:  bool,
    pub speed:       u32,        // B/s
    pub queue_size:  u32,
    pub country:     Option<String>,  // from GetUserStats, populated asynchronously
}
```

Country code is not in the search response. It comes from `GetUserStats`
(server code 36) which you request asynchronously after receiving results.
Nicotine+ fires a background `GetUserStats` for each unique username in
results to populate the country column.

---

## 9. Wishlist

### 9.1 Mechanics

The wishlist is a persistent list of search terms that are searched
automatically on a server-defined interval.

On login, the server sends `WishlistInterval` (code 104) with the interval
in seconds. Nicotine+ observes 10–15 minutes in practice. Start a repeating
timer at this interval that calls `_do_next_wishlist_search()`.

**Only one wishlist item is searched per interval.** The wishlist is a
round-robin queue: each tick, the next item is moved from the front to the
back and searched. This spreads load across the server's rate limit.

```rust
fn do_next_wishlist_search(wishlist: &mut VecDeque<WishlistItem>, ...) {
    // Rotate to next auto-search item
    let mut n = 0;
    while n < wishlist.len() {
        let item = wishlist.pop_front().unwrap();
        let is_target = item.auto_search;
        wishlist.push_back(item);
        n += 1;
        if is_target { break; }
    }
    // The item is now at the back; search it
    if let Some(item) = wishlist.back_mut() {
        item.is_ignored = false;
        send_wishlist_search(item);
    }
}
```

### 9.2 Wishlist Persistence

Saved as JSON to `wishlist.json` in the data directory. Written every 3
minutes and on quit. Format:

```json
[
  {
    "term": "pink floyd dark side",
    "auto_search": true,
    "filter_mode": "custom",
    "time_added": 1700000000,
    "custom_filters": ["", "", "10", "320", false, "", "mp3", "", false],
    "ignored_users": ["spammer123"]
  }
]
```

`custom_filters` is a fixed-length array with positional fields:
`[include, exclude, min_size_mb, min_bitrate, free_slot_only, country, file_type, min_length, public_only]`

### 9.3 Wishlist Item States

| `is_ignored` | `auto_search` | Meaning |
|-------------|--------------|---------|
| `true` | `true` | Active wishlist item, not currently searching |
| `false` | `true` | Currently searching (token is live) |
| `true` | `false` | Disabled wishlist item (user toggled off) |

When the user opens a wishlist item's result tab, `is_ignored` is set to
`false` and the tab receives live results for the current search interval.

---

## 10. Search History

- Max 200 entries, stored in config
- Uses `term_sanitized` (not `term_transmitted`) as the key
- Duplicates are moved to the front, not duplicated
- Written to config on each new search

```rust
fn update_search_history(history: &mut Vec<String>, term: &str, limit: usize) {
    history.retain(|t| t != term);  // remove duplicate
    history.insert(0, term.to_string());
    history.truncate(limit);
}
```

---

## 11. Message Reference

| Message | Direction | Code | Notes |
|---------|-----------|------|-------|
| `FileSearch` | → Server | 26 | Global search outgoing |
| `FileSearch` | ← Server | 26 | Relayed search from another user (inbound) |
| `UserSearch` | → Server | 42 | User/buddy search |
| `RoomSearch` | → Server | 120 | Room search |
| `WishlistSearch` | → Server | 103 | Wishlist search (same format as FileSearch) |
| `WishlistInterval` | ← Server | 104 | Sets wishlist polling interval |
| `ExcludedSearchPhrases` | ← Server | 160 | Phrases to exclude from responses |
| `FileSearchResponse` | ↔ Peer | 9 | Results, zlib-compressed |
| `FileSearchRequest` | ↔ Peer | 8 | **OBSOLETE** — ignore on receive, never send |
| `FolderContentsRequest` | → Peer | 36 | Request folder listing |
| `FolderContentsResponse` | ← Peer | 37 | Folder listing, zlib-compressed |
| `SharedFileListRequest` | → Peer | 4 | Request full share browse |
| `SharedFileListResponse` | ← Peer | 5 | Full share browse, zlib-compressed |
| `DistribSearch` | ← Distrib | 3 | Search query from distributed network |

---

## 12. Common Implementation Bugs

**Not gating FileSearchResponse on the allowed token set.** Without the
allow/disallow mechanism, every incoming compressed response is fully
decompressed. Soulseek is a busy network — this will peg a CPU core on an
active session.

**Transmitting excluded words.** `-word` must be stripped from the transmitted
term. Only transmit included words. The exclusion is a local filter.

**Using term_transmitted for history.** Search history should store
`term_sanitized` (display-friendly), not `term_transmitted` (wire format).
They differ when the user types excluded or partial words.

**Not stripping lone `-` tokens.** The character `-` alone (not `-word`) must
be stripped from the transmitted term. `FileSearch` and `RoomSearch` do this
in their constructors; make sure sanitization handles it.

**Searching immediately on wishlist load.** Wishlist searches should only fire
on the interval timer, not immediately on load. The interval is not known until
the server sends `WishlistInterval` after login.

**Not clearing allowed tokens on disconnect.** The allowed set must be cleared
on server disconnect. Token values can collide between sessions (token
generation is random but bounded). A stale token from a previous session could
match a new unrelated response.

**Forgetting to request GetUserStats for country codes.** Country is not in
`FileSearchResponse`. Without requesting `GetUserStats` asynchronously per
unique result username, the country column will always be empty.

**Single-word partial searches.** A search of only `*word` with no full words
must return no results. The word index requires at least one complete word as
the anchor before partial or exclusion filtering applies.
