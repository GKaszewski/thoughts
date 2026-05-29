use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::federation_actors::remote_actor_posts_handler,
        crate::handlers::federation_actors::actor_followers_handler,
        crate::handlers::federation_actors::actor_following_handler,
    ),
    components(schemas(
        api_types::responses::ActorConnectionPageResponse,
        api_types::responses::ActorConnectionResponse,
    )),
)]
pub struct FederationActorsDoc;
