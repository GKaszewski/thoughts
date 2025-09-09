use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use app::{
    persistence::{follow::get_following_ids, thought::get_feed_for_users_and_self_paginated},
    state::AppState,
};
use models::{
    queries::pagination::PaginationQuery,
    schemas::{pagination::PaginatedResponse, thought::ThoughtSchema},
};

use crate::{error::ApiError, extractor::AuthUser};

#[utoipa::path(
    get,
    path = "",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Authenticated user's feed", body = PaginatedResponse<ThoughtSchema>)
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn feed_get(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let following_ids = get_following_ids(&state.conn, auth_user.id).await?;
    let (thoughts_with_authors, total_items) = get_feed_for_users_and_self_paginated(
        &state.conn,
        auth_user.id,
        following_ids,
        &pagination,
    )
    .await?;

    let thoughts_schema: Vec<ThoughtSchema> = thoughts_with_authors
        .into_iter()
        .map(ThoughtSchema::from)
        .collect();

    let page = pagination.page();
    let page_size = pagination.page_size();
    let total_pages = (total_items as f64 / page_size as f64).ceil() as u64;

    let response = PaginatedResponse {
        items: thoughts_schema,
        total_items,
        total_pages,
        page,
        page_size,
    };

    Ok(Json(response))
}

pub fn create_feed_router() -> Router<AppState> {
    Router::new().route("/", get(feed_get))
}
