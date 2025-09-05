use crate::domains::{thought, user};
use common::DateTimeWithTimeZoneWrapper;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, FromQueryResult)]
pub struct ThoughtSchema {
    pub id: i32,
    #[schema(example = "frutiger")]
    pub author_username: String,
    #[schema(example = "This is my first thought! #welcome")]
    pub content: String,
    pub created_at: DateTimeWithTimeZoneWrapper,
}

impl ThoughtSchema {
    pub fn from_models(thought: &thought::Model, author: &user::Model) -> Self {
        Self {
            id: thought.id,
            author_username: author.username.clone(),
            content: thought.content.clone(),
            created_at: thought.created_at.into(),
        }
    }
}

#[derive(Serialize, ToSchema)]
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
    pub id: i32,
    pub content: String,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub author_id: i32,
    pub author_username: String,
}

impl From<ThoughtWithAuthor> for ThoughtSchema {
    fn from(model: ThoughtWithAuthor) -> Self {
        Self {
            id: model.id,
            author_username: model.author_username,
            content: model.content,
            created_at: model.created_at.into(),
        }
    }
}
