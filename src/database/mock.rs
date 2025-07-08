use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    database::RefreshTokenRepository,
    error::ApiError,
    models::{refresh_token::RefreshToken, user::User},
};

use super::{Database, UserRepository};

#[derive(Debug)]
pub struct MockDatabase {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    refresh_tokens: Arc<RwLock<HashMap<Uuid, RefreshToken>>>,
}

impl MockDatabase {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl Database for MockDatabase {
    async fn migrate(&self) -> Result<(), ApiError> {
        tracing::warn!("no migrations needed for in-memory mock database");
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
impl UserRepository for MockDatabase {
    async fn create(&self, user: User) -> Result<User, ApiError> {
        for u in self.users.read().unwrap().values() {
            if u.username == user.username {
                return Err(ApiError::Internal(anyhow::Error::msg(
                    "username already exists",
                )));
            }
        }

        self.users.write().unwrap().insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_all(&self) -> Result<Vec<User>, ApiError> {
        let users: Vec<User> = self.users.read().unwrap().values().cloned().collect();
        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError> {
        let user = self.users.read().unwrap().get(&id).cloned();
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApiError> {
        let user = self
            .users
            .read()
            .unwrap()
            .values()
            .find(|user| user.username == username)
            .cloned();

        Ok(user)
    }

    async fn delete_all(&self) -> Result<u64, ApiError> {
        let mut users = self.users.write().unwrap();
        let deleted_count = users.len();
        users.clear();
        Ok(deleted_count as u64)
    }
}

#[async_trait]
impl RefreshTokenRepository for MockDatabase {
    async fn create(&self, refresh_token: RefreshToken) -> Result<RefreshToken, ApiError> {
        self.refresh_tokens
            .write()
            .unwrap()
            .insert(refresh_token.jti, refresh_token.clone());
        Ok(refresh_token)
    }

    async fn delete_expired(&self) -> Result<u64, ApiError> {
        let now = chrono::Utc::now().timestamp();

        let mut tokens = self.refresh_tokens.write().unwrap();
        let original_count = tokens.len();

        tokens.retain(|_, tok| tok.exp > now);
        Ok((original_count - tokens.len()) as u64)
    }

    async fn delete_by_jti(&self, jti: Uuid) -> Result<Option<RefreshToken>, ApiError> {
        let mut tokens = self.refresh_tokens.write().unwrap();
        Ok(tokens.remove_entry(&jti).map(|(_, tok)| tok))
    }
}
