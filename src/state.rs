use std::{collections::HashMap, sync::{Arc, atomic::AtomicUsize}};
use axum::extract::ws::Message;
use sqlx::PgPool;
use tokio::sync::{RwLock, mpsc};
use tower_http::cors::CorsLayer;
use crate::config::Config;

type Users = Arc<RwLock<HashMap<usize, mpsc::Sender<Message>>>>;

#[derive(Clone)]
pub struct AppConfig {
    pub secret: String,
    pub cors: CorsLayer,
    pub access_ttl: u32,
    pub refresh_ttl: u32,
}


#[derive(Clone)]
pub struct AppState {
    pub app_config: Arc<AppConfig>,
    pub database: PgPool,
    pub users: Users,
    pub n_user_id: Arc<AtomicUsize>,
}

impl AppState {
    pub async fn new(config: Config, pool: PgPool) -> Self {
        let app_config = Arc::new(AppConfig {
            secret: config.jwt_secret,
            cors: config.cors,
            access_ttl: config.access_ttl,
            refresh_ttl: config.refresh_ttl,
        });

    
        Self {
            app_config,
            database: pool,
            users: Arc::new(RwLock::new(HashMap::new())),
            n_user_id: Arc::new(AtomicUsize::new(1)),
        }
    }
}