use crate::{errors::ApiError, state::AppState};
use axum::{extract::FromRequestParts, http::request::Parts};
use domain::value_objects::UserId;

// ---------------------------------------------------------------------------
// deps_struct! — generates Deps struct + impl FromAppState from a field list.
// Field names must match AppState exactly (enforced at compile time).
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! deps_struct {
    ( $name:ident { $( $field:ident : $trait:path ),+ $(,)? } ) => {
        pub struct $name {
            $( pub $field: ::std::sync::Arc<dyn $trait>, )+
        }
        impl $crate::extractors::FromAppState for $name {
            fn from_state(s: &$crate::state::AppState) -> Self {
                Self {
                    $( $field: ::std::sync::Arc::clone(&s.$field), )+
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Deps<S> extractor — narrows AppState to a handler-specific deps struct
// ---------------------------------------------------------------------------

pub struct Deps<S>(pub S);

pub trait FromAppState: Sized {
    fn from_state(s: &AppState) -> Self;
}

impl<S: FromAppState + Send + 'static> FromRequestParts<AppState> for Deps<S> {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Deps(S::from_state(state)))
    }
}

// ---------------------------------------------------------------------------
// Auth extractors
// ---------------------------------------------------------------------------

pub struct AuthUser(pub UserId);
pub struct OptionalAuthUser(pub Option<UserId>);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        extract_user_id(parts, state)
            .await?
            .ok_or(ApiError::Unauthorized)
            .map(AuthUser)
    }
}

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        Ok(OptionalAuthUser(extract_user_id(parts, state).await?))
    }
}

async fn extract_user_id(parts: &mut Parts, state: &AppState) -> Result<Option<UserId>, ApiError> {
    if let Some(auth_header) = parts.headers.get("Authorization") {
        if let Ok(s) = auth_header.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                return state
                    .auth
                    .validate_token(token)
                    .map(Some)
                    .map_err(|_| ApiError::Unauthorized);
            }
        }
    }
    if let Some(key_header) = parts.headers.get("X-Api-Key") {
        if let Ok(raw) = key_header.to_str() {
            return state
                .api_key_auth
                .validate_key(raw)
                .await
                .map_err(|_| ApiError::Unauthorized);
        }
    }
    Ok(None)
}
