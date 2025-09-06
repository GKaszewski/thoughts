use axum::Router;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, Http, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

mod api_key;
mod auth;
mod feed;
mod friends;
mod root;
mod tag;
mod thought;
mod user;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = root::RootApi),
        (path = "/auth", api = auth::AuthApi),
        (path = "/users", api = user::UserApi),
        (path = "/users/me/api-keys", api = api_key::ApiKeyApi),
        (path = "/thoughts", api = thought::ThoughtApi),
        (path = "/feed", api = feed::FeedApi),
        (path = "/tags", api = tag::TagApi),
        (path = "/friends", api = friends::FriendsApi),
    ),
    tags(
        (name = "root", description = "Root API"),
        (name = "auth", description = "Authentication API"),
        (name = "user", description = "User & Social API"),
        (name = "thought", description = "Thoughts API"),
        (name = "feed", description = "Feed API"),
        (name = "tag", description = "Tag Discovery API"),
        (name = "friends", description = "Friends API"),
    ),
    modifiers(&SecurityAddon),
)]
struct _ApiDoc;

struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer)),
        );
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
        );
    }
}

pub trait ApiDoc {
    fn attach_doc(self) -> Self;
}

impl ApiDoc for Router {
    fn attach_doc(self) -> Self {
        self.merge(SwaggerUi::new("/docs").url("/openapi.json", _ApiDoc::openapi()))
            .merge(Scalar::with_url("/scalar", _ApiDoc::openapi()))
    }
}
