use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::federation_management::get_pending_requests,
        crate::handlers::federation_management::post_accept_request,
        crate::handlers::federation_management::delete_follower,
        crate::handlers::federation_management::get_remote_followers,
        crate::handlers::federation_management::get_remote_following,
        crate::handlers::federation_management::get_remote_friends_handler,
        crate::handlers::federation_management::delete_following,
        crate::handlers::federation_management::post_move_account,
        crate::handlers::federation_management::patch_also_known_as,
    ),
    components(schemas(
        api_types::responses::RemoteActorResponse,
        api_types::responses::ProfileField,
        crate::handlers::federation_management::ActorUrlBody,
        crate::handlers::federation_management::HandleBody,
        crate::handlers::federation_management::MoveBody,
        crate::handlers::federation_management::AlsoKnownAsBody,
    ))
)]
pub struct FederationManagementDoc;
