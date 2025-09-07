use std::time::Duration;

use axum::Router;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use app::config::Config;
use app::state::AppState;

use crate::routers::create_router;

pub fn setup_router(conn: DatabaseConnection, config: &Config) -> Router {
    create_router(AppState {
        conn,
        base_url: config.base_url.clone(),
    })
}

pub fn setup_config() -> Config {
    dotenvy::dotenv().ok();
    Config::from_env()
}

pub async fn setup_db(db_url: &str, prefork: bool) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(db_url);
    opt.max_lifetime(Duration::from_secs(60));

    if !prefork {
        opt.min_connections(10).max_connections(100);
    }

    Database::connect(opt)
        .await
        .expect("Database connection failed")
}
