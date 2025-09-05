use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

mod feed;
mod root;
mod thought;
mod user;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = root::RootApi),
        (path = "/users", api = user::UserApi),
        (path = "/thoughts", api = thought::ThoughtApi),
        (path = "/feed", api = feed::FeedApi),
    ),
    tags(
        (name = "root", description = "Root API"),
        (name = "user", description = "User & Social API"),
        (name = "thought", description = "Thoughts API"),
        (name = "feed", description = "Feed API"),
    ),
)]
struct _ApiDoc;

pub trait ApiDoc {
    fn attach_doc(self) -> Self;
}

impl ApiDoc for Router {
    fn attach_doc(self) -> Self {
        self.merge(SwaggerUi::new("/docs").url("/openapi.json", _ApiDoc::openapi()))
            .merge(Scalar::with_url("/scalar", _ApiDoc::openapi()))
    }
}
