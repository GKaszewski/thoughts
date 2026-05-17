const MAX_TOP_FRIENDS: usize = 8;

use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        top_friend::TopFriend,
        user::{UpdateProfileInput, User},
    },
    ports::{EventPublisher, TopFriendRepository, UserReader, UserWriter},
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

#[cfg(test)]
mod tests;
