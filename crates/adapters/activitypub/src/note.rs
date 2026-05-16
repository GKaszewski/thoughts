use activitypub_base::NoteType;
use activitypub_base::AS_PUBLIC;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// AP Note representing a Thought.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtNote {
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub id: Url,
    pub url: Url, // Mastodon uses this as the clickable link
    pub attributed_to: Url,
    pub content: String,
    pub published: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<Url>,
    pub sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tag: Vec<serde_json::Value>,
}

impl ThoughtNote {
    #[allow(clippy::too_many_arguments)]
    pub fn new_public(
        id: Url,
        actor_url: Url,
        content: String,
        published: DateTime<Utc>,
        in_reply_to: Option<Url>,
        sensitive: bool,
        summary: Option<String>,
        followers_url: Url,
    ) -> Self {
        Self {
            kind: Default::default(),
            url: id.clone(),
            id,
            attributed_to: actor_url,
            content,
            published,
            to: vec![AS_PUBLIC.to_string()],
            cc: vec![followers_url.to_string()],
            in_reply_to,
            sensitive,
            summary,
            tag: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_serializes_with_public_audience() {
        let note = ThoughtNote::new_public(
            "https://example.com/thoughts/1".parse().unwrap(),
            "https://example.com/users/alice".parse().unwrap(),
            "Hello world".to_string(),
            chrono::Utc::now(),
            None,
            false,
            None,
            "https://example.com/users/alice/followers".parse().unwrap(),
        );
        let json = serde_json::to_string(&note).unwrap();
        assert!(json.contains(AS_PUBLIC));
        assert!(json.contains("Hello world"));
        assert!(json.contains("\"url\""));
    }
}
