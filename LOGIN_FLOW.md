# Menthol — Login & Connection Flow Specification

This document covers the full connection and login lifecycle: from initial TCP
connect through to a fully authenticated, ready-to-use session. Includes the
message sequence, hash construction, post-login burst, reconnect behavior, and
the `Relogged` edge case.

Reference: `pynicotine/slskproto.py`, `pynicotine/slskmessages.py`,
`pynicotine/core.py`, and all subsystem `_server_login` handlers.

---

## 1. Server Connection

The Soulseek server address is `server.slsknet.org:2242`. There is no TLS.

### 1.1 TCP Keepalive

Immediately after the socket is established, before any bytes are sent,
configure TCP keepalive on the server socket:

```rust
use socket2::SockRef;

fn configure_server_keepalive(stream: &TcpStream) -> io::Result<()> {
    let sock = SockRef::from(stream);
    sock.set_keepalive(true)?;

    // After 10s idle, send keepalive probes
    sock.set_tcp_keepalive(&socket2::TcpKeepalive::new()
        .with_time(Duration::from_secs(10))
        .with_interval(Duration::from_secs(2))
        .with_retries(10)
    )?;
    // Total timeout: 10s idle + 10 probes * 2s interval = 30s before dead conn detected
    Ok(())
}
```

Nicotine+ sets idle=10, interval=2, count=10. This means a dead connection
is detected within 30 seconds regardless of application-level traffic. Without
this, a silent network failure would leave the connection in a zombie state
indefinitely.

### 1.2 Connection State

Before sending `Login`, the connection is in state `Connecting`. No other
outgoing messages are permitted until `Login` is sent. Nicotine+ enforces this:

```python
def _is_outgoing_server_message_permitted(self, msg):
    return self._server_address is not None or msg.__class__ is Login
```

Menthol should do the same — gate the outbound message queue on auth state.

```rust
pub enum ServerConnState {
    Disconnected,
    Connecting,           // TCP established, Login not yet sent
    LoggingIn,            // Login sent, awaiting response
    Connected,            // Login succeeded, session active
}
```

Only `Login` may be sent in `Connecting`. All other outbound messages queue
until `Connected`.

---

## 2. The Login Message (Server Code 1)

### 2.1 Wire Format (Outbound)

```
[ u32 length ][ u32 code=1 ][ payload ]

payload:
  [ u32 username_len ][ username bytes ]
  [ u32 password_len ][ password bytes ]
  [ u32 major_version ]
  [ u32 hash_len ][ hash bytes ]   ← MD5 hex digest
  [ u32 minor_version ]
```

### 2.2 The MD5 Hash — Critical Detail

The hash field is `MD5(username + password)` encoded as a lowercase hex string.

**This is not the password hash.** It is a combined hash of both fields
concatenated as a UTF-8 string with no separator.

```rust
use md5::{Md5, Digest};

fn login_hash(username: &str, password: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(username.as_bytes());
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

Example: username `"user"`, password `"pass"` →
MD5 of `"userpass"` → `"67b6d6f59...(32 hex chars)"`

The server echoes this hash back in the success response as a checksum. You
can verify it matches but Nicotine+ discards it (`_checksum = self.unpack_string()`).

### 2.3 Version Numbers

| Field | Menthol value | Notes |
|-------|--------------|-------|
| `major_version` | `182` | Pick a unique value — don't use 157 (NS/Qt) or 160 (Nicotine+) or 170 (slskd) |
| `minor_version` | `1` | Start at 1, increment on protocol-breaking changes |

Using a unique major version is important. The server uses it to differentiate
clients. Impersonating another client's version can cause the server to apply
that client's specific rules to your session.

### 2.4 SetWaitPort (Server Code 2)

Send this **immediately after** `Login`, before the response arrives. Nicotine+
sends both in the same write cycle:

```python
self._send_message_to_server(Login(...))
self._send_message_to_server(SetWaitPort(self._listen_port))
```

```
payload: [ u32 port ]
```

This tells the server which port we are listening on for incoming peer
connections. Default Soulseek port is 2234, but any port works. If the port
is 0, peers cannot initiate connections to us (we can still initiate outward).

Menthol should open the listen socket **before** connecting to the server, so
the port is known when `SetWaitPort` is constructed.

Do not send the obfuscated port variant. SoulseekQt sends obfuscation type +
obfuscated port alongside the plain port. Nicotine+ explicitly skips this and
there is no evidence ISP traffic shaping is a real problem on Soulseek today.

---

## 3. Login Response (Server Code 1, Inbound)

### 3.1 Wire Format

```
[ u32 length ][ u32 code=1 ][ payload ]

