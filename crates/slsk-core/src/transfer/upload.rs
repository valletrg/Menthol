//! Upload queue management with slot-based concurrency.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use slsk_proto::types::TransferDirection;

use super::{Transfer, TransferFailure, TransferId, TransferState};

/// A queued upload candidate
#[derive(Debug)]
pub struct QueuedUpload {
    pub id: TransferId,
    pub username: String,
    pub virtual_path: String,
    pub file_size: u64,
    pub token: u32,
    pub is_buddy: bool,
    pub queued_at: std::time::Instant,
}

/// Upload slot manager
#[derive(Debug)]
pub struct UploadQueue {
    /// Maximum concurrent uploads
    slots: usize,
    /// Currently active uploads (token -> Transfer)
    active: HashMap<u32, Transfer>,
    /// Queued uploads (FIFO, buddies promoted to front)
    queue: VecDeque<QueuedUpload>,
    /// Buddy usernames for queue prioritization
    buddies: std::collections::HashSet<String>,
}

impl UploadQueue {
    pub fn new(slots: usize) -> Self {
        Self {
            slots,
            active: HashMap::new(),
            queue: VecDeque::new(),
            buddies: std::collections::HashSet::new(),
        }
    }

    pub fn set_buddies(&mut self, buddies: impl IntoIterator<Item = String>) {
        self.buddies.clear();
        self.buddies.extend(buddies);
    }

    pub fn free_slots(&self) -> usize {
        self.slots.saturating_sub(self.active.len())
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Add an upload to the queue
    pub fn enqueue(&mut self, mut upload: QueuedUpload) {
        upload.is_buddy = self.buddies.contains(&upload.username);
        if upload.is_buddy {
            // Buddy uploads go to the front, just after other buddies
            let pos = self
                .queue
                .iter()
                .position(|u| !u.is_buddy)
                .unwrap_or(self.queue.len());
            let idx = self.queue.len().min(pos);
            let mut iter = self.queue.split_off(idx);
            self.queue.push_back(upload);
            self.queue.append(&mut iter);
        } else {
            self.queue.push_back(upload);
        }
    }

    /// Attempt to start the next upload if a slot is free
    pub fn try_start_next(&mut self) -> Option<QueuedUpload> {
        if self.free_slots() == 0 {
            return None;
        }
        self.queue.pop_front()
    }

    /// Record that an upload has started
    pub fn upload_started(&mut self, token: u32, transfer: Transfer) {
        self.active.insert(token, transfer);
    }

    /// Record that an upload has finished (success or failure)
    pub fn upload_finished(&mut self, token: u32) -> Option<Transfer> {
        self.active.remove(&token)
    }

    /// Get queue position for a given virtual path
    pub fn queue_position(&self, virtual_path: &str) -> Option<u32> {
        self.queue
            .iter()
            .position(|u| u.virtual_path == virtual_path)
            .map(|p| p as u32 + 1) // 1-based position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn make_upload(id: u64, username: &str) -> QueuedUpload {
        QueuedUpload {
            id: TransferId(id),
            username: username.into(),
            virtual_path: format!("/music/{}.mp3", id),
            file_size: 1_000_000,
            token: id as u32,
            is_buddy: false,
            queued_at: Instant::now(),
        }
    }

    #[test]
    fn upload_queue_buddy_priority() {
        let mut queue = UploadQueue::new(5);
        queue.set_buddies(["bob".to_string()]);

        queue.enqueue(make_upload(1, "alice")); // normal
        queue.enqueue(make_upload(2, "bob")); // buddy
        queue.enqueue(make_upload(3, "charlie")); // normal

        // Buddy should be first
        let next = queue.try_start_next().unwrap();
        assert_eq!(next.id, TransferId(2));
    }

    #[test]
    fn upload_queue_free_slots() {
        let queue = UploadQueue::new(3);
        assert_eq!(queue.free_slots(), 3);
    }

    #[test]
    fn queue_position() {
        let mut queue = UploadQueue::new(5);
        queue.enqueue(make_upload(1, "alice"));
        queue.enqueue(make_upload(2, "bob"));

        assert_eq!(queue.queue_position("/music/1.mp3"), Some(1));
        assert_eq!(queue.queue_position("/music/2.mp3"), Some(2));
        assert_eq!(queue.queue_position("/music/99.mp3"), None);
    }
}
