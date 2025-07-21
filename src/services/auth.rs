use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::refresh_token::RefreshToken,
    routes::auth::AuthPayload,
    services::{
        self,
        jwt::{pairs_from_user, Claims},
    },
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

pub fn hash_string(plain: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_hash(hash: &str, plain: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed_hash| Argon2::default().verify_password(plain.as_bytes(), &parsed_hash))
        .is_ok_and(|res| res.is_ok())
}

pub async fn login(
    state: &Arc<ApiState>,
    auth_payload: AuthPayload,
) -> Result<(String, String, Option<RefreshToken>), ApiError> {
    let user = state
        .db
        .users()
        .find_by_username(&auth_payload.username)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !verify_hash(&user.password_hash, &auth_payload.password) {
        return Err(AuthError::InvalidCredentials.into());
    }

    // This is for session limiting that prevents the user from the 'login spam'.
    // If the user has 'user_session_limit' or more refresh tokens in the DB, remove
    // the oldest one before issuing a new token.
    // NOTE: This will log out the device that uses the oldest token - 'user_session_limit' value
    //       propably should be at least 5.
    let deleted_token = services::jwt::revoke_oldest_token(state, user.id).await?;

    let (access_token, refresh_token, token_model) = pairs_from_user(
        &user,
        state.config.jwt_access_expiration,
        state.config.jwt_refresh_expiration,
        &state.config.jwt_access_secret,
        &state.config.jwt_refresh_secret,
    )?;

    let _ = state.db.refresh_tokens().create(token_model).await?;
    Ok((access_token, refresh_token, deleted_token))
}

// FIXME: Once some sort of token blacklisting is setup, blacklist the access token here.
pub async fn refresh(
    state: &Arc<ApiState>,
    refresh_token: &str,
    _access_jti: &Uuid,
) -> Result<String, ApiError> {
    let claims = services::jwt::decode_token(refresh_token, &state.config.jwt_refresh_secret)?;

    if state
        .db
        .refresh_tokens()
        .find_by_jti(claims.jti)
        .await?
        .is_none()
    {
        return Err(AuthError::TokenInvalid.into());
    }

    let now = chrono::Utc::now();
    let access_claims = Claims::new(
        claims.sub,
        now.checked_add_signed(chrono::Duration::seconds(
            state.config.jwt_refresh_expiration,
        ))
        .unwrap()
        .timestamp(),
        now.timestamp(),
        Uuid::new_v4(),
        claims.username,
        claims.roles,
        claims.admin,
    );
    let access_token =
        services::jwt::generate_token(&access_claims, &state.config.jwt_access_secret)?;

    Ok(access_token)
}

// FIXME: Once some sort of token blacklisting is setup, blacklist the access token here.
pub async fn logout(
    state: &Arc<ApiState>,
    refresh_token: &str,
    _access_jti: &Uuid,
) -> Result<Option<Uuid>, ApiError> {
    let claims = services::jwt::decode_token(refresh_token, &state.config.jwt_refresh_secret)?;
    let deleted_token = state.db.refresh_tokens().delete_by_jti(claims.jti).await?;

    if let Some(token) = deleted_token {
        return Ok(Some(token.jti));
    }

    Ok(None)
}

// TODO: This should include some sort of token blacklist checks.
pub async fn auth_guard(
    Extension(state): Extension<Arc<ApiState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let access_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AuthError::Unauthorized)?;

    let claims = services::jwt::decode_token(access_token, &state.config.jwt_access_secret)?;
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
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
