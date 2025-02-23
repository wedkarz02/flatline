use config::Config;
use sqlx::postgres::PgPoolOptions;

pub mod config;

pub async fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    tracing::info!("Environment configuration loaded: {:#?}", config);

    let pool = PgPoolOptions::new()
        .max_connections(config.postgres_pool)
        .connect(&config.postgres_uri())
        .await?;

    match sqlx::query("SELECT 1")
        .execute(&pool)
        .await
    {
        Ok(v) => tracing::info!("Database connection healthy: {:?}", v),
        Err(_) => tracing::error!("Database connection failed"),
    }

    let listener = tokio::net::TcpListener::bind(config.socket_addr()).await?;
    tracing::info!("Listening on: {}", listener.local_addr()?);

    Ok(())
}
