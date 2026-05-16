use crate::{
    extractors::{Deps, FromAppState},
    state::AppState,
};
use axum::Json;
use domain::ports::UserRepository;
use std::sync::Arc;

pub struct HealthDeps {
    pub users: Arc<dyn UserRepository>,
}

impl FromAppState for HealthDeps {
    fn from_state(s: &AppState) -> Self {
        Self {
            users: s.users.clone(),
        }
    }
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "Service health status")))]
pub async fn health_handler(Deps(d): Deps<HealthDeps>) -> Json<serde_json::Value> {
    let db_ok = d.users.list_with_stats().await.is_ok();
    Json(serde_json::json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "db": if db_ok { "connected" } else { "error" },
    }))
}
