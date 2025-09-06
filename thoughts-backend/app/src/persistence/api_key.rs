use bcrypt::{hash, verify, DEFAULT_COST};
use models::domains::{api_key, user};
use rand::distributions::{Alphanumeric, DistString};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::error::UserError;

const KEY_PREFIX: &str = "th_";
const KEY_RANDOM_LENGTH: usize = 32;
const KEY_LOOKUP_PREFIX_LENGTH: usize = 8;

fn generate_key() -> String {
    let random_part = Alphanumeric.sample_string(&mut rand::thread_rng(), KEY_RANDOM_LENGTH);
    format!("{}{}", KEY_PREFIX, random_part)
}

pub async fn create_api_key(
    db: &DbConn,
    user_id: Uuid,
    name: String,
) -> Result<(api_key::Model, String), UserError> {
    let plaintext_key = generate_key();
    let key_hash =
        hash(&plaintext_key, DEFAULT_COST).map_err(|e| UserError::Internal(e.to_string()))?;
    let key_prefix = plaintext_key[..KEY_LOOKUP_PREFIX_LENGTH].to_string();

    let new_key = api_key::ActiveModel {
        user_id: Set(user_id),
        name: Set(name),
        key_hash: Set(key_hash),
        key_prefix: Set(key_prefix),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|e| UserError::Internal(e.to_string()))?;

    Ok((new_key, plaintext_key))
}

pub async fn validate_api_key(db: &DbConn, plaintext_key: &str) -> Result<user::Model, UserError> {
    if !plaintext_key.starts_with(KEY_PREFIX)
        || plaintext_key.len() != KEY_PREFIX.len() + KEY_RANDOM_LENGTH
    {
        return Err(UserError::Validation("Invalid API key format".to_string()));
    }

    let key_prefix = &plaintext_key[..KEY_LOOKUP_PREFIX_LENGTH];

    let candidate_keys = api_key::Entity::find()
        .filter(api_key::Column::KeyPrefix.eq(key_prefix))
        .all(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?;

    for key in candidate_keys {
        if verify(plaintext_key, &key.key_hash).unwrap_or(false) {
            return super::user::get_user(db, key.user_id)
                .await
                .map_err(|e| UserError::Internal(e.to_string()))?
                .ok_or(UserError::NotFound);
        }
    }

    Err(UserError::Validation("Invalid API key".to_string()))
}

pub async fn get_api_keys_for_user(
    db: &DbConn,
    user_id: Uuid,
) -> Result<Vec<api_key::Model>, DbErr> {
    api_key::Entity::find()
        .filter(api_key::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn delete_api_key(db: &DbConn, key_id: Uuid, user_id: Uuid) -> Result<(), UserError> {
    let result = api_key::Entity::delete_many()
        .filter(api_key::Column::Id.eq(key_id))
        .filter(api_key::Column::UserId.eq(user_id)) // Ensure user owns the key
        .exec(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?;

    if result.rows_affected == 0 {
        Err(UserError::NotFound)
    } else {
        Ok(())
    }
}
