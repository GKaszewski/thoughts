use chrono::{DateTime, Utc};
use k_ap::NoteType;
use k_ap::AS_PUBLIC;
use serde::{Deserialize, Serialize};
use url::Url;

const STANDARD_NOTE_FIELDS: &[&str] = &[
    "type",
    "id",
    "attributedTo",
    "content",
    "published",
    "to",
    "cc",
    "inReplyTo",
    "sensitive",
    "summary",
    "tag",
    "url",
    "@context",
    "mediaType",
];

pub fn extract_extensions(obj: &serde_json::Value) -> Option<serde_json::Value> {
    let extensions: serde_json::Map<String, serde_json::Value> = obj
        .as_object()?
        .iter()
        .filter(|(k, _)| !STANDARD_NOTE_FIELDS.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    if extensions.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(extensions))
    }
}

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
    /// Returns `(note, extensions)` if `value` is a Note object, `None` otherwise.
    pub fn try_from_ap(mut value: serde_json::Value) -> Option<(Self, Option<serde_json::Value>)> {
        let obj_type = value.get("type").and_then(|v| v.as_str());
        if !matches!(obj_type, Some("Note" | "Article" | "Page")) {
            return None;
        }
        let extensions = extract_extensions(&value);
        if let Some(obj) = value.as_object_mut() {
            obj.insert("type".to_string(), serde_json::json!("Note"));
        }
        serde_json::from_value(value)
            .ok()
            .map(|note| (note, extensions))
    }

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
