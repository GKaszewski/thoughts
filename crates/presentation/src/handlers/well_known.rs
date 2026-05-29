use axum::{extract::State, http::header, response::IntoResponse};

use crate::state::AppState;

pub async fn host_meta(State(state): State<AppState>) -> impl IntoResponse {
    let domain = url::Url::parse(&state.base_url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0">
  <Link rel="lrdd" template="https://{domain}/.well-known/webfinger?resource={{uri}}"/>
</XRD>"#
    );
    (
        [(header::CONTENT_TYPE, "application/xrd+xml; charset=utf-8")],
        body,
    )
}
