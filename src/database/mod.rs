use std::str::FromStr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{refresh_token::RefreshToken, user::User},
};

pub mod mock;
pub mod postgres;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseVariant {
    Postgres,
    Sqlite,
    MySql,
    Mock,
}

impl FromStr for DatabaseVariant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "postgresql" => Ok(Self::Postgres),
            "sqlite3" => Ok(Self::Sqlite),
            "mysql" => Ok(Self::MySql),
            "mock" => Ok(Self::Mock),
            _ => Err(s.to_owned()),
        }
    }
}

impl std::fmt::Display for DatabaseVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseVariant::Postgres => write!(f, "postgresql"),
            DatabaseVariant::Sqlite => write!(f, "sqlite3"),
            DatabaseVariant::MySql => write!(f, "mysql"),
            DatabaseVariant::Mock => write!(f, "mock"),
        }
    }
}

impl valuable::Valuable for DatabaseVariant {
    fn as_value(&self) -> valuable::Value<'_> {
        match self {
            DatabaseVariant::Postgres => valuable::Value::String("postgresql"),
            DatabaseVariant::Sqlite => valuable::Value::String("sqlite3"),
            DatabaseVariant::MySql => valuable::Value::String("mysql"),
            DatabaseVariant::Mock => valuable::Value::String("mock"),
        }
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        visit.visit_value(self.as_value());
    }
}

#[async_trait]
pub trait Database: Send + Sync {
    async fn migrate(&self) -> Result<(), ApiError>;
    fn users(&self) -> &dyn UserRepository;
    fn refresh_tokens(&self) -> &dyn RefreshTokenRepository;
}

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: User) -> Result<User, ApiError>;
    async fn find_all(&self) -> Result<Vec<User>, ApiError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApiError>;
    async fn delete_all(&self) -> Result<u64, ApiError>;
}

#[async_trait]
pub trait RefreshTokenRepository {
    async fn create(&self, refresh_token: RefreshToken) -> Result<RefreshToken, ApiError>;
    async fn delete_expired(&self) -> Result<u64, ApiError>;
    async fn delete_by_jti(&self, jti: Uuid) -> Result<Option<RefreshToken>, ApiError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{database::mock::MockDatabase, models::user::Role};

    #[tokio::test]
    async fn create_user() {
        let db = MockDatabase::new();
        let user_res = db
            .users()
            .create(User::new("test_user", "test_password", &[Role::User]))
            .await;
        assert!(user_res.is_ok());

        let user = user_res.unwrap();
        assert_eq!(user.username, "test_user");
        assert_eq!(user.password_hash, "test_password");
    }

    #[tokio::test]
    async fn create_user_username_exists() {
        let db = MockDatabase::new();
        let _ = db
            .users()
            .create(User::new("test_user", "test_password", &[Role::User]))
            .await;
        let another_user_res = db
            .users()
            .create(User::new(
                "test_user",
                "another_test_password",
                &[Role::User],
            ))
            .await;

        assert!(another_user_res.is_err());
    }

    #[tokio::test]
    async fn find_all_users_non_found() {
        let db = MockDatabase::new();
        let users_res = db.users().find_all().await;
        assert!(users_res.is_ok());
        assert!(users_res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn find_all_users() {
        let db = MockDatabase::new();
        let users_ctr = 10;

        for i in 0..users_ctr {
            let user_res = db
                .users()
                .create(User::new(
                    &format!("test_user_{}", i),
                    &format!("test_password_{}", i),
                    &[Role::User],
                ))
                .await;
            assert!(user_res.is_ok());
        }

        let users_res = db.find_all().await;
        assert!(users_res.is_ok());

        let mut users = users_res.unwrap();
        assert_eq!(users.len(), users_ctr);

        users.sort_by(|a, b| a.username.cmp(&b.username));
        for (i, user) in users.iter().enumerate() {
            assert_eq!(user.username, format!("test_user_{}", i));
        }
    }

    #[tokio::test]
    async fn find_user_by_id_not_found() {
        let db = MockDatabase::new();
        let user_res = db.users().find_by_id(Uuid::new_v4()).await;
        assert!(user_res.is_ok());
        assert!(user_res.unwrap().is_none());
    }

    #[tokio::test]
    async fn find_user_by_id() {
        let db = MockDatabase::new();
        let user_create_res = db
            .users()
            .create(User::new("test_user", "test_password", &[Role::User]))
            .await;
        assert!(user_create_res.is_ok());

        let user_create = user_create_res.unwrap();
        let user_res = db.find_by_id(user_create.id).await;
        assert!(user_res.is_ok());

        let user_opt = user_res.unwrap();
        assert!(user_opt.is_some());

        let user = user_opt.unwrap();
        assert_eq!(user.id, user_create.id);
        assert_eq!(user.username, user_create.username);
    }

    #[tokio::test]
    async fn delete_all_users() {
        let db = MockDatabase::new();
        let users_ctr = 10;

        for i in 0..users_ctr {
            let user_res = db
                .users()
                .create(User::new(
                    &format!("test_user_{}", i),
                    &format!("test_password_{}", i),
                    &[Role::User],
                ))
                .await;
            assert!(user_res.is_ok());
        }

        let deleted_count_res = db.delete_all().await;
        assert!(deleted_count_res.is_ok());
        assert_eq!(deleted_count_res.unwrap(), users_ctr);
    }
}
