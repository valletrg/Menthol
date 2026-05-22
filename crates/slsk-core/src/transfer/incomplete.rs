//! Incomplete file management for resumable downloads.
//!
//! Downloads are written to a temporary incomplete file during transfer.
//! On successful completion, the file is moved atomically to its final location.

use md5::{Digest, Md5};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::task::JoinError;

/// In-progress incomplete file tracker (fcntl equivalent).
/// Tracks which incomplete paths are currently open to prevent collisions.
#[derive(Default)]
pub struct IncompleteTracker {
    in_progress: HashSet<PathBuf>,
}

impl IncompleteTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a path is already in use.
    pub fn is_in_use(&self, path: &Path) -> bool {
        self.in_progress.contains(path)
    }

    /// Mark a path as in use.
    pub fn mark_open(&mut self, path: PathBuf) -> bool {
        self.in_progress.insert(path)
    }

    /// Mark a path as closed.
    pub fn mark_closed(&mut self, path: &Path) {
        self.in_progress.remove(path);
    }

    pub fn len(&self) -> usize {
        self.in_progress.len()
    }
}

/// Generate the incomplete file path for a given virtual path and username.
/// Uses MD5 of (virtual_path + username) to create a collision-safe name.
pub fn incomplete_path(incomplete_dir: &Path, username: &str, virtual_path: &str) -> PathBuf {
    let mut hasher = Md5::new();
    hasher.update(virtual_path.as_bytes());
    hasher.update(username.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let basename = virtual_path
        .rsplit('\\')
        .next()
        .or_else(|| virtual_path.rsplit('/').next())
        .unwrap_or("download");

    incomplete_dir.join(format!("INCOMPLETE{hash}{basename}"))
}

/// Move completed incomplete file to final destination atomically.
///
/// Falls back to copy+delete if cross-mount (atomic rename only works
/// within the same filesystem).
pub async fn complete_file(
    incomplete: &Path,
    final_path: &Path,
) -> std::io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Try atomic rename first (same filesystem)
    match fs::rename(incomplete, final_path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices
            || e.raw_os_error() == Some(libc::EXDEV) => {
            // Cross-mount: copy + delete
            fs::copy(incomplete, final_path).await?;
            fs::remove_file(incomplete).await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Delete an incomplete file (on cancel or error).
pub async fn discard_incomplete(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_path_format() {
        let path = incomplete_path(
            Path::new("/tmp/incomplete"),
            "alice",
            "/music/song.mp3",
        );
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("INCOMPLETE"));
        assert!(name.ends_with("song.mp3"));
        assert_eq!(name.len(), 32 + 4 + 8); // hash(32) + prefix(9) + basename(8)
    }

    #[test]
    fn incomplete_path_collision() {
        // Same user + path = same hash
        let p1 = incomplete_path(Path::new("/tmp"), "alice", "/song.mp3");
        let p2 = incomplete_path(Path::new("/tmp"), "alice", "/song.mp3");
        assert_eq!(p1, p2);

        // Different user or path = different hash
        let p3 = incomplete_path(Path::new("/tmp"), "bob", "/song.mp3");
        assert_ne!(p1, p3);
    }
}
