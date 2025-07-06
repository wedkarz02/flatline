use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use axum::http::StatusCode;

use crate::{
    error::ApiError,
    models::user::{Role, User},
    routes::auth::AuthPayload,
    services::jwt::pairs_from_user,
    ApiState,
};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Token is invalid")]
    TokenInvalid,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Username already taken")]
    UsernameAlreadyTaken,
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
            AuthError::Forbidden => StatusCode::FORBIDDEN,
            AuthError::TokenInvalid => StatusCode::UNAUTHORIZED,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthError::UsernameAlreadyTaken => StatusCode::CONFLICT,
        }
    }
}

pub fn hash_string(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_hash(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed_hash| Argon2::default().verify_password(password.as_bytes(), &parsed_hash))
        .is_ok_and(|res| res.is_ok())
}

pub async fn register(
    state: &Arc<ApiState>,
    auth_payload: AuthPayload,
    roles: &[Role],
) -> Result<User, ApiError> {
    if state
        .db
        .users()
        .find_by_username(&auth_payload.username)
        .await?
        .is_some()
    {
        return Err(AuthError::UsernameAlreadyTaken.into());
    }

    let new_user = User::new(
        &auth_payload.username,
        &hash_string(&auth_payload.password),
        roles,
    );

    let created_user = state.db.users().create(new_user).await?;
    Ok(created_user)
}

pub async fn login(
    state: &Arc<ApiState>,
    auth_payload: AuthPayload,
) -> Result<(String, String), ApiError> {
    let user = state
        .db
        .users()
        .find_by_username(&auth_payload.username)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !verify_hash(&user.password_hash, &auth_payload.password) {
        return Err(AuthError::InvalidCredentials.into());
    }

    let (access_token, refresh_token, token_model) = pairs_from_user(
        &user,
        state.config.jwt_access_expiration,
        state.config.jwt_refresh_expiration,
        &state.config.jwt_access_secret,
        &state.config.jwt_refresh_secret,
    )?;

    let _ = state.db.refresh_tokens().create(token_model).await?;
    Ok((access_token, refresh_token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_hash_correct() {
        let password = "test_password";
        let hash = hash_string(password);

        assert!(verify_hash(&hash, password));
    }

    #[test]
    fn verify_hash_failed() {
        let password = "test_password";
        let hash = hash_string(password);

        assert!(!verify_hash(&hash, "wrong_password"));
    }
}
