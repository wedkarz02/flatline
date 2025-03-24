use std::sync::Arc;

use crate::database::postgres::PostgresDatabase;
use config::Config;
use database::Database;

pub mod config;
pub mod database;
pub mod error;
pub mod models;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    db: Arc<dyn Database>,
    config: Config,
}

async fn init_database(cfg: &Config) -> anyhow::Result<Arc<dyn Database>> {
    let db: Arc<dyn Database> = match cfg.database_variant {
        database::DatabaseVariant::Postgres => PostgresDatabase::connect(cfg).await?,
    };

    tracing::info!("Connected to {}", cfg.database_variant);
    db.migrate().await?;
    tracing::info!("Migrations executed");
    Ok(db)
}

async fn ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
    tracing::info!("Ctrl-C signal received, shutting down...");
}

pub async fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    tracing::info!("Environment configuration loaded: {:?}", config);

    let db = init_database(&config).await?;
    let state = Arc::new(AppState { db, config });

    let listener = tokio::net::TcpListener::bind(state.config.socket_addr()).await?;
    tracing::info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, routes::create_routes(state))
        .with_graceful_shutdown(ctrl_c())
        .await?;

    Ok(())
}
