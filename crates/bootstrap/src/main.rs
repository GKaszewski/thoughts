mod config;
mod factory;

const MS_PER_MINUTE: u64 = 60_000;
const RATE_LIMITER_CLEANUP_INTERVAL_SECS: u64 = 60;

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let cfg = config::Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let infra = factory::build(&cfg).await;

    // CORS
    let cors = if cfg.cors_origins.trim() == "*" {
        CorsLayer::permissive()
    } else {
        let origins: Vec<http::HeaderValue> = cfg
            .cors_origins
            .split(',')
            .map(|o| o.trim())
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    };

    let base = presentation::routes::router()
        .merge(infra.ap_service.router::<presentation::state::AppState>())
        .with_state(infra.state)
        .layer(cors);

    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    if let Some(rate_limit) = cfg.rate_limit {
        use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer}; // crate: tower_governor

        // per_millisecond sets the token replenishment interval.
        // rate_limit = max requests/minute => replenish every (60000 / rate_limit) ms.
        let ms = MS_PER_MINUTE.saturating_div(rate_limit as u64).max(1);
        let governor_conf = Arc::new(
            GovernorConfigBuilder::default()
                .per_millisecond(ms)
                .burst_size(rate_limit)
                .use_headers()
                .finish()
                .expect("valid rate limit config"),
        );

        let limiter = governor_conf.limiter().clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                RATE_LIMITER_CLEANUP_INTERVAL_SECS,
            ));
            loop {
                interval.tick().await;
                limiter.retain_recent();
            }
        });

        let app = base.layer(GovernorLayer::new(governor_conf));
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    } else {
        axum::serve(listener, base).await.unwrap();
    }
}
