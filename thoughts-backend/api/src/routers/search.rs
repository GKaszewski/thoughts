use crate::{error::ApiError, extractor::OptionalAuthUser};
use app::{persistence::search, state::AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use models::schemas::{
    search::SearchResultsSchema,
    thought::{ThoughtListSchema, ThoughtSchema},
    user::UserListSchema,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct SearchQuery {
    q: String,
}

#[utoipa::path(
    get,
    path = "",
    params(SearchQuery),
    responses((status = 200, body = SearchResultsSchema))
)]
async fn search_all(
    State(state): State<AppState>,
    viewer: OptionalAuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let viewer_id = viewer.0.map(|u| u.id);

    let (users, thoughts) = tokio::try_join!(
        search::search_users(&state.conn, &query.q),
        search::search_thoughts(&state.conn, &query.q, viewer_id)
    )?;

    let thought_schemas: Vec<ThoughtSchema> =
        thoughts.into_iter().map(ThoughtSchema::from).collect();

    let response = SearchResultsSchema {
        users: UserListSchema::from(users),
        thoughts: ThoughtListSchema::from(thought_schemas),
    };

    Ok(Json(response))
}

pub fn create_search_router() -> Router<AppState> {
    Router::new().route("/", get(search_all))
}
