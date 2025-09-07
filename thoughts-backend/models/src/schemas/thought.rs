use crate::domains::{
    thought::{self, Visibility},
    user,
};
use common::DateTimeWithTimeZoneWrapper;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema, FromQueryResult, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtSchema {
    pub id: Uuid,
    #[schema(example = "frutiger")]
    pub author_username: String,
    #[schema(example = "This is my first thought! #welcome")]
    pub content: String,
    pub visibility: Visibility,
    pub reply_to_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZoneWrapper,
}

impl ThoughtSchema {
    pub fn from_models(thought: &thought::Model, author: &user::Model) -> Self {
        Self {
            id: thought.id,
            author_username: author.username.clone(),
            content: thought.content.clone(),
            visibility: thought.visibility.clone(),
            reply_to_id: thought.reply_to_id,
            created_at: thought.created_at.into(),
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtListSchema {
    pub thoughts: Vec<ThoughtSchema>,
}

impl From<Vec<ThoughtSchema>> for ThoughtListSchema {
    fn from(thoughts: Vec<ThoughtSchema>) -> Self {
        Self { thoughts }
    }
}

#[derive(Debug, FromQueryResult)]
pub struct ThoughtWithAuthor {
    pub id: Uuid,
    pub content: String,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub visibility: Visibility,
    pub author_id: Uuid,
    pub author_username: String,
    pub reply_to_id: Option<Uuid>,
}

impl From<ThoughtWithAuthor> for ThoughtSchema {
    fn from(model: ThoughtWithAuthor) -> Self {
        Self {
            id: model.id,
            author_username: model.author_username,
            content: model.content,
            created_at: model.created_at.into(),
            reply_to_id: model.reply_to_id,
            visibility: model.visibility,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtThreadSchema {
    pub id: Uuid,
    pub author_username: String,
    pub content: String,
    pub visibility: Visibility,
    pub reply_to_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZoneWrapper,
    pub replies: Vec<ThoughtThreadSchema>,
}
