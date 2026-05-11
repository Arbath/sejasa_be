use axum::serve;
use dotenvy::dotenv;
use tracing::{info, error};

use sejasa::{config::Config, database::{self, migrate_app}, routes::create_app, state::AppState};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::init();
    
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(config.log_level)
        .compact()
        .init();

    let pool = database::create_pool(&config.db_url, config.db_max_con).await;
    if config.migrate {
        let _ = migrate_app(&pool).await.expect("Migrations failed!\n");
    }

    let state = AppState::new(config.clone(), pool).await;

    run_axum_server(state, &config).await;
}

async fn run_axum_server(state: AppState, config: &Config) {
    let addr = format!("0.0.0.0:{}", config.port);
    let app = create_app(state);
    
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind port {}: {:?}", config.port, e);
            return;
        }
    };

    info!("Axum Server listening on {}", addr);
    info!("Log level: {:?}", config.log_level);

    if let Err(e) = serve(listener, app).await {
        error!("Fatal Server Error: {:?}", e);
    }
}