use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{error::AppError, models::user::User};

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
    async fn migrate(&self) -> Result<(), AppError> {
        tracing::warn!("no migrations needed for in-memory mock database");
        Ok(())
    }

    fn users(&self) -> &dyn UserRepository {
        self
    }
}

#[async_trait]
impl UserRepository for MockDatabase {
    async fn create(&self, username: &str, password: &str) -> Result<User, AppError> {
        let id = Uuid::new_v4();
        let new_user = User {
            id,
            username: username.to_owned(),
            password: password.to_owned(),
        };

        for user in self.users.read().unwrap().values() {
            if user.username == new_user.username {
                return Err(AppError::Internal(anyhow::Error::msg(
                    "username already exists",
                )));
            }
        }

        self.users.write().unwrap().insert(id, new_user.clone());
        Ok(new_user)
    }

    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        let users: Vec<User> = self.users.read().unwrap().values().cloned().collect();
        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let user = self.users.read().unwrap().get(&id).cloned();
        Ok(user)
    }

    async fn delete_all(&self) -> Result<u64, AppError> {
        let mut users = self.users.write().unwrap();
        let deleted_count = users.len();
        users.clear();
        Ok(deleted_count as u64)
    }
}
