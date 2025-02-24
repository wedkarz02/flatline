use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

use crate::{config::Config, models::user::User};

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn connect(cfg: &Config) -> anyhow::Result<Database> {
        let pool = PgPoolOptions::new()
            .max_connections(cfg.postgres_pool)
            .connect(&cfg.postgres_uri())
            .await?;

        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
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

// FIXME: Move the stuff below somewhere

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<User>;
    async fn get_all_users(&self) -> anyhow::Result<Vec<User>>;
    async fn get_user_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>>;
}

#[async_trait]
impl UserRepository for Database {
    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<User> {
        let mut tx = self
            .pool
            .begin()
            .await?;

        let created_user = sqlx::query_as::<_, User>("INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id, username, password")
            .bind(username)
            .bind(password)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit()
            .await?;

        Ok(created_user)
    }

    async fn get_all_users(&self) -> anyhow::Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }
    async fn get_user_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(Some(user))
    }
}