payload (success=true):
  [ u8 success=1 ]
  [ u32 banner_len ][ banner bytes ]   ← MOTD string
  [ u32 own_ip ]                       ← our external IP as seen by server (big-endian)
  [ u32 hash_len ][ hash bytes ]       ← echo of MD5 hash we sent (discard)
  [ u8 is_supporter ]                  ← 1 if account has privileges

payload (success=false):
  [ u8 success=0 ]
  [ u32 reason_len ][ reason bytes ]   ← rejection reason string
  [ u32 detail_len ][ detail bytes ]   ← optional, only for INVALIDUSERNAME
```

### 3.2 Rejection Reasons

| String | Meaning |
|--------|---------|
| `"INVALIDUSERNAME"` | Username contains disallowed characters or is too long. A detail string follows with the specific rule violated. |
| `"EMPTYPASSWORD"` | Password field was empty. |
| `"INVALIDPASS"` | Password is wrong. |
| `"INVALIDVERSION"` | Major version is not recognized. (Rare — server whitelist.) |
| `"SVRFULL"` | Server is at capacity. Retry after backoff. |
| `"SVRPRIVATE"` | Server is in private mode (maintenance). Retry later. |

`"INVALIDUSERNAME"` is the only rejection that includes a detail string.
Read it only when the reason matches. Reading it unconditionally will corrupt
the buffer on other rejection types.

```rust
pub enum LoginResult {
    Success {
        banner:       String,
        own_ip:       Ipv4Addr,
        is_supporter: bool,
    },
    Failure {
        reason: LoginRejectReason,
        detail: Option<String>,  // only populated for InvalidUsername
    },
}

