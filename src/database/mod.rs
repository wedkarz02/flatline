use std::str::FromStr;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::User;

pub mod postgres;

#[derive(Debug, Clone)]
pub enum DatabaseVariant {
    Postgres,
}

impl FromStr for DatabaseVariant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "postgres" => Ok(Self::Postgres),
            _ => Err(s.to_owned()),
        }
    }
}

impl std::fmt::Display for DatabaseVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseVariant::Postgres => write!(f, "postgresql"),
        }
    }
}

#[async_trait]
pub trait Database: Send + Sync + std::fmt::Debug {
    async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError>;
    fn users(&self) -> &dyn UserRepository;
}

#[async_trait]
pub trait UserRepository {
    async fn create(&self, username: &str, password: &str) -> Result<User, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn delete_all(&self) -> Result<u64, sqlx::Error>;
}
