use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::User;

use super::Database;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, username: &str, password: &str) -> Result<User, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn delete_all(&self) -> Result<u64, sqlx::Error>;
}

#[async_trait]
impl UserRepository for Database {
    async fn create(&self, username: &str, password: &str) -> Result<User, sqlx::Error> {
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

    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(Some(user))
    }

    async fn delete_all(&self) -> Result<u64, sqlx::Error> {
        let mut tx = self
            .pool
            .begin()
            .await?;

        let deleted_count = sqlx::query("DELETE FROM users")
            .execute(&mut *tx)
            .await?
            .rows_affected();

        tx.commit()
            .await?;

        Ok(deleted_count)
    }
}
