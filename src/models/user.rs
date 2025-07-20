use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

                Role::from_str(trimmed).ok()
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub roles: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            roles: user.roles.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl From<&User> for UserDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            roles: user.roles.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_string_from_vec() {
        let roles = vec![Role::User, Role::Admin];
        let role_string = Role::from_vec(&roles);

        assert_eq!("user,admin", role_string);
    }

    #[test]
    fn roles_from_string() {
        let role_string = "user,admin";
        let roles = Role::to_vec(role_string);

        assert_eq!(vec![Role::User, Role::Admin], roles);
    }

    #[test]
    fn user_has_role() {
        let user = User::new("test_user", "test_hash", &[Role::User]);

        assert!(user.has_role(Role::User));
        assert!(!user.has_role(Role::Admin));
    }
}
