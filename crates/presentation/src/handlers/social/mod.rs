use crate::handlers::auth::to_user_response;
use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps},
};
use api_types::requests::{PaginationQuery, SetTopFriendsRequest};
use api_types::responses::{PagedResponse, TopFriendsResponse, UserResponse};
use application::use_cases::profile::{get_top_friends, get_user_by_username, set_top_friends};
use application::use_cases::social::*;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use domain::{
    ports::{
        BlockRepository, BoostRepository, EventPublisher, FederationActionPort, FollowRepository,
        LikeRepository, TopFriendRepository, UserRepository,
    },
    value_objects::{ThoughtId, UserId},
};
use uuid::Uuid;

deps_struct!(SocialDeps {
    likes: LikeRepository,
    boosts: BoostRepository,
    follows: FollowRepository,
    users: UserRepository,
    federation: FederationActionPort,
    events: EventPublisher,
    blocks: BlockRepository,
    top_friends: TopFriendRepository,
});

#[utoipa::path(post, path = "/thoughts/{id}/like", params(("id" = uuid::Uuid, Path, description = "Thought ID")), responses((status = 204, description = "Liked")), security(("bearer_auth" = [])))]
pub async fn post_like(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    like_thought(&*d.likes, &*d.events, &uid, &ThoughtId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(delete, path = "/thoughts/{id}/like", params(("id" = uuid::Uuid, Path, description = "Thought ID")), responses((status = 204, description = "Unliked")), security(("bearer_auth" = [])))]
pub async fn delete_like(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    unlike_thought(&*d.likes, &*d.events, &uid, &ThoughtId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(post, path = "/thoughts/{id}/boost", params(("id" = uuid::Uuid, Path, description = "Thought ID")), responses((status = 204, description = "Boosted")), security(("bearer_auth" = [])))]
pub async fn post_boost(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    boost_thought(&*d.boosts, &*d.events, &uid, &ThoughtId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(delete, path = "/thoughts/{id}/boost", params(("id" = uuid::Uuid, Path, description = "Thought ID")), responses((status = 204, description = "Unboosted")), security(("bearer_auth" = [])))]
pub async fn delete_boost(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    unboost_thought(&*d.boosts, &*d.events, &uid, &ThoughtId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(
    post, path = "/users/{username}/follow",
    params(("username" = String, Path, description = "Username or user@domain handle")),
    responses((status = 204, description = "Following")),
    security(("bearer_auth" = []))
)]
pub async fn post_follow(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    follow_actor(
        &*d.follows,
        &*d.users,
        &*d.federation,
        &*d.events,
        &uid,
        &username,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(
    delete, path = "/users/{username}/follow",
    params(("username" = String, Path, description = "Username")),
    responses((status = 204, description = "Unfollowed")),
    security(("bearer_auth" = []))
)]
pub async fn delete_follow(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    unfollow_actor(
        &*d.follows,
        &*d.users,
        &*d.federation,
        &*d.events,
        &uid,
        &username,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(post, path = "/users/{username}/block", params(("username" = String, Path, description = "Username")), responses((status = 204, description = "Blocked")), security(("bearer_auth" = [])))]
pub async fn post_block(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    block_by_username(
        &*d.blocks,
        &*d.users,
        &*d.federation,
        &*d.events,
        &uid,
        &username,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(delete, path = "/users/{username}/block", params(("username" = String, Path, description = "Username")), responses((status = 204, description = "Unblocked")), security(("bearer_auth" = [])))]
pub async fn delete_block(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    unblock_by_username(
        &*d.blocks,
        &*d.users,
        &*d.federation,
        &*d.events,
        &uid,
        &username,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(put, path = "/users/me/top-friends", request_body = SetTopFriendsRequest, responses((status = 204, description = "Top friends updated")), security(("bearer_auth" = [])))]
pub async fn put_top_friends(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<SetTopFriendsRequest>,
) -> Result<StatusCode, ApiError> {
    let ids: Vec<UserId> = body.friend_ids.into_iter().map(UserId::from_uuid).collect();
    set_top_friends(&*d.top_friends, &uid, ids).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(get, path = "/users/{username}/top-friends",
    params(("username" = String, Path, description = "Username")),
    responses((status = 200, description = "Top friends list", body = TopFriendsResponse)))]
pub async fn get_top_friends_handler(
    Deps(d): Deps<SocialDeps>,
    Path(username): Path<String>,
) -> Result<Json<TopFriendsResponse>, ApiError> {
    let user = get_user_by_username(&*d.users, &username).await?;
    let friends = get_top_friends(&*d.top_friends, &user.id).await?;
    let top_friends = friends.iter().map(|(_, u)| to_user_response(u)).collect();
    Ok(Json(TopFriendsResponse { top_friends }))
}

#[utoipa::path(
    get, path = "/users/me/friends",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Local mutual follows (paginated)", body = inline(PagedResponse<UserResponse>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_friends_handler(
    Deps(d): Deps<SocialDeps>,
    AuthUser(uid): AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PagedResponse<UserResponse>>, ApiError> {
    use domain::models::feed::PageParams;
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let result = get_local_friends(&*d.follows, &uid, &page).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_user_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    }))
}

#[cfg(test)]
mod tests;
