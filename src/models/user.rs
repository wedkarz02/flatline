use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Role {
    User,
    Admin,
}

impl FromStr for Role {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Self::User),
            "admin" => Ok(Self::Admin),
            val => Err(ApiError::BadRequest(format!(
                "role ({}) not recognized",
                val
            ))),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Admin => write!(f, "admin"),
        }
    }
}

impl Role {
    pub fn from_vec(roles: &[Self]) -> String {
        roles
            .iter()
            .map(|role| role.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }

    pub fn to_vec(roles: &str) -> Vec<Self> {
        roles
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return None;
                }

                match Role::from_str(trimmed) {
                    Ok(role) => Some(role),
                    Err(_) => None,
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub roles: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: &str, password_hash: &str, roles: &[Role]) -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            username: username.to_owned(),
            password_hash: password_hash.to_owned(),
            roles: Role::from_vec(roles),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role.to_string())
    }
}