pub enum LoginRejectReason {
    InvalidUsername,
    EmptyPassword,
    InvalidPassword,
    InvalidVersion,
    ServerFull,
    ServerPrivate,
    Unknown(String),
}
```

---

## 4. Post-Login Burst

On successful login, several messages are sent immediately and several
subsystems re-initialize. This all happens before the UI reports "connected".

### 4.1 Distributed Network Init (from `slskproto.py`)

This is triggered directly in the Login response handler, not via the event
system:

```
→ HaveNoParent(true)          code 71  — tell server we need a parent
→ BranchRoot(username)        code 93  — our branch root is ourselves
→ BranchLevel(0)              code 94  — we are at level 0
→ AcceptChildren(false)       code 100 — not accepting children yet
```

These four messages must go out together as a unit immediately after login
success. They register us in the distributed search network. Without them the
server will not send `PossibleParents` and we won't participate in search
propagation.

### 4.2 Shares Count (from `shares.py`)

```
→ SharedFoldersFiles(num_folders, num_files)    code 35
```

Report how many folders and files we are sharing. Send 0/0 if the share index
has not been built yet. The server displays this count on our profile. It does
not affect functionality.

### 4.3 Chatrooms (from `chatrooms.py`)

```
→ RoomList()                       code 64  — request full room list
→ PrivateRoomToggle(enabled)       code 141 — enable/disable room invitations
→ JoinRoom(name) × N               code 14  — rejoin each previously joined room
```

The server automatically sends a limited room list on login (excluding rooms
with few users and blacklisted rooms). `RoomList()` requests the complete
unfiltered list.

For the global room: `JoinGlobalRoom()` (code 57) instead of `JoinRoom`.

### 4.4 Private Chat (from `privatechat.py`)

```
→ WatchUser(username) × N         code 5   — re-watch all users with open chats
```

Re-subscribes to status notifications for all users who have open private
message conversations. This restores their online/away/offline status
indicators.

### 4.5 Interests (from `interests.py`)

```
→ AddThingILike(item) × N         code 51
→ AddThingIHate(item) × N         code 52
```

Re-sends all liked and disliked tags from config. The server does not persist
these between sessions — they must be re-declared on every login.

Each item is lowercased and stripped before sending. Skip items that are not
strings or are empty after stripping.

### 4.6 Transfers (from `transfers.py`)

```
→ WatchUser(username) × N         code 5
```

Re-watches all users who have failed transfers, to detect when they come back
online for retry. Also reapplies bandwidth limits.

### 4.7 User Info (from `userinfo.py`)

```
→ WatchUser(username) × N         code 5
```

Re-watches all users whose info pages are open.

### 4.8 Complete Post-Login Sequence (Ordered)

The order that Nicotine+ sends these is determined by event subscription order.
For Menthol, use this canonical order:

```
1.  HaveNoParent(true)
2.  BranchRoot(own_username)
3.  BranchLevel(0)
4.  AcceptChildren(false)
5.  SetWaitPort(listen_port)          ← already sent before response, but confirm
6.  SharedFoldersFiles(folders, files)
7.  RoomList()
8.  PrivateRoomToggle(enabled)
9.  JoinRoom(name) × N  (or JoinGlobalRoom())
10. AddThingILike(item) × N
11. AddThingIHate(item) × N
12. WatchUser(username) × N  (all: chat users, transfer users, info users)
```

Items 1–4 are highest priority — they must go out before anything else touches
the distributed network state.

---

## 5. Reconnect Behavior

On any unexpected disconnect (network error, server close, timeout), Menthol
reconnects automatically using exponential backoff with jitter.

### 5.1 Backoff Schedule (from `slskproto.py`)

```rust
fn next_reconnect_delay(current: Option<Duration>) -> Duration {
    match current {
        // First attempt: random jitter to spread load if server went down
        None => Duration::from_secs(rand::thread_rng().gen_range(5..=15)),
        // Subsequent: exponential backoff, capped at 5 minutes
        Some(d) => (d * 2).min(Duration::from_secs(300)),
    }
}
```

Timeline example:
- Disconnect → wait 5–15s (random) → attempt 1
- Fail → wait 10–30s → attempt 2
- Fail → wait 20–60s → attempt 3
- ...capped at 300s between attempts

### 5.2 Manual Reconnect

If the user triggers a manual reconnect (disconnect + connect button), use a
fixed 5s delay instead of the backoff schedule. Reset the backoff counter on
successful login.

### 5.3 On Disconnect — Subsystem Teardown

When the server connection closes, clean up state in this order:

```rust
// transfers: abort all active, mark queued as offline
// chatrooms: clear all room user lists, ticker lists
// privatechat: clear message queue, away-message tracking
// userinfo: clear request timestamps
// users: clear all cached addresses and statuses
// distributed: close parent connection, close child connections
// network: close all peer connections
```

Do not persist runtime state across disconnects. Re-fetch everything fresh on
the next login.

---

## 6. The Relogged Edge Case (Server Code 41)

`Relogged` is sent by the server when another client logs into the same
account. The current session is immediately terminated by the server.

```rust
// payload: empty
pub struct Relogged;
```

**Handling:**

1. Receive `Relogged` → set a `relogged` flag, do not attempt reconnect
2. The server will close the TCP connection immediately after
3. On disconnect event: if `relogged` flag is set, display a specific message
   to the user: "Someone else logged into your account"
4. Do not auto-reconnect — reconnecting would immediately kick out whoever
   just logged in, creating a loop

```rust
pub enum DisconnectReason {
    Manual,
    Relogged,
    NetworkError(io::Error),
    ServerClosed,
}
```

The `Relogged` message arrives before the TCP close, so you have time to set
the flag before the disconnect handler fires.

---

## 7. Rust Implementation Structure

### 7.1 State Machine

```rust
pub enum ConnectionState {
    Disconnected { reason: Option<DisconnectReason> },
    Connecting,
    LoggingIn,
    Connected {
        username:     String,
        own_ip:       Ipv4Addr,
        is_supporter: bool,
        banner:       String,
    },
}
```

Transitions:
```
Disconnected → Connecting          (user initiates connect)
Connecting   → LoggingIn           (TCP established, Login + SetWaitPort sent)
LoggingIn    → Connected           (LoginResponse success received)
LoggingIn    → Disconnected        (LoginResponse failure received)
Connected    → Disconnected        (network error, server close, or Relogged)
```

### 7.2 Login Task

```rust
async fn run_login(
    stream: TcpStream,
    config: &Config,
    evt_tx: &mpsc::Sender<Event>,
) -> Result<(), CoreError> {

    configure_server_keepalive(&stream)?;

    let (reader, mut writer) = stream.into_split();

    // Build and send Login + SetWaitPort immediately
    let hash = login_hash(&config.username, &config.password);
    let login_msg = LoginRequest {
        username:      config.username.clone(),
        password:      config.password.clone(),
        major_version: CLIENT_MAJOR_VERSION,   // 182
        minor_version: CLIENT_MINOR_VERSION,   // 1
        hash,
    };
    let wait_port_msg = SetWaitPort { port: config.listen_port };

    send_server_message(&mut writer, &login_msg).await?;
    send_server_message(&mut writer, &wait_port_msg).await?;

    // Read exactly one response message — must be Login (code 1)
    let response = read_server_message(&mut reader).await?;

    match response {
        ServerMessage::Login(LoginResult::Success { banner, own_ip, is_supporter }) => {
            evt_tx.send(Event::Connected { banner, own_ip, is_supporter }).await?;
            // Proceed to post-login burst and main loop
            send_post_login_burst(&mut writer, &config).await?;
            Ok(())
        }
        ServerMessage::Login(LoginResult::Failure { reason, detail }) => {
            evt_tx.send(Event::LoginFailed { reason, detail }).await?;
            Err(CoreError::LoginRejected)
        }
        other => {
            // Server sent something unexpected before Login response
            Err(CoreError::UnexpectedMessage(other))
        }
    }
}
```

### 7.3 Post-Login Burst

```rust
async fn send_post_login_burst(
    writer: &mut OwnedWriteHalf,
    config: &Config,
    state: &AppState,
) -> Result<(), CoreError> {

    // 1-4: Distributed network
    send_server_message(writer, &HaveNoParent { have_no_parent: true }).await?;
    send_server_message(writer, &BranchRoot { username: config.username.clone() }).await?;
    send_server_message(writer, &BranchLevel { level: 0 }).await?;
    send_server_message(writer, &AcceptChildren { accept: false }).await?;

    // 5: Share counts
    let (folders, files) = state.shares.counts();
    send_server_message(writer, &SharedFoldersFiles { folders, files }).await?;

    // 6-8: Rooms
    send_server_message(writer, &RoomList {}).await?;
    send_server_message(writer, &PrivateRoomToggle {
        enable: config.private_chatrooms_enabled
    }).await?;
    for room in &state.chatrooms.joined_rooms {
        send_server_message(writer, &JoinRoom { room: room.clone() }).await?;
    }

    // 9-10: Interests
    for item in &config.liked_interests {
        send_server_message(writer, &AddThingILike { item: item.clone() }).await?;
    }
    for item in &config.disliked_interests {
        send_server_message(writer, &AddThingIHate { item: item.clone() }).await?;
    }

    // 11: Watch users
    for username in state.all_watched_users() {
        send_server_message(writer, &WatchUser { username: username.clone() }).await?;
    }

    Ok(())
}
```

---

## 8. Common Implementation Bugs

These are bugs that are easy to make and hard to debug because they don't
produce immediate errors — they just silently break things.

**Wrong hash input.** The MD5 is over `username + password` as a concatenated
UTF-8 string, not just the password, and not in any other encoding. Using only
the password or using UTF-16 will produce a wrong hash that the server accepts
anyway (it echoes it back but does not validate it), leading to confusion.

**Sending SetWaitPort late.** If you wait for the `Login` response before
sending `SetWaitPort`, the server may attempt to broker a peer connection to
you before it knows your port. Send both in the same write burst.

**Not sending HaveNoParent.** Without this, the server never sends
`PossibleParents` and you never join the distributed search network. Searches
may appear to work (you still receive results via direct connections) but you
will miss many results and be a bad network citizen.

**Reconnecting on Relogged.** Auto-reconnect after `Relogged` creates a
kick-loop between two clients. Must be a non-reconnecting disconnect.

**Reading detail string unconditionally on failure.** Only `INVALIDUSERNAME`
is followed by a detail string. Attempting to read it for `INVALIDPASS` etc.
will consume the next message's bytes into the detail field, corrupting the
read buffer for all subsequent messages.

**Not resetting WatchUser on reconnect.** WatchUser subscriptions do not
persist on the server between sessions. Re-send for all tracked users in the
post-login burst or user status indicators will stay stale.
