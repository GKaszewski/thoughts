#[derive(Debug)]
pub enum UserError {
    NotFound,
    NotFollowing,
    Forbidden,
    UsernameTaken,
    AlreadyFollowing,
    Internal(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::NotFound => write!(f, "User not found"),
            UserError::NotFollowing => write!(f, "You are not following this user"),
            UserError::Forbidden => write!(f, "You do not have permission to perform this action"),
            UserError::UsernameTaken => write!(f, "Username is already taken"),
            UserError::AlreadyFollowing => write!(f, "You are already following this user"),
            UserError::Internal(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for UserError {}
