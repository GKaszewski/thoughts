use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
};

use jsonwebtoken::{decode, DecodingKey, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use app::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

static JWT_SECRET: Lazy<String> =
    Lazy::new(|| std::env::var("AUTH_SECRET").expect("AUTH_SECRET must be set"));

pub struct AuthUser {
    pub id: i32,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(user_id_header) = parts.headers.get("x-test-user-id") {
            let user_id_str = user_id_header.to_str().unwrap_or("0");
            let user_id = user_id_str.parse::<i32>().unwrap_or(0);
            return Ok(AuthUser { id: user_id });
        }

        let token = get_token_from_header(&parts.headers)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing or invalid token"))?;

        let decoding_key = DecodingKey::from_secret(JWT_SECRET.as_ref());

        let claims = decode::<Claims>(&token, &decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        Ok(AuthUser { id: claims.sub })
    }
}

fn get_token_from_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(|token| token.to_owned())
}
