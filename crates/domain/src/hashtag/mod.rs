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
mod tests;
