use crate::error::ApiError;
use app::{persistence::thought::get_thoughts_by_tag_name, state::AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use models::schemas::thought::{ThoughtListSchema, ThoughtSchema};

#[utoipa::path(
    get,
    path = "{tagName}",
    params(("tagName" = String, Path, description = "Tag name")),
    responses((status = 200, description = "List of thoughts with a specific tag", body = ThoughtListSchema))
)]
async fn get_thoughts_by_tag(
    State(state): State<AppState>,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let thoughts_with_authors = get_thoughts_by_tag_name(&state.conn, &tag_name).await;
    let thoughts_with_authors = thoughts_with_authors?;
    let thoughts_schema: Vec<ThoughtSchema> = thoughts_with_authors
        .into_iter()
        .map(ThoughtSchema::from)
        .collect();
    Ok(Json(ThoughtListSchema::from(thoughts_schema)))
}

pub fn create_tag_router() -> Router<AppState> {
    Router::new().route("/{tag_name}", get(get_thoughts_by_tag))
}
