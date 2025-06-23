use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{error::ApiError, models::user::User};

use super::{Database, UserRepository};

#[derive(Debug)]
pub struct MockDatabase {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl MockDatabase {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            users: Arc::new(RwLock::new(HashMap::new())),
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
