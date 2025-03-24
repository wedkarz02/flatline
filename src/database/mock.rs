use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::User;

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
    async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        tracing::warn!("no migrations needed for in-memory mock database");
        Ok(())
    }

    fn users(&self) -> &dyn UserRepository {
        self
    }
}

#[async_trait]
impl UserRepository for MockDatabase {
    async fn create(&self, username: &str, password: &str) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let user = User {
            id,
            username: username.to_owned(),
            password: password.to_owned(),
        };

        self.users.write().unwrap().insert(id, user.clone());
        Ok(user)
    }

    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        let users: Vec<User> = self.users.read().unwrap().values().cloned().collect();
        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = self.users.read().unwrap().get(&id).cloned();
        Ok(user)
    }

    async fn delete_all(&self) -> Result<u64, sqlx::Error> {
        let mut users = self.users.write().unwrap();
        let deleted_count = users.len();
        users.clear();
        Ok(deleted_count as u64)
    }
}
