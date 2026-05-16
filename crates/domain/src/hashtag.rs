use std::collections::HashSet;

/// A hashtag extracted from content.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hashtag {
    /// Original casing, e.g. "Rust"
    pub raw: String,
    /// Lowercased, e.g. "rust" — used for DB lookups
    pub normalized: String,
    /// "tags/rust" — callers prepend base_url
    pub url_slug: String,
    /// "#rust" — used directly in AP tag array
    pub ap_name: String,
}

/// Extract hashtags from content using a char-by-char scan.
///
/// Rules:
/// - Tag starts after a bare `#` followed immediately by an alphanumeric char.
/// - Tag chars: `[A-Za-z0-9_]`.
/// - Deduplicated case-insensitively; first occurrence wins.
/// - Returned in order of first appearance.
pub fn extract(content: &str) -> Vec<Hashtag> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut tags: Vec<Hashtag> = Vec::new();
    let mut chars = content.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        if c == '#'
            && chars
                .peek()
                .map(|(_, nc)| nc.is_alphanumeric())
                .unwrap_or(false)
        {
            let raw: String = chars
                .by_ref()
                .take_while(|(_, nc)| nc.is_alphanumeric() || *nc == '_')
                .map(|(_, nc)| nc)
                .collect();

            if raw.is_empty() {
                continue;
            }

            let normalized = raw.to_lowercase();
            if seen.insert(normalized.clone()) {
                tags.push(Hashtag {
                    url_slug: format!("tags/{}", normalized),
                    ap_name: format!("#{}", normalized),
                    raw,
                    normalized,
                });
            }
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(tags: &[Hashtag]) -> Vec<&str> {
        tags.iter().map(|h| h.normalized.as_str()).collect()
    }

    #[test]
    fn basic() {
        let tags = extract("Hello #world and #Rust!");
        assert_eq!(names(&tags), ["world", "rust"]);
    }

    #[test]
    fn fields() {
        let tags = extract("#Rust");
        assert_eq!(tags.len(), 1);
        let h = &tags[0];
        assert_eq!(h.raw, "Rust");
        assert_eq!(h.normalized, "rust");
        assert_eq!(h.url_slug, "tags/rust");
        assert_eq!(h.ap_name, "#rust");
    }

    #[test]
    fn dedup_case_insensitive() {
        let tags = extract("#rust #Rust #RUST");
        assert_eq!(names(&tags), ["rust"]);
        assert_eq!(tags[0].raw, "rust"); // first occurrence wins
    }

    #[test]
    fn deduplicates_non_adjacent() {
        // The old algorithm used Vec::dedup() which only removes adjacent duplicates.
        // Using HashSet silently fixed this bug. This test documents the fix.
        let tags = extract("#a #b #a");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].normalized, "a");
        assert_eq!(tags[1].normalized, "b");
    }

    #[test]
    fn mid_word_extracted() {
        // `text#tag` — `#` not preceded by whitespace is still matched by the
        // char-by-char scan (the old algorithm didn't require whitespace before `#`).
        // This test documents the authoritative behaviour: mid-word tags ARE extracted.
        let tags = extract("text#tag");
        assert_eq!(names(&tags), ["tag"]);
    }

    #[test]
    fn hash_only_ignored() {
        assert!(extract("# lone hash").is_empty());
    }

    #[test]
    fn trailing_punctuation_excluded() {
        // punctuation after tag terminates the tag, not included
        let tags = extract("#rust.");
        assert_eq!(names(&tags), ["rust"]);
    }

    #[test]
    fn underscore_allowed() {
        let tags = extract("#hello_world");
        assert_eq!(names(&tags), ["hello_world"]);
    }

    #[test]
    fn empty_content() {
        assert!(extract("").is_empty());
    }

    #[test]
    fn order_of_appearance() {
        let tags = extract("#b #a #c");
        assert_eq!(names(&tags), ["b", "a", "c"]);
    }
}
