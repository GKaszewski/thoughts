use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps, OptionalAuthUser},
    handlers::auth::to_user_response,
};
use api_types::requests::{PaginationQuery, SearchQuery};
use api_types::responses::{PagedResponse, ThoughtResponse};
use application::use_cases::feed::{
    get_home_feed, get_popular_tags as uc_get_popular_tags, get_public_feed, get_tag_feed,
    get_user_feed,
};
use application::use_cases::profile::{
    get_user_by_id_or_username, get_user_by_username, list_local_followers, list_local_following,
};
use axum::{
    extract::{Path, Query},
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
    Json,
};
use domain::{
    models::feed::PageParams,
    ports::{
        FederationActionPort, FeedFilter, FeedOptions, FeedRepository, FeedSort, FollowRepository,
        SearchPort, TagRepository, UserRepository,
    },
};

#[derive(serde::Deserialize, Default, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FeedOptionsQuery {
    /// Sort order: `newest` (default), `oldest`, `most_liked`, `most_boosted`, `most_discussed`
    pub sort: Option<String>,
    /// Show only original posts (mutually exclusive with `replies_only`)
    pub originals_only: Option<bool>,
    /// Show only replies (mutually exclusive with `originals_only`)
    pub replies_only: Option<bool>,
    /// Show only posts from this instance
    pub local_only: Option<bool>,
    /// Hide posts marked as sensitive
    pub hide_sensitive: Option<bool>,
}

impl TryFrom<FeedOptionsQuery> for FeedOptions {
    type Error = crate::errors::ApiError;

    fn try_from(q: FeedOptionsQuery) -> Result<Self, Self::Error> {
        if q.originals_only.unwrap_or(false) && q.replies_only.unwrap_or(false) {
            return Err(crate::errors::ApiError::BadRequest(
                "originals_only and replies_only are mutually exclusive".to_string(),
            ));
        }
        let sort = match q.sort.as_deref() {
            None | Some("newest") => FeedSort::Newest,
            Some("oldest") => FeedSort::Oldest,
            Some("most_liked") => FeedSort::MostLiked,
            Some("most_boosted") => FeedSort::MostBoosted,
            Some("most_discussed") => FeedSort::MostDiscussed,
            Some(other) => {
                return Err(crate::errors::ApiError::BadRequest(format!(
                    "unknown sort value: {other}"
                )))
            }
        };
        Ok(FeedOptions {
            sort,
            filter: FeedFilter {
                originals_only: q.originals_only.unwrap_or(false),
                replies_only: q.replies_only.unwrap_or(false),
                local_only: q.local_only.unwrap_or(false),
                hide_sensitive: q.hide_sensitive.unwrap_or(false),
            },
        })
    }
}

fn wants_activity_json(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("application/activity+json"))
}

deps_struct!(FeedDeps {
    feed: FeedRepository,
    follows: FollowRepository,
    search: SearchPort,
    federation: FederationActionPort,
    users: UserRepository,
    tags: TagRepository,
});

pub fn to_thought_response(e: &domain::models::feed::FeedEntry) -> ThoughtResponse {
    ThoughtResponse {
        id: e.thought.id.as_uuid(),
        content: e.thought.content.as_str().to_string(),
        author: to_user_response(&e.author),
        in_reply_to_id: e.thought.in_reply_to_id.as_ref().map(|id| id.as_uuid()),
        in_reply_to_url: e.thought.in_reply_to_url.clone(),
        visibility: e.thought.visibility.as_str().to_string(),
        content_warning: e.thought.content_warning.clone(),
        sensitive: e.thought.sensitive,
        like_count: e.stats.like_count,
        boost_count: e.stats.boost_count,
        reply_count: e.stats.reply_count,
        liked_by_viewer: e.viewer.as_ref().map(|v| v.liked).unwrap_or(false),
        boosted_by_viewer: e.viewer.as_ref().map(|v| v.boosted).unwrap_or(false),
        created_at: e.thought.created_at,
        updated_at: e.thought.updated_at,
        note_extensions: e.thought.note_extensions.clone(),
        mood: e.thought.mood.clone(),
    }
}

#[utoipa::path(
    get, path = "/feed",
    params(PaginationQuery, FeedOptionsQuery),
    responses((status = 200, description = "Home feed")),
    security(("bearer_auth" = []))
)]
pub async fn home_feed(
    Deps(d): Deps<FeedDeps>,
    AuthUser(uid): AuthUser,
    Query(q): Query<PaginationQuery>,
    Query(opts_q): Query<FeedOptionsQuery>,
) -> Result<Json<PagedResponse<ThoughtResponse>>, ApiError> {
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let opts = FeedOptions::try_from(opts_q)?;
    let result = get_home_feed(&*d.feed, &*d.follows, &uid, page, opts).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_thought_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    }))
}

#[utoipa::path(
    get, path = "/feed/public",
    params(PaginationQuery, FeedOptionsQuery),
    responses((status = 200, description = "Public feed"))
)]
pub async fn public_feed(
    Deps(d): Deps<FeedDeps>,
    OptionalAuthUser(viewer): OptionalAuthUser,
    Query(q): Query<PaginationQuery>,
    Query(opts_q): Query<FeedOptionsQuery>,
) -> Result<Json<PagedResponse<ThoughtResponse>>, ApiError> {
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let opts = FeedOptions::try_from(opts_q)?;
    let result = get_public_feed(&*d.feed, viewer, page, opts).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_thought_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    }))
}

