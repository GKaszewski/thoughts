use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use app::state::AppState;

// A dummy struct to represent an authenticated user.
// In a real app, this would contain user details from a validated JWT.
pub struct AuthUser {
    pub id: i32,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // For now, we'll just return a hardcoded user.
        // In a real implementation, you would:
        // 1. Extract the `Authorization: Bearer <token>` header.
        // 2. Validate the JWT.
        // 3. Extract the user ID from the token claims.
        // 4. Return an error if the token is invalid or missing.
        if let Some(user_id_header) = parts.headers.get("x-test-user-id") {
            let user_id_str = user_id_header.to_str().unwrap_or("1");
            let user_id = user_id_str.parse::<i32>().unwrap_or(1);
            return Ok(AuthUser { id: user_id });
        } else {
            return Ok(AuthUser { id: 1 });
        }
    }
}
