//! Search term sanitization per SEARCH_SYSTEM.md §4.
//!
//! Handles special syntax (`-word`, `*word`, `"phrase"`), removed characters
//! stripping, and producing the transmitted vs. display terms.

use std::fmt;

/// Characters removed from the transmitted search term per spec §4.2.
/// These cause SoulseekQt to return no results if present in the wire format.
pub const REMOVED_SEARCH_CHARS: &[char] = &[
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=',
    '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
    // Unicode variants
    '\u{2013}', // –
    '\u{2014}', // —
    '\u{2010}', // ‐
    '\u{2018}', // '
    '\u{201C}', // "
    '\u{201D}', // "
    '\u{2026}', // …
];

/// Token types from search term parsing
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A normal word
    Word(String),
    /// An excluded word (prefixed with `-`)
    Excluded(String),
    /// A partial word match (prefixed with `*`)
    Partial(String),
    /// A phrase that must match exactly (quoted)
    Phrase(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Word(s) => write!(f, "{}", s),
            Token::Excluded(s) => write!(f, "-{}", s),
            Token::Partial(s) => write!(f, "*{}", s),
            Token::Phrase(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Simple shlex-like tokenizer that respects quoted phrases.
/// Does not handle escape sequences.
fn tokenize_search(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        if c == '"' {
            // Quoted phrase
            let mut phrase = String::new();
            while let Some(&c) = chars.peek() {
                if c == '"' {
                    let _ = chars.next();
                    break;
                }
                phrase.push(c);
                chars.next();
            }
            tokens.push(Token::Phrase(phrase));
        } else if c == '*' {
            // Partial word
            let mut word = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '*' || c == '"' || c == '-' {
                    break;
                }
                word.push(c);
                chars.next();
            }
            if !word.is_empty() {
                tokens.push(Token::Partial(word));
            }
        } else if c == '-' {
            // Check if it's an excluded word (not standalone)
            let mut word = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '*' || c == '"' || c == '-' {
                    break;
                }
                word.push(c);
                chars.next();
            }
            if !word.is_empty() {
                tokens.push(Token::Excluded(word));
            }
        } else {
            // Normal word
            let mut word = String::new();
            word.push(c);
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '*' || c == '"' || c == '-' {
                    break;
                }
                word.push(c);
                chars.next();
            }
            tokens.push(Token::Word(word));
        }
    }

    tokens
}

/// Strip all characters in `REMOVED_SEARCH_CHARS` from a string.
fn strip_removed_chars(s: &str) -> String {
    s.chars()
        .filter(|c| !REMOVED_SEARCH_CHARS.contains(c))
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitized search term with display vs. wire format separation.
/// See SEARCH_SYSTEM.md §4.3
#[derive(Debug, Clone, PartialEq)]
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

impl fmt::Display for SanitizedSearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SanitizedSearch {{ term: {}, transmitted: {} }}",
            self.term, self.term_transmitted
        )
    }
}

