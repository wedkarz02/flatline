use std::{path::PathBuf, sync::Arc};

use crate::database::postgres::PostgresDatabase;
use config::Config;
use database::{mock::MockDatabase, Database};
use serde::Serialize;

pub mod config;
pub mod database;
pub mod error;
pub mod models;
pub mod routes;
pub mod services;

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

fn redact_fields<T>(data: &T, fields: &[&str]) -> anyhow::Result<serde_json::Value>
where
    T: Serialize,
{
    let mut data_json = serde_json::to_value(data)?;

    if let Some(map) = data_json.as_object_mut() {
        for field in fields {
            map.insert(
                field.to_string(),
                serde_json::Value::String("<redacted>".to_string()),
            );
        }
    }

    Ok(data_json)
}

async fn ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
    tracing::info!("Ctrl-C signal received, shutting down...");
}

pub async fn run(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let config = match config_path {
        Some(path) => Config::from_json(&path)?,
        None => Config::from_env(),
    };

    let redacted_config = redact_fields(
        &config,
        &[
            "database_password",
            "jwt_access_secret",
            "jwt_refresh_secret",
        ],
    )?;

    tracing::info!(
        config = %redacted_config,
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
