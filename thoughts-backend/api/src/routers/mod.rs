use axum::Router;

pub mod feed;
pub mod root;
pub mod thought;
pub mod user;

use app::state::AppState;
use root::create_root_router;
use user::create_user_router;

use crate::routers::{feed::create_feed_router, thought::create_thought_router};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(create_root_router())
        .nest("/users", create_user_router())
        .nest("/thoughts", create_thought_router())
        .nest("/feed", create_feed_router())
        .with_state(state)
}
