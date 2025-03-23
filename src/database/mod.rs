use std::str::FromStr;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::User;

pub mod postgres;

#[derive(Debug, Clone)]
pub enum SupportedDatabases {
    Postgres,
}

impl FromStr for SupportedDatabases {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "postgres" => Ok(Self::Postgres),
            _ => Err(s.to_owned()),
        }
    }
}

impl ToString for SupportedDatabases {
    fn to_string(&self) -> String {
        match self {
            SupportedDatabases::Postgres => String::from("postgresql"),
        }
    }
}

#[async_trait]
pub trait Database: Send + Sync {
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
