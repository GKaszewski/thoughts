use super::AuthUser;
use crate::error::ApiError;
use app::state::AppState;
use axum::{extract::FromRequestParts, http::request::Parts};

pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            // If the user is not authenticated for any reason, we just treat them as a guest.
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
