use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(crate::handlers::health::health_handler))]
pub struct HealthDoc;
