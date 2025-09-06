use common::DateTimeWithTimeZoneWrapper;
use serde::Serialize;
use utoipa::ToSchema;

use crate::domains::user;

#[derive(Serialize, ToSchema)]
pub struct UserSchema {
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_url: Option<String>,
    pub custom_css: Option<String>,
    // In a real implementation, you'd fetch and return this data.
    // For now, we'll omit it from the schema to keep it simple.
    // pub top_friends: Vec<String>,
    pub joined_at: DateTimeWithTimeZoneWrapper,
}

impl From<user::Model> for UserSchema {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            header_url: user.header_url,
            custom_css: user.custom_css,
            joined_at: user.created_at.into(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct UserListSchema {
    pub users: Vec<UserSchema>,
}

impl From<Vec<user::Model>> for UserListSchema {
    fn from(users: Vec<user::Model>) -> Self {
        Self {
            users: users.into_iter().map(UserSchema::from).collect(),
        }
    }
}
