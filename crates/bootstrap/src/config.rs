/// All configuration read from environment variables at startup.
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub base_url: String,
    pub nats_url: Option<String>,
    pub port: u16,
    pub allow_registration: bool,
    /// true when RUST_ENV != "production" — enables AP debug mode
    pub debug: bool,
    pub host: String,
    pub cors_origins: String,
    pub rate_limit: Option<u32>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL is required"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET is required"),
            base_url: std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
            nats_url: std::env::var("NATS_URL").ok(),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            allow_registration: std::env::var("ALLOW_REGISTRATION")
                .map(|v| v == "true")
                .unwrap_or(true),
            debug: std::env::var("RUST_ENV")
                .map(|v| v != "production")
                .unwrap_or(true),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            cors_origins: std::env::var("CORS_ORIGINS").unwrap_or_else(|_| "*".into()),
            rate_limit: std::env::var("RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }
}
