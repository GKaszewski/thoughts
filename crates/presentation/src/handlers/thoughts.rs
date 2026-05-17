use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps, OptionalAuthUser},
    handlers::feed::to_thought_response,
};
use api_types::{
    requests::{CreateThoughtRequest, EditThoughtRequest},
    responses::ErrorResponse,
};
use application::use_cases::thoughts::{
    create_thought, delete_thought, edit_thought, get_thought_view, get_thread_views,
    CreateThoughtInput,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use domain::{
    models::feed::{EngagementStats, FeedEntry, ViewerContext},
    ports::{
        EngagementRepository, EventPublisher, OutboxWriter, TagRepository, ThoughtRepository,
        UserRepository,
    },
    value_objects::ThoughtId,
};
use uuid::Uuid;

deps_struct!(ThoughtsDeps {
    thoughts: ThoughtRepository,
    users: UserRepository,
    tags: TagRepository,
    events: EventPublisher,
    outbox: OutboxWriter,
    engagement: EngagementRepository,
});

#[utoipa::path(
    post, path = "/thoughts",
    request_body = CreateThoughtRequest,
    responses(
        (status = 201, description = "Thought created"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 422, description = "Content too long", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn post_thought(
    Deps(d): Deps<ThoughtsDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<CreateThoughtRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let in_reply_to = body.in_reply_to_id.map(ThoughtId::from_uuid);
    let out = create_thought(
        &*d.thoughts,
        &*d.users,
        &*d.tags,
        &*d.events,
        &*d.outbox,
        CreateThoughtInput {
            user_id: uid.clone(),
            content: body.content,
            in_reply_to_id: in_reply_to,
            visibility: body.visibility,
            content_warning: body.content_warning,
            sensitive: body.sensitive.unwrap_or(false),
        },
    )
    .await?;
    let author = d
        .users
        .find_by_id(&uid)
        .await?
        .ok_or(domain::errors::DomainError::NotFound)?;
    let entry = FeedEntry {
        thought: out.thought,
        author,
        stats: EngagementStats {
            like_count: 0,
            boost_count: 0,
            reply_count: 0,
        },
        viewer: Some(ViewerContext {
            liked: false,
            boosted: false,
        }),
    };
    Ok((StatusCode::CREATED, Json(to_thought_response(&entry))))
}

#[utoipa::path(
    get, path = "/thoughts/{id}",
    params(("id" = uuid::Uuid, Path, description = "Thought ID")),
    responses(
        (status = 200, description = "Thought with author info"),
        (status = 404, description = "Not found", body = ErrorResponse),
    )
)]
pub async fn get_thought_handler(
    Deps(d): Deps<ThoughtsDeps>,
    Path(id): Path<Uuid>,
    OptionalAuthUser(viewer): OptionalAuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let entry = get_thought_view(
        &*d.thoughts,
        &*d.users,
        &*d.engagement,
        &ThoughtId::from_uuid(id),
        viewer.as_ref(),
    )
    .await?;
    Ok(Json(
        serde_json::to_value(to_thought_response(&entry)).unwrap(),
    ))
}

#[utoipa::path(
    delete, path = "/thoughts/{id}",
    params(("id" = uuid::Uuid, Path, description = "Thought ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found or not owner", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_thought_handler(
    Deps(d): Deps<ThoughtsDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    delete_thought(
        &*d.thoughts,
        &*d.events,
        &*d.outbox,
        &ThoughtId::from_uuid(id),
        &uid,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch, path = "/thoughts/{id}",
    params(("id" = uuid::Uuid, Path, description = "Thought ID")),
    request_body = EditThoughtRequest,
    responses(
        (status = 204, description = "Updated"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found or not owner", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_thought(
    Deps(d): Deps<ThoughtsDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<EditThoughtRequest>,
) -> Result<StatusCode, ApiError> {
    edit_thought(
        &*d.thoughts,
        &*d.events,
        &ThoughtId::from_uuid(id),
        &uid,
        body.content,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get, path = "/thoughts/{id}/thread",
    params(("id" = uuid::Uuid, Path, description = "Root thought ID")),
    responses(
        (status = 200, description = "Thread (root + replies)"),
    )
)]
pub async fn get_thread_handler(
    Deps(d): Deps<ThoughtsDeps>,
    Path(id): Path<Uuid>,
    OptionalAuthUser(viewer): OptionalAuthUser,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let entries = get_thread_views(
        &*d.thoughts,
        &*d.users,
        &*d.engagement,
        &ThoughtId::from_uuid(id),
        viewer.as_ref(),
    )
    .await?;
    let items: Vec<_> = entries
        .iter()
        .map(|e| serde_json::to_value(to_thought_response(e)).unwrap())
        .collect();
    Ok(Json(items))
}
