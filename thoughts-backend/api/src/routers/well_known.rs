use app::state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize)]
pub struct WebFingerQuery {
    resource: String,
}

#[derive(Serialize)]
pub struct WebFingerLink {
    rel: String,
    #[serde(rename = "type")]
    type_: String,
    href: Url,
}

#[derive(Serialize)]
pub struct WebFingerResponse {
    subject: String,
    links: Vec<WebFingerLink>,
}

pub async fn webfinger(
    State(state): State<AppState>,
    Query(query): Query<WebFingerQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Some((scheme, account_info)) = query.resource.split_once(':') {
        if scheme != "acct" {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid resource scheme",
            ));
        }

        let account_parts: Vec<&str> = account_info.split('@').collect();
        let username = account_parts[0];

        let user = match app::persistence::user::get_user_by_username(&state.conn, username).await {
            Ok(Some(user)) => user,
            _ => return Err((axum::http::StatusCode::NOT_FOUND, "User not found")),
        };

        let user_url = Url::parse(&format!("{}/users/{}", &state.base_url, user.username)).unwrap();

        let response = WebFingerResponse {
            subject: query.resource,
            links: vec![WebFingerLink {
                rel: "self".to_string(),
                type_: "application/activity+json".to_string(),
                href: user_url,
            }],
        };

        Ok(Json(response))
    } else {
        Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Invalid resource format",
        ))
    }
}

pub fn create_well_known_router() -> axum::Router<AppState> {
    axum::Router::new().route("/webfinger", axum::routing::get(webfinger))
}
