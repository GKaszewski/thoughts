mod api_key_service;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use domain::{
    errors::DomainError,
    ports::{AuthService, GeneratedToken, PasswordHasher},
    value_objects::{PasswordHash, UserId},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub use api_key_service::ApiKeyServiceImpl;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct JwtAuthService {
    secret: String,
    ttl_seconds: i64,
}

impl JwtAuthService {
    pub fn new(secret: String, ttl_seconds: i64) -> Self {
        Self {
            secret,
            ttl_seconds,
        }
    }
}

impl AuthService for JwtAuthService {
    fn generate_token(&self, user_id: &UserId) -> Result<GeneratedToken, DomainError> {
        let exp = (Utc::now() + Duration::seconds(self.ttl_seconds)).timestamp() as usize;
        let claims = Claims {
            sub: user_id.as_uuid().to_string(),
            exp,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(GeneratedToken {
            token,
            user_id: user_id.clone(),
        })
    }

    fn validate_token(&self, token: &str) -> Result<UserId, DomainError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| DomainError::Unauthorized)?;
        let uuid =
            uuid::Uuid::parse_str(&data.claims.sub).map_err(|_| DomainError::Unauthorized)?;
        Ok(UserId::from_uuid(uuid))
    }
}

pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, plain: &str) -> Result<PasswordHash, DomainError> {
        use argon2::{password_hash::SaltString, Argon2, PasswordHasher as _};
        use rand::rngs::OsRng;
        let salt = SaltString::generate(OsRng);
        let hash = Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .to_string();
        Ok(PasswordHash(hash))
    }

    async fn verify(&self, plain: &str, hash: &PasswordHash) -> Result<bool, DomainError> {
        if hash.0.starts_with("$2") {
            return bcrypt::verify(plain, &hash.0)
                .map_err(|e| DomainError::Internal(e.to_string()));
        }
        use argon2::{password_hash::PasswordHash as ArgonHash, Argon2, PasswordVerifier};
        let parsed = ArgonHash::new(&hash.0).map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok())
    }
}

#[cfg(test)]
mod tests;