/// Sanitize a search term per the spec.
///
/// - `term`: original input string
/// - Returns `SanitizedSearch` with display and wire-format terms separated
pub fn sanitize_search_term(input: &str) -> SanitizedSearch {
    let input = input.trim().to_string();

    let mut included_words = Vec::new();
    let mut excluded_words = Vec::new();
    let mut transmitted_words = Vec::new();
    let mut term_sanitized_words = Vec::new();

    for token in tokenize_search(&input) {
        match &token {
            Token::Excluded(word) => {
                excluded_words.push(word.to_lowercase());
                // Excluded words are NOT transmitted
            }
            Token::Partial(word) => {
                included_words.push(word.to_lowercase());
                transmitted_words.push(format!("*{}", strip_removed_chars(word)));
                term_sanitized_words.push(format!("*{}", word));
            }
            Token::Phrase(phrase) => {
                included_words.push(phrase.to_lowercase());
                // Transmit each word in the phrase separately (stripped)
                for w in strip_removed_chars(phrase).split_whitespace() {
                    if !w.is_empty() {
                        transmitted_words.push(w.to_string());
                    }
                }
                term_sanitized_words.push(phrase.clone());
            }
            Token::Word(word) => {
                let stripped = strip_removed_chars(word);
                if !stripped.is_empty() {
                    for w in stripped.split_whitespace() {
                        included_words.push(w.to_lowercase());
                        transmitted_words.push(w.to_string());
                    }
                    // For term_sanitized, keep original word
                    term_sanitized_words.push(word.clone());
                }
            }
        }
    }

    let term_transmitted = transmitted_words.join(" ");
    let term_sanitized = term_sanitized_words.join(" ");

    SanitizedSearch {
        term: input,
        term_sanitized,
        term_transmitted,
        included_words,
        excluded_words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_removed_chars() {
        assert_eq!(strip_removed_chars("hello-world.mp3"), "helloworldmp3");
        assert_eq!(strip_removed_chars("test!file?"), "testfile");
        assert_eq!(strip_removed_chars("no-special"), "nospecial");
    }

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize_search("hello world");
        assert_eq!(
            tokens,
            vec![Token::Word("hello".into()), Token::Word("world".into())]
        );
    }

    #[test]
    fn test_tokenize_excluded() {
        let tokens = tokenize_search("flac -320kbps");
        assert_eq!(
            tokens,
            vec![
                Token::Word("flac".into()),
                Token::Excluded("320kbps".into())
            ]
        );
    }

    #[test]
    fn test_tokenize_partial() {
        let tokens = tokenize_search("*flo dark side");
        assert_eq!(
            tokens,
            vec![
                Token::Partial("flo".into()),
                Token::Word("dark".into()),
                Token::Word("side".into())
            ]
        );
    }

    #[test]
    fn test_tokenize_phrase() {
        let tokens = tokenize_search("\"dark side\" of the moon");
        assert_eq!(
            tokens,
            vec![
                Token::Phrase("dark side".into()),
                Token::Word("of".into()),
                Token::Word("the".into()),
                Token::Word("moon".into())
            ]
        );
    }

    #[test]
    fn test_sanitize_basic() {
        let s = sanitize_search_term("hello world");
        assert_eq!(s.term, "hello world");
        assert_eq!(s.term_transmitted, "hello world");
        assert!(s.included_words.contains(&"hello".into()));
        assert!(s.included_words.contains(&"world".into()));
        assert!(s.excluded_words.is_empty());
    }

    #[test]
    fn test_sanitize_excluded_stripped() {
        let s = sanitize_search_term("flac -320");
        assert_eq!(s.term_transmitted, "flac");
        assert!(s.excluded_words.contains(&"320".into()));
    }

    #[test]
    fn test_sanitize_special_chars() {
        // SoulseekQt returns no results if these chars are present
        let s = sanitize_search_term("test!file.mp3");
        assert_eq!(s.term_transmitted, "testfilemp3");
    }

    #[test]
    fn test_sanitize_phrase() {
        let s = sanitize_search_term("\"dark side\"");
        // Phrase words are split and transmitted separately
        assert_eq!(s.term_transmitted, "dark side");
        assert!(s.included_words.contains(&"dark side".into()));
    }

    #[test]
    fn test_sanitize_partial() {
        let s = sanitize_search_term("*flo");
        // Partial alone should still be transmitted with *
        assert_eq!(s.term_transmitted, "*flo");
    }

    #[test]
    fn test_sanitize_mixed() {
        let s = sanitize_search_term("pink floyd -mp3 *dark \"dark side\"");
        assert!(s.term_transmitted.contains("pink"));
        assert!(s.term_transmitted.contains("floyd"));
        assert!(s.term_transmitted.contains("*dark"));
        assert!(!s.term_transmitted.contains("mp3")); // excluded
        assert!(s.excluded_words.contains(&"mp3".into()));
    }
}
