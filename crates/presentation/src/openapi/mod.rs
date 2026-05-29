mod api_keys;
mod auth;
mod federation_actors;
mod federation_management;
mod feed;
mod health;
mod notifications;
mod social;
mod thoughts;
mod users;

use axum::Router;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key"))),
        );
    }
}

fn build() -> utoipa::openapi::OpenApi {
    let mut api = auth::AuthDoc::openapi();
    api.info = utoipa::openapi::InfoBuilder::new()
        .title("Thoughts API")
        .version("2.0.0")
        .description(Some(
            "Federated social network API. Authenticate via `POST /auth/login` to get a Bearer token, \
             or use `X-Api-Key` header with a key from `POST /api-keys`."
        ))
        .build();
    api.merge(users::UsersDoc::openapi());
    api.merge(thoughts::ThoughtsDoc::openapi());
    api.merge(feed::FeedDoc::openapi());
    api.merge(social::SocialDoc::openapi());
    api.merge(notifications::NotificationsDoc::openapi());
    api.merge(api_keys::ApiKeysDoc::openapi());
    api.merge(health::HealthDoc::openapi());
    api.merge(federation_management::FederationManagementDoc::openapi());
    api.merge(federation_actors::FederationActorsDoc::openapi());
    SecurityAddon.modify(&mut api);
    api
}

pub fn serve<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    tracing::info!("API docs at /docs (Swagger UI) and /scalar (Scalar)");
    let spec = build();
    router
        .merge(SwaggerUi::new("/docs").url("/openapi.json", spec.clone()))
        .merge(Scalar::with_url("/scalar", spec))
}
