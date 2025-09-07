use super::{thought::ThoughtListSchema, user::UserListSchema};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SearchResultsSchema {
    pub users: UserListSchema,
    pub thoughts: ThoughtListSchema,
}
