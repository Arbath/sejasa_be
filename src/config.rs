use std::env;
use http::{HeaderValue, Method, header};
use tower_http::cors::CorsLayer;
use tracing::Level;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
    pub cors: CorsLayer,
    pub access_ttl:u32,
    pub refresh_ttl:u32,
    pub db_url: String,
    pub db_max_con:u32,
    pub migrate: bool,
    pub log_level: Level
}

impl Config {
    pub fn init() -> Config {
        let port_str = env::var("APP_PORT").unwrap_or_else(|_| "8000".to_string());
        let port = port_str
            .trim()
            .parse::<u16>()
            .expect(&format!("Invalid APP_PORT: '{}'", port_str));

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET required");
        let cors_origins_str = env::var("CORS_ORIGINS").expect("CORS_ORIGINS required in .env");
        let access_ttl = env::var("ACCESS_TTL_IN_MINUTES").ok().and_then(|v| v.parse::<u32>().map(|v| v.max(1)).ok()).unwrap_or(15) * 60;
        let refresh_ttl = env::var("REFRESH_TTL_IN_DAYS").ok().and_then(|v| v.parse::<u32>().map(|v| v.max(1)).ok()).unwrap_or(7) * 24 * 60 * 60;
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL required");
        let db_max_con = env::var("DB_CONNECTIONS").ok().and_then(|v| v.parse::<u32>().map(|v| v.max(1)).ok()).unwrap_or(5);
        let migrate = env::var("MIGRATE").unwrap_or("false".to_string()).to_lowercase().parse::<bool>().unwrap_or(false);
        let log_level_str = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string()).to_uppercase();

        let log_level = match log_level_str.as_str() {
            "TRACE" => Level::TRACE,
            "DEBUG" => Level::DEBUG,
            "WARN"  => Level::WARN,
            "ERROR" => Level::ERROR,
            _ => Level::INFO,
        };
        let origins: Vec<HeaderValue> = cors_origins_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<HeaderValue>().expect("Invalid URI in CORS_ORIGINS"))
            .collect();
        let cors = CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

        Config {
            port,
            jwt_secret,
            cors,
            access_ttl,
            refresh_ttl,
            db_url,
            db_max_con,
            migrate,
            log_level
        }
    }
}