use crate::errors::DomainError;
use uuid::Uuid;

const MAX_USERNAME_LENGTH: usize = 32;
const MAX_EMAIL_LENGTH: usize = 255;
const MAX_CONTENT_LENGTH: usize = 128;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name(Uuid);
        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn from_uuid(u: Uuid) -> Self {
                Self(u)
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

uuid_id!(UserId);
uuid_id!(ThoughtId);
uuid_id!(LikeId);
uuid_id!(BoostId);
uuid_id!(ApiKeyId);
uuid_id!(NotificationId);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Username(String);
impl Username {
    pub fn new(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into();
        if s.is_empty() || s.len() > MAX_USERNAME_LENGTH {
            return Err(DomainError::InvalidInput("username: 1-32 chars".into()));
        }
        if !s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            return Err(DomainError::InvalidInput(
                "username: alphanumeric, underscore, or dot only".into(),
            ));
        }
        Ok(Self(s))
    }
    pub fn from_trusted(s: String) -> Self {
        Self(s)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Email(String);
impl Email {
    pub fn new(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into().to_lowercase();
        if !s.contains('@') || s.len() > MAX_EMAIL_LENGTH {
            return Err(DomainError::InvalidInput("invalid email".into()));
        }
        Ok(Self(s))
    }
    pub fn from_trusted(s: String) -> Self {
        Self(s)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Content(String);
impl Content {
    pub fn new_local(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into();
        if s.is_empty() || s.len() > MAX_CONTENT_LENGTH {
            return Err(DomainError::InvalidInput("content: 1-128 chars".into()));
        }
        Ok(Self(s))
    }
    pub fn new_remote(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_rejects_empty() {
        assert!(Username::new("").is_err());
    }
    #[test]
    fn username_rejects_too_long() {
        assert!(Username::new("a".repeat(33)).is_err());
    }
    #[test]
    fn username_rejects_invalid_chars() {
        assert!(Username::new("hello world").is_err());
    }
    #[test]
    fn username_accepts_valid() {
        assert!(Username::new("hello_123").is_ok());
    }
    #[test]
    fn content_local_rejects_over_128() {
        assert!(Content::new_local("a".repeat(129)).is_err());
    }
    #[test]
    fn content_local_accepts_128() {
        assert!(Content::new_local("a".repeat(128)).is_ok());
    }
    #[test]
    fn email_rejects_no_at() {
        assert!(Email::new("notanemail").is_err());
    }
}
