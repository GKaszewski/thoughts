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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub url: Option<Url>,
    pub attributed_to: Url,
    pub content: String,
    pub published: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<Url>,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tag: Vec<serde_json::Value>,
}

pub struct ThoughtNoteInput {
    pub id: Url,
    pub actor_url: Url,
    pub content: String,
    pub published: DateTime<Utc>,
    pub in_reply_to: Option<Url>,
    pub sensitive: bool,
    pub summary: Option<String>,
    pub followers_url: Url,
}

impl ThoughtNote {
    pub fn new_public(p: ThoughtNoteInput) -> Self {
        Self {
            kind: Default::default(),
            url: Some(p.id.clone()),
            id: p.id,
            attributed_to: p.actor_url,
            content: p.content,
            published: p.published,
            to: vec![AS_PUBLIC.to_string()],
            cc: vec![p.followers_url.to_string()],
            in_reply_to: p.in_reply_to,
            sensitive: p.sensitive,
            summary: p.summary,
            tag: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests;
