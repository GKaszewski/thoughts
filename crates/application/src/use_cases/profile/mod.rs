const MAX_TOP_FRIENDS: usize = 8;
const MAX_PROFILE_FIELDS: usize = 4;
const MAX_FIELD_NAME_LEN: usize = 64;
const MAX_FIELD_VALUE_LEN: usize = 256;

use bytes::Bytes;
use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        feed::{PageParams, Paginated, UserSummary},
        top_friend::TopFriend,
        user::{UpdateProfileInput, User},
    },
    ports::{
        EventPublisher, FollowRepository, MediaStore, TopFriendRepository, UserReader,
        UserRepository, UserWriter,
    },
    value_objects::{UserId, Username},
};

pub async fn get_user(users: &dyn UserReader, user_id: &UserId) -> Result<User, DomainError> {
    users
        .find_by_id(user_id)
        .await?
        .ok_or(DomainError::NotFound)
}

pub async fn get_user_by_username(
    users: &dyn UserReader,
    username: &str,
) -> Result<User, DomainError> {
    let username = Username::new(username).map_err(|_| DomainError::NotFound)?;
    users
        .find_by_username(&username)
        .await?
        .ok_or(DomainError::NotFound)
}

/// Resolve a path segment that is either a UUID (AP actor URL) or a username.
pub async fn get_user_by_id_or_username(
    users: &dyn UserReader,
    id_or_username: &str,
) -> Result<User, DomainError> {
    if let Ok(uuid) = uuid::Uuid::parse_str(id_or_username) {
        users
            .find_by_id(&UserId::from_uuid(uuid))
            .await?
            .ok_or(DomainError::NotFound)
    } else {
        get_user_by_username(users, id_or_username).await
    }
}

pub async fn update_profile(
    users: &dyn UserWriter,
    events: &dyn EventPublisher,
    user_id: &UserId,
    input: UpdateProfileInput,
) -> Result<(), DomainError> {
    if let Some(ref fields) = input.profile_fields {
        if fields.len() > MAX_PROFILE_FIELDS {
            return Err(DomainError::InvalidInput(format!(
                "profile fields: max {MAX_PROFILE_FIELDS}"
            )));
        }
        for (name, value) in fields {
            if name.len() > MAX_FIELD_NAME_LEN || value.len() > MAX_FIELD_VALUE_LEN {
                return Err(DomainError::InvalidInput(
                    "profile field name or value too long".into(),
                ));
            }
        }
    }
    users.update_profile(user_id, input).await?;
    events
        .publish(&DomainEvent::ProfileUpdated {
            user_id: user_id.clone(),
        })
        .await
}

pub async fn get_top_friends(
    top_friends: &dyn TopFriendRepository,
    user_id: &UserId,
) -> Result<Vec<(TopFriend, User)>, DomainError> {
    top_friends.list_for_user(user_id).await
}

pub async fn set_top_friends(
    top_friends: &dyn TopFriendRepository,
    user_id: &UserId,
    friend_ids: Vec<UserId>,
) -> Result<(), DomainError> {
    if friend_ids.len() > MAX_TOP_FRIENDS {
        return Err(DomainError::InvalidInput("top friends: max 8".into()));
    }
    let friends: Vec<(UserId, i16)> = friend_ids
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, (i + 1) as i16))
        .collect();
    top_friends.set_top_friends(user_id, friends).await
}

#[derive(Clone)]
pub struct UploadConfig {
    pub max_bytes: usize,
    pub allowed_content_types: Vec<String>,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_bytes: 5 * 1024 * 1024,
            allowed_content_types: vec![
                "image/jpeg".into(),
                "image/png".into(),
                "image/gif".into(),
                "image/webp".into(),
                "image/avif".into(),
            ],
        }
    }
}

fn mime_to_ext(mime: &str) -> Result<&'static str, DomainError> {
    match mime {
        "image/jpeg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/gif" => Ok("gif"),
        "image/webp" => Ok("webp"),
        "image/avif" => Ok("avif"),
        _ => Err(DomainError::InvalidInput("unsupported content type".into())),
    }
}

