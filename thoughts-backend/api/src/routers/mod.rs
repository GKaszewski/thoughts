use axum::Router;

pub mod api_key;
pub mod auth;
pub mod feed;
pub mod friends;
pub mod root;
pub mod search;
pub mod tag;
pub mod thought;
pub mod user;

use crate::routers::auth::create_auth_router;
use app::state::AppState;
use root::create_root_router;
use tower_http::cors::CorsLayer;
use user::create_user_router;

use crate::routers::{feed::create_feed_router, thought::create_thought_router};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .merge(create_root_router())
        .nest("/auth", create_auth_router())
        .nest("/users", create_user_router())
        .nest("/thoughts", create_thought_router())
        .nest("/feed", create_feed_router())
        .nest("/tags", tag::create_tag_router())
        .nest("/friends", friends::create_friends_router())
        .nest("/search", search::create_search_router())
        .with_state(state)
        .layer(cors)
}
