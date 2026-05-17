use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::user::User,
    ports::{AuthService, EventPublisher, PasswordHasher, UserReader, UserRepository},
    value_objects::{Email, UserId, Username},
};

pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}
#[derive(Debug)]
pub struct RegisterOutput {
    pub user: User,
    pub token: String,
}

pub async fn register(
    users: &dyn UserRepository,
    hasher: &dyn PasswordHasher,
    auth: &dyn AuthService,
    events: &dyn EventPublisher,
    input: RegisterInput,
) -> Result<RegisterOutput, DomainError> {
    let username = Username::new(input.username)?;
    let email = Email::new(input.email)?;
    if users.find_by_username(&username).await?.is_some() {
        return Err(DomainError::Conflict("username taken".into()));
    }
    if users.find_by_email(&email).await?.is_some() {
        return Err(DomainError::Conflict("email taken".into()));
    }
    let hash = hasher.hash(&input.password).await?;
    let user = User::new_local(UserId::new(), username, email, hash);
    users.save(&user).await.map_err(|e| match e {
        DomainError::UniqueViolation { field: "username" } => {
            DomainError::Conflict("username taken".into())
        }
        DomainError::UniqueViolation { field: "email" } => {
            DomainError::Conflict("email taken".into())
        }
        DomainError::UniqueViolation { .. } => DomainError::Conflict("already exists".into()),
        other => other,
    })?;
    events
        .publish(&DomainEvent::UserRegistered {
            user_id: user.id.clone(),
        })
        .await?;
    let token = auth.generate_token(&user.id)?;
    Ok(RegisterOutput {
        user,
        token: token.token,
    })
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}
#[derive(Debug)]
pub struct LoginOutput {
    pub user: User,
    pub token: String,
}

pub async fn login(
    users: &dyn UserReader,
    hasher: &dyn PasswordHasher,
    auth: &dyn AuthService,
    input: LoginInput,
) -> Result<LoginOutput, DomainError> {
    let email = Email::new(input.email)?;
    let user = users.find_by_email(&email).await?;
    if user.is_none() {
        // Timing equalization — prevents email enumeration via response-time oracle.
        // Running the hasher on a miss makes "no such user" take the same time as
        // "wrong password", so attackers cannot distinguish the two cases.
        let _ = hasher.hash(&input.password).await;
        return Err(DomainError::Unauthorized);
    }
    let user = user.unwrap();
    if !hasher.verify(&input.password, &user.password_hash).await? {
        return Err(DomainError::Unauthorized);
    }
    let token = auth.generate_token(&user.id)?;
    Ok(LoginOutput {
        user,
        token: token.token,
    })
}

#[cfg(test)]
mod tests;
