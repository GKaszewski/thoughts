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
    // Storage
    pub storage_backend: String,
    pub storage_path: Option<String>,
    pub storage_prefix: String,
    pub s3_endpoint: Option<String>,
    pub s3_access_key_id: Option<String>,
    pub s3_secret_access_key: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    // Upload limits
    pub upload_max_bytes: usize,
    pub upload_allowed_types: Vec<String>,
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
            storage_backend: std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".into()),
            storage_path: std::env::var("STORAGE_PATH").ok(),
            storage_prefix: std::env::var("STORAGE_PREFIX").unwrap_or_default(),
            s3_endpoint: std::env::var("S3_ENDPOINT").ok(),
            s3_access_key_id: std::env::var("S3_ACCESS_KEY_ID").ok(),
            s3_secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY").ok(),
            s3_bucket: std::env::var("S3_BUCKET").ok(),
            s3_region: std::env::var("S3_REGION").ok(),
            upload_max_bytes: std::env::var("UPLOAD_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5 * 1024 * 1024),
            upload_allowed_types: std::env::var("UPLOAD_ALLOWED_TYPES")
                .unwrap_or_else(|_| "image/jpeg,image/png,image/gif,image/webp,image/avif".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}
