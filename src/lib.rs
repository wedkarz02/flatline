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

pub async fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    tracing::info!("Environment configuration loaded: {:?}", config);

    let db: Arc<dyn Database> = PostgresDatabase::connect(&config).await?;
    tracing::info!("Connected to the database");

    db.migrate()
        .await?;
    tracing::info!("Migrations executed");

    let state = Arc::new(AppState { db, config });

    let listener = tokio::net::TcpListener::bind(
        state
            .config
            .socket_addr(),
    )
    .await?;

    tracing::info!("Listening on: {}", listener.local_addr()?);
    axum::serve(listener, routes::create_routes(state)).await?;

    Ok(())
}
