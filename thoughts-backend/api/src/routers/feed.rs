use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use app::{
    persistence::{follow::get_following_ids, thought::get_feed_for_user},
    state::AppState,
};
use models::schemas::thought::{ThoughtListSchema, ThoughtSchema};

use crate::{error::ApiError, extractor::AuthUser};

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Authenticated user's feed", body = ThoughtListSchema)
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn feed_get(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let following_ids = get_following_ids(&state.conn, auth_user.id).await?;
    let mut thoughts_with_authors = get_feed_for_user(&state.conn, following_ids).await?;

    let own_thoughts = get_feed_for_user(&state.conn, vec![auth_user.id]).await?;
    thoughts_with_authors.extend(own_thoughts);

    let thoughts_schema: Vec<ThoughtSchema> = thoughts_with_authors
        .into_iter()
        .map(ThoughtSchema::from)
        .collect();

    Ok(Json(ThoughtListSchema::from(thoughts_schema)))
}

pub fn create_feed_router() -> Router<AppState> {
    Router::new().route("/", get(feed_get))
}