#[utoipa::path(
    get, path = "/search",
    params(SearchQuery),
    responses((status = 200, description = "Search results: thoughts and users"))
)]
pub async fn search_handler(
    Deps(d): Deps<FeedDeps>,
    OptionalAuthUser(viewer): OptionalAuthUser,
    Query(q): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = PageParams {
        page: q.page.unwrap_or(api_types::requests::DEFAULT_PAGE),
        per_page: q.per_page.unwrap_or(api_types::requests::DEFAULT_PER_PAGE),
    };
    let query = q.q.trim().to_string();

    let (thoughts_result, users_result) = tokio::join!(
        d.search.search_thoughts(&query, &page, viewer.as_ref()),
        d.search.search_users(&query, &page),
    );

    let thoughts = thoughts_result?
        .items
        .iter()
        .map(to_thought_response)
        .collect::<Vec<_>>();

    let users = users_result?
        .items
        .into_iter()
        .map(|u| to_user_response(&u))
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "query": query,
        "thoughts": thoughts,
        "users": users,
    })))
}

#[utoipa::path(
    get, path = "/users/{username}/following",
    params(
        ("username" = String, Path, description = "Username"),
        PaginationQuery,
    ),
    responses((status = 200, description = "Users this account follows"))
)]
pub async fn get_following_handler(
    Deps(d): Deps<FeedDeps>,
    Path(param): Path<String>,
    Query(q): Query<PaginationQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if wants_activity_json(&headers) {
        let user = get_user_by_id_or_username(&*d.users, &param).await?;
        let user_id = user.id;
        let page = q.page().try_into().ok();
        let json = d
            .federation
            .following_collection_json(&user_id, page)
            .await?;
        return Ok(([(header::CONTENT_TYPE, "application/activity+json")], json).into_response());
    }

    let user = get_user_by_username(&*d.users, &param).await?;
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let result = list_local_following(&*d.follows, &user.id, page).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_user_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    })
    .into_response())
}

#[utoipa::path(
    get, path = "/users/{username}/followers",
    params(
        ("username" = String, Path, description = "Username"),
        PaginationQuery,
    ),
    responses((status = 200, description = "Accounts that follow this user"))
)]
pub async fn get_followers_handler(
    Deps(d): Deps<FeedDeps>,
    Path(param): Path<String>,
    Query(q): Query<PaginationQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if wants_activity_json(&headers) {
        let user = get_user_by_id_or_username(&*d.users, &param).await?;
        let user_id = user.id;
        let page = q.page().try_into().ok();
        let json = d
            .federation
            .followers_collection_json(&user_id, page)
            .await?;
        return Ok(([(header::CONTENT_TYPE, "application/activity+json")], json).into_response());
    }

    let user = get_user_by_username(&*d.users, &param).await?;
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let result = list_local_followers(&*d.follows, &user.id, page).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_user_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    })
    .into_response())
}

#[utoipa::path(
    get, path = "/users/{username}/thoughts",
    params(
        ("username" = String, Path, description = "Username"),
        PaginationQuery,
        FeedOptionsQuery,
    ),
    responses((status = 200, description = "User's public thoughts"))
)]
pub async fn user_thoughts_handler(
    Deps(d): Deps<FeedDeps>,
    Path(username): Path<String>,
    OptionalAuthUser(viewer): OptionalAuthUser,
    Query(q): Query<PaginationQuery>,
    Query(opts_q): Query<FeedOptionsQuery>,
) -> Result<Json<PagedResponse<ThoughtResponse>>, ApiError> {
    let user = get_user_by_username(&*d.users, &username).await?;
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let opts = FeedOptions::try_from(opts_q)?;
    let result = get_user_feed(&*d.feed, user.id.clone(), page, opts, viewer).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_thought_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    }))
}

#[utoipa::path(
    get, path = "/tags/popular",
    params(
        ("limit" = Option<u64>, Query, description = "Max tags to return (default 20, max 100)"),
    ),
    responses((status = 200, description = "Most-used tags"))
)]
pub async fn get_popular_tags(
    Deps(d): Deps<FeedDeps>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit: usize = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(api_types::requests::DEFAULT_PER_PAGE as usize)
        .min(api_types::requests::MAX_PER_PAGE as usize);
    let tags = uc_get_popular_tags(&*d.tags, limit).await?;
    Ok(Json(serde_json::json!({
        "tags": tags.iter().map(|(name, count)| serde_json::json!({
            "name": name,
            "thought_count": count,
        })).collect::<Vec<_>>()
    })))
}

#[utoipa::path(
    get, path = "/tags/{name}",
    params(
        ("name" = String, Path, description = "Tag name"),
        PaginationQuery,
        FeedOptionsQuery,
    ),
    responses((status = 200, description = "Thoughts with this tag"))
)]
pub async fn tag_thoughts_handler(
    Deps(d): Deps<FeedDeps>,
    Path(tag_name): Path<String>,
    OptionalAuthUser(viewer): OptionalAuthUser,
    Query(q): Query<PaginationQuery>,
    Query(opts_q): Query<FeedOptionsQuery>,
) -> Result<Json<PagedResponse<ThoughtResponse>>, ApiError> {
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let opts = FeedOptions::try_from(opts_q)?;
    let result = get_tag_feed(&*d.feed, &tag_name, page, opts, viewer).await?;
    Ok(Json(PagedResponse {
        items: result.items.iter().map(to_thought_response).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    }))
}
