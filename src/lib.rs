use std::sync::Arc;

use config::Config;
use database::{Database, Repository};

pub mod config;
pub mod database;
pub mod models;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    db: Arc<dyn Repository + Send + Sync>,
    config: Config,
}

pub async fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    tracing::info!("Environment configuration loaded: {:?}", config);

    let db = Database::connect(&config).await?;
    tracing::info!("Connected to Postgres");

    db.migrate()
        .await?;
    tracing::info!("Migrations run");

    let state = Arc::new(AppState {
        db: Arc::new(db),
        config,
    });

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
