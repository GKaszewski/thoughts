use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use models::{
    domains::user,
    params::auth::{LoginParams, RegisterParams},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use validator::Validate; // Import the Validate trait

use crate::error::UserError;

fn hash_password(password: &str) -> Result<String, BcryptError> {
    hash(password, DEFAULT_COST)
}

pub async fn register_user(db: &DbConn, params: RegisterParams) -> Result<user::Model, UserError> {
    params
        .validate()
        .map_err(|e| UserError::Validation(e.to_string()))?;

    let hashed_password =
        hash_password(&params.password).map_err(|e| UserError::Internal(e.to_string()))?;

    let new_user = user::ActiveModel {
        username: Set(params.username.clone()),
        password_hash: Set(Some(hashed_password)),
        email: Set(Some(params.email)),
        display_name: Set(Some(params.username)),
        ..Default::default()
    };

    new_user.insert(db).await.map_err(|e| {
        if let Some(sea_orm::SqlErr::UniqueConstraintViolation { .. }) = e.sql_err() {
            UserError::UsernameTaken
        } else {
            UserError::Internal(e.to_string())
        }
    })
}

pub async fn authenticate_user(db: &DbConn, params: LoginParams) -> Result<user::Model, UserError> {
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(params.username))
        .one(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?
        .ok_or(UserError::NotFound)?;

    let password_hash = user.password_hash.as_ref().ok_or(UserError::NotFound)?;

    if verify(params.password, password_hash).map_err(|e| UserError::Internal(e.to_string()))? {
        Ok(user)
    } else {
        Err(UserError::NotFound)
    }
}
