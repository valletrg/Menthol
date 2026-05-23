//! Word index for responding to incoming searches per SEARCH_SYSTEM.md §7.
//!
//! Maps lowercase words to file indices for efficient search result lookup.
//! Used when we receive a search query from another user and need to find
//! matching files in our shared inventory.

use std::collections::{HashMap, HashSet};

/// Maximum number of results to return for a single-word search.
/// Prevents huge allocations when a common word matches millions of files.
const MAX_SINGLE_WORD_RESULTS: usize = 1000;

/// A file entry in the word index
#[derive(Debug, Clone)]
pub struct IndexedFile {
    /// Virtual path as presented to searchers (backslash-separated)
    pub path: String,
    /// Size in bytes
    pub size: u64,
    /// File extension (lowercase, without dot)
    pub extension: String,
}

impl IndexedFile {
    /// Create from a file path
    pub fn new(path: String, size: u64) -> Self {
        let extension = std::path::Path::new(&path)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        Self {
            path,
            size,
            extension,
        }
    }
}

/// Word index that maps words to file indices.
#[derive(Debug, Clone, Default)]
pub struct WordIndex {
    /// Maps word -> set of file indices that contain that word
    index: HashMap<String, HashSet<usize>>,
    /// All indexed files by index
    files: HashMap<usize, IndexedFile>,
    /// Next available file index
    next_index: usize,
}

impl WordIndex {
    /// Create a new empty word index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file to the index. The path is lowercased and split into words
    /// (on non-alphanumeric boundaries), then each word maps to this file's index.
    pub fn add_file(&mut self, path: String, size: u64) {
        let idx = self.next_index;
        self.next_index += 1;

        let file = IndexedFile::new(path, size);
        let words = Self::extract_words(&file.path);

        for word in words {
            self.index.entry(word).or_default().insert(idx);
        }

        self.files.insert(idx, file);
    }

    /// Clear the entire index
    pub fn clear(&mut self) {
        self.index.clear();
        self.files.clear();
        self.next_index = 0;
    }

    /// Extract lowercase words from a path, splitting on non-alphanumeric chars.
    fn extract_words(path: &str) -> Vec<String> {
        path.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    /// Search the index for files matching the given terms.
    ///
    /// - `included`: words that MUST appear in results (AND logic)
    /// - `excluded`: words that must NOT appear (subtracted from results)
    /// - `partial`: words that must match as suffix (e.g. `*flo` matches `floyd`)
    /// - `max_results`: cap on results for single-word searches
    ///
    /// Returns indices of matching files, or None if no matches.
    ///
    /// # Algorithm (per spec §7.3)
    ///
    /// 1. All included words must exist in the index
    /// 2. Start with first included word's file set
    /// 3. Intersect with remaining included words
    /// 4. Apply partial word matches (suffix matching)
    /// 5. Subtract excluded words
    pub fn search(
        &self,
        included: &[String],
        excluded: &[String],
        partial: &[String],
        max_results: usize,
    ) -> Option<HashSet<usize>> {
        // At least one included word is required (per spec §7.3)
        // A search for *ello alone returns nothing - partial requires at least one included word
        if included.is_empty() && !partial.is_empty() {
            return None;
        }

        if included.is_empty() && partial.is_empty() {
            return None;
        }

        // Start with the first included word's result set
        let start_word = included.first().or(partial.first())?;
        let start_results: HashSet<usize> = if included.len() + partial.len() == 1 {
            // Single-word search: cap at max_results to avoid huge allocations
            self.index
                .get(start_word)
                .map(|s| s.iter().copied().take(max_results).collect())
                .unwrap_or_default()
        } else {
            self.index.get(start_word).cloned().unwrap_or_default()
        };

        let mut results = start_results;

        // Intersect with remaining included words
        for word in included.iter().skip(1) {
            if let Some(word_results) = self.index.get(word) {
                results.retain(|idx| word_results.contains(idx));
            } else {
                results.clear();
            }
            if results.is_empty() {
                return None;
            }
        }

        // Partial words (*ello): find all index words starting with the partial word
        for partial_word in partial {
            // Partial matching: words that START with the partial word
            let partial_results: HashSet<usize> = self
                .index
                .iter()
                .filter(|(k, _)| k.starts_with(partial_word.as_str()))
                .flat_map(|(_, v)| v.iter().copied())
                .collect();

            if partial_results.is_empty() {
                // No files found matching this partial word
                return None;
            }

            if results.is_empty() {
                // First partial match - initialize results
                results = partial_results;
            } else {
                // Intersect with existing results (from included words or previous partial)
                results.retain(|idx| partial_results.contains(idx));
            }
        }

        // Subtract excluded words
        for word in excluded {
            if let Some(excluded_results) = self.index.get(word) {
                for i in excluded_results {
                    results.remove(i);
                }
            }
            if results.is_empty() {
                return None;
            }
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    /// Get a file by its index
    pub fn get(&self, idx: usize) -> Option<&IndexedFile> {
        self.files.get(&idx)
    }

    /// Get all files matching given indices
    pub fn get_many(&self, indices: &HashSet<usize>) -> Vec<&IndexedFile> {
        indices.iter().filter_map(|&i| self.files.get(&i)).collect()
    }

    /// Iterate over all words in the index (for debugging)
    pub fn all_words(&self) -> impl Iterator<Item = &String> {
        self.index.keys()
    }

    /// Total number of files indexed
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_search() {
        let mut index = WordIndex::new();
        index.add_file("pink floyd dark side of the moon.flac".into(), 100_000_000);
        index.add_file("pink floyd the wall.mp3".into(), 50_000_000);
        index.add_file("led zeppelin IV.flac".into(), 200_000_000);

        // Basic search - single word
        let results = index.search(&["pink".into()], &[], &[], 1000);
        assert!(results.is_some());
        assert_eq!(results.unwrap().len(), 2);

        // Multi-word AND
        let results = index.search(&["pink".into(), "floyd".into()], &[], &[], 1000);
        assert!(results.is_some());
        assert_eq!(results.unwrap().len(), 2);

        // Exclude
        let results = index.search(&["pink".into()], &["floyd".into()], &[], 1000);
        assert!(results.is_none());

        // Partial alone (no included words) returns nothing per spec §7.3
        // "At least one complete included word is required. A search for *ello alone returns nothing"
        let results = index.search(&[], &[], &["flo".into()], 1000);
        assert!(
            results.is_none(),
            "partial-only search should return None per spec §7.3"
        );

        // Partial WITH a complete word works
        let results = index.search(&["pink".into()], &[], &["flo".into()], 1000);
        assert!(results.is_some());
        assert_eq!(results.unwrap().len(), 2); // pink floyd files
    }

    #[test]
    fn test_extract_words() {
        // Words should be lowercase, split on non-alphanumeric
        let words = WordIndex::extract_words("Hello-World.mp3");
        assert!(words.contains(&"hello".into()));
        assert!(words.contains(&"world".into()));
        assert!(words.contains(&"mp3".into()));
    }
}
