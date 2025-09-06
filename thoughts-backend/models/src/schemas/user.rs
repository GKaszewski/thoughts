use common::DateTimeWithTimeZoneWrapper;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domains::user;

#[derive(Serialize, ToSchema)]
pub struct UserSchema {
    pub id: Uuid,
    pub username: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub bio: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "headerUrl")]
    pub header_url: Option<String>,
    #[serde(rename = "customCss")]
    pub custom_css: Option<String>,
    #[serde(rename = "topFriends")]
    pub top_friends: Vec<String>,
    #[serde(rename = "joinedAt")]
    pub joined_at: DateTimeWithTimeZoneWrapper,
}

impl From<(user::Model, Vec<user::Model>)> for UserSchema {
    fn from((user, top_friends): (user::Model, Vec<user::Model>)) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            header_url: user.header_url,
            custom_css: user.custom_css,
            top_friends: top_friends.into_iter().map(|u| u.username).collect(),
            joined_at: user.created_at.into(),
        }
    }
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
            top_friends: vec![], // Defaults to an empty list
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

#[derive(Serialize, ToSchema)]
pub struct MeSchema {
    #[serde(flatten)]
    pub user: UserSchema,
    pub following: Vec<UserSchema>,
}
