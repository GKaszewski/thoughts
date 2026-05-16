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
    let tags = extract("#a #b #a");
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].normalized, "a");
    assert_eq!(tags[1].normalized, "b");
}

#[test]
fn mid_word_extracted() {
    let tags = extract("text#tag");
    assert_eq!(names(&tags), ["tag"]);
}

#[test]
fn hash_only_ignored() {
    assert!(extract("# lone hash").is_empty());
}

#[test]
fn trailing_punctuation_excluded() {
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
