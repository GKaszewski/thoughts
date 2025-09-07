use axum::Router;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, Http, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "root", description = "Root API"),
        (name = "auth", description = "Authentication API"),
        (name = "user", description = "User & Social API"),
        (name = "thought", description = "Thoughts API"),
        (name = "feed", description = "Feed API"),
        (name = "tag", description = "Tag Discovery API"),
        (name = "friends", description = "Friends API"),
        (name = "search", description = "Search API"),
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

pub trait ApiDocExt {
    fn attach_doc(self) -> Self;
}

impl ApiDocExt for Router {
    fn attach_doc(self) -> Self {
        tracing::info!("Attaching API documentation");

        self.merge(SwaggerUi::new("/docs").url("/openapi.json", _ApiDoc::openapi()))
            .merge(Scalar::with_url("/scalar", _ApiDoc::openapi()))
    }
}
