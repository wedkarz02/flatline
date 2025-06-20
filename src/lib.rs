use std::sync::Arc;

use crate::database::postgres::PostgresDatabase;
use config::Config;
use database::{mock::MockDatabase, Database};

pub mod config;
pub mod database;
pub mod error;
pub mod models;
pub mod routes;

#[derive(Clone)]
pub struct ApiState {
    db: Arc<dyn Database>,
    config: Config,
}

async fn init_database(cfg: &Config) -> anyhow::Result<Arc<dyn Database>> {
    let db: Arc<dyn Database> = match cfg.database_variant {
        database::DatabaseVariant::Postgres => PostgresDatabase::connect(cfg).await?,
        database::DatabaseVariant::Mock => MockDatabase::new(),
        database::DatabaseVariant::Sqlite => unimplemented!("Sqlite3 is not implemented"),
        database::DatabaseVariant::MySql => unimplemented!("MySql is not implemented"),
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

    let mut config_json = serde_json::to_value(&config)?;
    if let Some(map) = config_json.as_object_mut() {
        map.insert(
            "database_password".to_string(),
            serde_json::Value::String("<redacted>".to_string()),
        );
    }

    tracing::info!(
        config = %config_json,
        "Environment configuration loaded"
    );

    let db = init_database(&config).await?;
    let state = Arc::new(ApiState { db, config });

    let listener = tokio::net::TcpListener::bind(state.config.socket_addr()).await?;
    tracing::info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, routes::create_routes(state))
        .with_graceful_shutdown(ctrl_c())
        .await?;

    Ok(())
}
