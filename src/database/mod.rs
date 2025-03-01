use crate::config::Config;
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use user_repo::UserRepository;

mod user_repo;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn connect(cfg: &Config) -> Result<Database, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(cfg.postgres_pool)
            .connect(&cfg.postgres_uri())
            .await?;

        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./src/database/migrations")
            .run(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
pub trait Repository {
    fn users(&self) -> &dyn UserRepository;
}

#[async_trait]
impl Repository for Database {
    fn users(&self) -> &dyn UserRepository {
        self
    }
}