pub struct UploadContext<'a> {
    pub users: &'a dyn UserRepository,
    pub media: &'a dyn MediaStore,
    pub events: &'a dyn EventPublisher,
    pub upload_config: &'a UploadConfig,
    pub base_url: &'a str,
}

async fn store_image(
    ctx: &UploadContext<'_>,
    content_type: &str,
    data: Bytes,
    user_id: &UserId,
    key_segment: &str,
    old_url: Option<&str>,
) -> Result<String, DomainError> {
    let cfg = ctx.upload_config;
    let media = ctx.media;
    let base_url = ctx.base_url;
    if !cfg.allowed_content_types.iter().any(|t| t == content_type) {
        return Err(DomainError::InvalidInput("unsupported content type".into()));
    }
    if data.len() > cfg.max_bytes {
        return Err(DomainError::InvalidInput("file too large".into()));
    }
    let ext = mime_to_ext(content_type)?;
    if let Some(old) = old_url {
        let prefix = format!("{base_url}/media/");
        if let Some(old_key) = old.strip_prefix(&prefix) {
            media.delete(old_key).await?;
        }
    }
    let key = format!("users/{}/{key_segment}.{ext}", user_id.as_uuid());
    let stream = Box::pin(futures::stream::once(async move { Ok(data) }));
    media.put(&key, stream).await?;
    Ok(key)
}

pub async fn upload_avatar(
    ctx: &UploadContext<'_>,
    user_id: &UserId,
    content_type: &str,
    data: Bytes,
) -> Result<(), DomainError> {
    let current = ctx
        .users
        .find_by_id(user_id)
        .await?
        .ok_or(DomainError::NotFound)?;
    let key = store_image(
        ctx,
        content_type,
        data,
        user_id,
        "avatar",
        current.avatar_url.as_deref(),
    )
    .await?;
    ctx.users
        .update_profile(
            user_id,
            UpdateProfileInput {
                avatar_url: Some(format!("{}/media/{key}", ctx.base_url)),
                ..Default::default()
            },
        )
        .await?;
    ctx.events
        .publish(&DomainEvent::ProfileUpdated {
            user_id: user_id.clone(),
        })
        .await
}

pub async fn upload_banner(
    ctx: &UploadContext<'_>,
    user_id: &UserId,
    content_type: &str,
    data: Bytes,
) -> Result<(), DomainError> {
    let current = ctx
        .users
        .find_by_id(user_id)
        .await?
        .ok_or(DomainError::NotFound)?;
    let key = store_image(
        ctx,
        content_type,
        data,
        user_id,
        "banner",
        current.header_url.as_deref(),
    )
    .await?;
    ctx.users
        .update_profile(
            user_id,
            UpdateProfileInput {
                header_url: Some(format!("{}/media/{key}", ctx.base_url)),
                ..Default::default()
            },
        )
        .await?;
    ctx.events
        .publish(&DomainEvent::ProfileUpdated {
            user_id: user_id.clone(),
        })
        .await
}

pub async fn get_user_profile(
    users: &dyn UserReader,
    follows: &dyn FollowRepository,
    id_or_username: &str,
    viewer_id: Option<&UserId>,
) -> Result<(User, bool), DomainError> {
    let user = get_user_by_id_or_username(users, id_or_username).await?;
    let is_followed = match viewer_id {
        Some(vid) if vid != &user.id => follows.find(vid, &user.id).await?.is_some(),
        _ => false,
    };
    Ok((user, is_followed))
}

pub async fn list_users(
    users: &dyn UserReader,
    page: PageParams,
) -> Result<Paginated<UserSummary>, DomainError> {
    users.list_paginated(page).await
}

pub async fn count_local_users(users: &dyn UserReader) -> Result<i64, DomainError> {
    users.count().await
}

pub async fn list_local_followers(
    follows: &dyn FollowRepository,
    user_id: &UserId,
    page: PageParams,
) -> Result<Paginated<User>, DomainError> {
    follows.list_followers(user_id, &page).await
}

pub async fn list_local_following(
    follows: &dyn FollowRepository,
    user_id: &UserId,
    page: PageParams,
) -> Result<Paginated<User>, DomainError> {
    follows.list_following(user_id, &page).await
}

#[cfg(test)]
mod tests;
