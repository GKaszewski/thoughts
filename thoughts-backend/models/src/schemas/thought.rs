use crate::domains::thought;
use common::DateTimeWithTimeZoneWrapper;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ThoughtSchema {
    pub id: i32,
    pub author_id: i32,
    pub content: String,
    pub created_at: DateTimeWithTimeZoneWrapper,
}

impl From<thought::Model> for ThoughtSchema {
    fn from(model: thought::Model) -> Self {
        Self {
            id: model.id,
            author_id: model.author_id,
            content: model.content,
            created_at: model.created_at.into(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ThoughtListSchema {
    pub thoughts: Vec<ThoughtSchema>,
}

impl From<Vec<thought::Model>> for ThoughtListSchema {
    fn from(models: Vec<thought::Model>) -> Self {
        Self {
            thoughts: models.into_iter().map(ThoughtSchema::from).collect(),
        }
    }
}
