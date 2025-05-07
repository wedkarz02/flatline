use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

use crate::{config::Config, error::ApiError, models::user::User};

use super::{Database, UserRepository};

#[derive(Clone, Debug)]
pub struct PostgresDatabase {
    pub pool: Pool<Postgres>,
}

impl PostgresDatabase {
    pub async fn connect(cfg: &Config) -> Result<Arc<dyn Database>, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(cfg.database_pool)
            .connect(&cfg.database_uri())
            .await?;

        Ok(Arc::new(Self { pool }))
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn migrate(&self) -> Result<(), ApiError> {
        sqlx::migrate!("./src/database/migrations")
            .run(&self.pool)
            .await?;

        Ok(())
    }

    fn users(&self) -> &dyn UserRepository {
        self
    }
}

#[async_trait]
impl UserRepository for PostgresDatabase {
    async fn create(&self, username: &str, password: &str) -> Result<User, ApiError> {
        let mut tx = self.pool.begin().await?;

        let created_user = sqlx::query_as::<_, User>("INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id, username, password")
            .bind(username)
            .bind(password)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(created_user)
    }

    async fn find_all(&self) -> Result<Vec<User>, ApiError> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(Some(user))
    }

    async fn delete_all(&self) -> Result<u64, ApiError> {
        let mut tx = self.pool.begin().await?;

        let deleted_count = sqlx::query("DELETE FROM users")
            .execute(&mut *tx)
            .await?
            .rows_affected();

        tx.commit().await?;
        Ok(deleted_count)
    }
}
