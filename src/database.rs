use std::time::Duration;
use std::collections::HashSet;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;
use sqlx::migrate::Migrator;

pub async fn create_pool(database_url : &String, max_con: u32) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(max_con)
        .acquire_timeout(Duration::from_secs(30))
        .min_connections(2)
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .expect("Failed connect to Posgtresql");
    
    info!("Postgresql connected with pool size: {}", max_con);

    pool
}
static APP_MIGRATOR: Migrator = sqlx::migrate!("./migrations");
pub async fn migrate_app(pool: &PgPool) -> Result<(), sqlx::Error>{
    info!("Checking database migrations...");

    let applied: Vec<(i64, String)> = sqlx::query_as(
        r#"
        SELECT version, description
        FROM _sqlx_migrations
        WHERE success = true
        ORDER BY version
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let applied_versions: HashSet<i64> =
        applied.iter().map(|(v, _)| *v).collect();

    // Log
    for migration in APP_MIGRATOR.migrations.iter() {
        if applied_versions.contains(&migration.version) {
            info!(
                "Migration applied : {} - {}",
                migration.version,
                migration.description
            );
        } else {
            info!(
                "Migration pending : {} - {}",
                migration.version,
                migration.description
            );
        }
    }

    APP_MIGRATOR.run(pool).await?;

    info!("Database migrations completed");
    Ok(())
}