const MAX_TOP_FRIENDS: usize = 8;

use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{top_friend::TopFriend, user::User},
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

#[allow(clippy::too_many_arguments)]
pub async fn update_profile(
    users: &dyn UserWriter,
    events: &dyn EventPublisher,
    user_id: &UserId,
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
    header_url: Option<String>,
    custom_css: Option<String>,
) -> Result<(), DomainError> {
    users
        .update_profile(
            user_id,
            display_name,
            bio,
            avatar_url,
            header_url,
            custom_css,
        )
        .await?;
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
mod tests {
    use super::*;
    use domain::{
        errors::DomainError,
        models::user::User,
        testing::TestStore,
        value_objects::{Email, PasswordHash, UserId, Username},
    };

    fn make_user() -> User {
        User::new_local(
            UserId::new(),
            Username::new("alice").unwrap(),
            Email::new("alice@ex.com").unwrap(),
            PasswordHash("h".into()),
        )
    }

    #[tokio::test]
    async fn set_top_friends_rejects_more_than_eight() {
        let store = TestStore::default();
        let uid = UserId::new();
        let friends: Vec<UserId> = (0..9).map(|_| UserId::new()).collect();
        let err = set_top_friends(&store, &uid, friends).await.unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn set_top_friends_assigns_sequential_positions() {
        let store = TestStore::default();
        let uid = UserId::new();
        let f1 = UserId::new();
        let f2 = UserId::new();
        let f3 = UserId::new();
        set_top_friends(&store, &uid, vec![f1.clone(), f2.clone(), f3.clone()])
            .await
            .unwrap();
        let tf = store.top_friends.lock().unwrap();
        assert_eq!(tf.len(), 3);
        let pos_f1 = tf
            .iter()
            .find(|t| t.friend_id == f1)
            .map(|t| t.position)
            .unwrap();
        let pos_f2 = tf
            .iter()
            .find(|t| t.friend_id == f2)
            .map(|t| t.position)
            .unwrap();
        assert!(pos_f1 < pos_f2, "f1 should come before f2");
    }

    #[tokio::test]
    async fn get_user_by_username_returns_not_found_for_missing_user() {
        let store = TestStore::default();
        let err = get_user_by_username(&store, "nobody").await.unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn get_user_by_username_returns_correct_user() {
        let store = TestStore::default();
        let user = make_user();
        store.users.lock().unwrap().push(user.clone());
        let found = get_user_by_username(&store, "alice").await.unwrap();
        assert_eq!(found.id, user.id);
    }
}
