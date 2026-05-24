use crate::{
    errors::ApiError,
    extractors::{Deps, FromAppState},
    state::AppState,
};
use axum::{
    body::Body,
    extract::Path,
    http::header,
    response::{IntoResponse, Response},
};
use domain::ports::MediaStore;
use futures::TryStreamExt;
use std::sync::Arc;

pub struct MediaDeps {
    pub media: Arc<dyn MediaStore>,
}

impl FromAppState for MediaDeps {
    fn from_state(s: &AppState) -> Self {
        Self {
            media: s.media.clone(),
        }
    }
}

fn ext_to_mime(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}

pub async fn get_media(
    Deps(d): Deps<MediaDeps>,
    Path(path): Path<String>,
) -> Result<Response, ApiError> {
    let stream = d.media.get(&path).await?;
    let content_type = path
        .rsplit('.')
        .next()
        .map(ext_to_mime)
        .unwrap_or("application/octet-stream");
    let body = Body::from_stream(stream.map_err(|e| e.to_string()));
    Ok(([(header::CONTENT_TYPE, content_type)], body).into_response())
}
