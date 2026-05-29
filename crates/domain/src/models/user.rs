use crate::value_objects::{Email, PasswordHash, UserId, Username};
use chrono::{DateTime, Utc};

#[derive(Debug, Default, Clone)]
pub struct UpdateProfileInput {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_url: Option<String>,
    pub custom_css: Option<String>,
    pub profile_fields: Option<Vec<(String, String)>>,
    pub custom_moods: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub email: Email,
    pub password_hash: PasswordHash,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_url: Option<String>,
    pub custom_css: Option<String>,
    pub profile_fields: Vec<(String, String)>,
    pub custom_moods: Vec<(String, String)>,
    pub local: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new_local(
        id: UserId,
        username: Username,
        email: Email,
        password_hash: PasswordHash,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email,
            password_hash,
            display_name: None,
            bio: None,
            avatar_url: None,
            header_url: None,
            custom_css: None,
            profile_fields: vec![],
            custom_moods: vec![],
            local: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_remote(id: UserId, username: Username, email: Email) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email,
            password_hash: PasswordHash(String::new()),
            display_name: None,
            bio: None,
            avatar_url: None,
            header_url: None,
            custom_css: None,
            profile_fields: vec![],
            custom_moods: vec![],
            local: false,
            created_at: now,
            updated_at: now,
        }
    }
}
