use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

use crate::{
    config::Config,
    database::RefreshTokenRepository,
    error::ApiError,
    models::{refresh_token::RefreshToken, user::User},
};

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

    fn refresh_tokens(&self) -> &dyn RefreshTokenRepository {
        self
    }
}

#[async_trait]
impl UserRepository for PostgresDatabase {
    async fn create(&self, user: User) -> Result<User, ApiError> {
        let mut tx = self.pool.begin().await?;

        let created_user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, password_hash, roles, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, password_hash, roles, created_at, updated_at
            "#,
        )
        .bind(user.id)
        .bind(user.username)
        .bind(user.password_hash)
        .bind(user.roles)
        .bind(user.created_at)
        .bind(user.updated_at)
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
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
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

#[async_trait]
impl RefreshTokenRepository for PostgresDatabase {
    async fn create(&self, refresh_token: RefreshToken) -> Result<RefreshToken, ApiError> {
        let mut tx = self.pool.begin().await?;

        let created_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (jti, sub, exp, iat, token_hash) 
            VALUES ($1, $2, $3, $4, $5)
            RETURNING jti, sub, exp, iat, token_hash 
            "#,
        )
        .bind(refresh_token.jti)
        .bind(refresh_token.sub)
        .bind(refresh_token.exp)
        .bind(refresh_token.iat)
        .bind(refresh_token.token_hash)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(created_token)
    }
}
