use url::Url;

use crate::error::Error;

pub const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";
pub const AP_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";
pub const AP_PAGE_SIZE: usize = 20;

pub fn extract_user_id_from_url(url: &Url) -> Option<uuid::Uuid> {
    let path = url.path();
    path.strip_prefix("/users/")
        .and_then(|s| s.split('/').next())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
}

pub fn activity_url(base_url: &str) -> Result<Url, Error> {
    Url::parse(&format!("{}/activities/{}", base_url, uuid::Uuid::new_v4()))
        .map_err(|e| Error::bad_request(anyhow::anyhow!(e)))
}

pub fn actor_url(base_url: &str, user_id: uuid::Uuid) -> Url {
    Url::parse(&format!("{}/users/{}", base_url, user_id))
        .expect("base_url is always a valid URL prefix")
}

/// Extract the username segment from a /users/:username URL.
#[allow(dead_code)]
pub fn extract_username_from_url(url: &Url) -> Option<String> {
    url.path()
        .strip_prefix("/users/")
        .and_then(|s| s.split('/').next())
        .map(|s| s.to_string())
}
