use crate::{handlers::*, openapi, state::AppState};
use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, patch, post, put},
    Router,
};

pub fn router() -> Router<AppState> {
    let api_routes = Router::new()
        // health
        .route("/health", get(health::health_handler))
        // auth
        .route("/auth/register", post(auth::post_register))
        .route("/auth/login", post(auth::post_login))
        // users — static before parameterised
        .route("/users", get(users::get_users))
        .route("/users/count", get(users::get_user_count))
        .route("/users/lookup", get(users::lookup_handler))
        .route("/users/me", get(users::get_me).patch(users::patch_profile))
        .route(
            "/users/me/avatar",
            put(users::upload_avatar).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route(
            "/users/me/banner",
            put(users::upload_banner).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/users/me/following", get(users::get_me_following))
        .route("/users/me/friends", get(social::get_friends_handler))
        .route("/users/me/top-friends", put(social::put_top_friends))
        .route("/users/{username}", get(users::get_user))
        .route(
            "/users/{username}/top-friends",
            get(social::get_top_friends_handler),
        )
        .route(
            "/users/{username}/follow",
            post(social::post_follow).delete(social::delete_follow),
        )
        .route(
            "/users/{username}/block",
            post(social::post_block).delete(social::delete_block),
        )
        .route(
            "/users/{username}/followers",
            get(feed::get_followers_handler),
        )
        .route(
            "/users/{username}/following",
            get(feed::get_following_handler),
        )
        .route(
            "/users/{username}/thoughts",
            get(feed::user_thoughts_handler),
        )
        // thoughts
        .route("/thoughts", post(thoughts::post_thought))
        .route(
            "/thoughts/{id}",
            get(thoughts::get_thought_handler)
                .patch(thoughts::patch_thought)
                .delete(thoughts::delete_thought_handler),
        )
        .route("/thoughts/{id}/thread", get(thoughts::get_thread_handler))
        // likes & boosts
        .route(
            "/thoughts/{id}/like",
            post(social::post_like).delete(social::delete_like),
        )
        .route(
            "/thoughts/{id}/boost",
            post(social::post_boost).delete(social::delete_boost),
        )
        // feeds
        .route("/feed", get(feed::home_feed))
        .route("/feed/public", get(feed::public_feed))
        .route("/search", get(feed::search_handler))
        .route(
            "/federation/actors/{handle}/posts",
            get(federation_actors::remote_actor_posts_handler),
        )
        .route(
            "/federation/actors/{handle}/followers-list",
            get(federation_actors::actor_followers_handler),
        )
        .route(
            "/federation/actors/{handle}/following-list",
            get(federation_actors::actor_following_handler),
        )
        .route(
            "/federation/me/followers/pending",
            get(federation_management::get_pending_requests),
        )
        .route(
            "/federation/me/followers/accept",
            post(federation_management::post_accept_request),
        )
        .route(
            "/federation/me/followers",
            get(federation_management::get_remote_followers)
                .delete(federation_management::delete_follower),
        )
        .route(
            "/federation/me/following",
            get(federation_management::get_remote_following)
                .delete(federation_management::delete_following),
        )
        .route(
            "/federation/me/move",
            post(federation_management::post_move_account),
        )
        .route(
            "/federation/me/also-known-as",
            patch(federation_management::patch_also_known_as),
        )
        .route("/tags/popular", get(feed::get_popular_tags))
        .route("/tags/{name}", get(feed::tag_thoughts_handler))
        // notifications
        .route(
            "/notifications",
            get(notifications::list_notifications).patch(notifications::mark_all_read),
        )
        .route(
            "/notifications/{id}",
            patch(notifications::mark_notification_read),
        )
        // api keys
        .route(
            "/api-keys",
            get(api_keys::get_api_keys).post(api_keys::post_api_key),
        )
        .route("/api-keys/{id}", delete(api_keys::delete_api_key_handler));

    openapi::serve(api_routes).route("/media/{*path}", get(media::get_media))
}
