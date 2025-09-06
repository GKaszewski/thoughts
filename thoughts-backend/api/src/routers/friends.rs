use crate::{error::ApiError, extractor::AuthUser};
use app::{persistence::user, state::AppState};
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use models::schemas::user::UserListSchema;

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "List of authenticated user's friends", body = UserListSchema)
    ),
    security(("bearer_auth" = []))
)]
async fn get_friends_list(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let friends = user::get_friends(&state.conn, auth_user.id).await?;
    Ok(Json(UserListSchema::from(friends)))
}

pub fn create_friends_router() -> Router<AppState> {
    Router::new().route("/", get(get_friends_list))
}
