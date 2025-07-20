use std::sync::Arc;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{
        refresh_token::RefreshToken,
        user::{Role, User},
    },
    services::{self, auth::AuthError},
    ApiState,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub jti: Uuid,
    pub username: String,
    pub roles: String,
    pub admin: bool,
}

impl Claims {
    pub fn new(
        sub: Uuid,
        exp: i64,
        iat: i64,
        jti: Uuid,
        username: String,
        roles: String,
        admin: bool,
    ) -> Self {
        Self {
            sub,
            exp,
            iat,
            jti,
            username,
            roles,
            admin,
        }
    }

    pub fn from_user(user: &User, exp: i64, iat: i64) -> Self {
        Self {
            sub: user.id,
            exp,
            iat,
            jti: Uuid::new_v4(),
            username: user.username.to_owned(),
            roles: user.roles.to_owned(),
            admin: user.has_role(Role::Admin),
        }
    }
}

pub fn generate_token(claims: &Claims, secret: &str) -> Result<String, ApiError> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, ApiError> {
    let token_data = jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| match err.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::TokenInvalid,
    })?;

    Ok(token_data.claims)
}

pub fn pairs_from_user(
    user: &User,
    aexp: i64,
    rexp: i64,
    asecret: &str,
    rsecret: &str,
) -> Result<(String, String, RefreshToken), ApiError> {
    let now = chrono::Utc::now();

    let access_claims = Claims::from_user(
        user,
        now.checked_add_signed(chrono::Duration::seconds(aexp))
            .unwrap()
            .timestamp(),
        now.timestamp(),
    );

    let refresh_claims = Claims::from_user(
        user,
        now.checked_add_signed(chrono::Duration::seconds(rexp))
            .unwrap()
            .timestamp(),
        now.timestamp(),
    );

    let access_token = generate_token(&access_claims, asecret)?;
    let refresh_token = generate_token(&refresh_claims, rsecret)?;

    let token_model = RefreshToken::new(
        refresh_claims.jti,
        refresh_claims.sub,
        refresh_claims.exp,
        refresh_claims.iat,
        services::auth::hash_string(&refresh_token),
    );

    Ok((access_token, refresh_token, token_model))
}

pub async fn revoke_oldest_token(
    state: &Arc<ApiState>,
    sub: Uuid,
) -> Result<Option<RefreshToken>, ApiError> {
    let tokens = state.db.refresh_tokens().find_by_sub(sub).await?;

    if tokens.len() < state.config.user_session_limit {
        return Ok(None);
    }

    let oldest_token = tokens
        .iter()
        .min_by_key(|tok| tok.iat)
        .expect("vec will never be empty due to the if above");

    let deleted_token = state
        .db
        .refresh_tokens()
        .delete_by_jti(oldest_token.jti)
        .await?;

    Ok(deleted_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCES_EXP_MIN: chrono::TimeDelta = chrono::Duration::minutes(15);
    const REFRE_EXP_MIN: chrono::TimeDelta = chrono::Duration::days(30);

    #[test]
    fn generate_jwt_token() {
        let now = chrono::Utc::now();
        let claims = Claims {
            sub: Uuid::new_v4(),
            exp: now.checked_add_signed(ACCES_EXP_MIN).unwrap().timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
            username: "test_user".to_owned(),
            roles: "user,admin".to_owned(),
            admin: true,
        };

        let token = generate_token(&claims, "test_secret");
        assert!(token.is_ok());
    }

    #[test]
    fn decode_jwt_token_correct() {
        let now = chrono::Utc::now();
        let claims = Claims {
            sub: Uuid::new_v4(),
            exp: now.checked_add_signed(REFRE_EXP_MIN).unwrap().timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
            username: "test_user".to_owned(),
            roles: "user,admin".to_owned(),
            admin: true,
        };

        let token = generate_token(&claims, "another_test_secret");
        assert!(token.is_ok());

        let result = decode_token(&token.unwrap(), "another_test_secret");
        assert!(result.is_ok());
        assert_eq!(claims, result.unwrap());
    }

    #[test]
    fn decode_invalid_jwt_token_failed() {
        let token = "invalid_token";
        let result = decode_token(&token, "bad_jwt_secret");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApiError::Auth(AuthError::TokenInvalid))
        ));
    }

    #[test]
    fn decode_expired_jwt_token_failed() {
        let now = chrono::Utc::now();
        let claims = Claims {
            sub: Uuid::new_v4(),
            exp: now.checked_sub_signed(REFRE_EXP_MIN).unwrap().timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
            username: "test_user".to_owned(),
            roles: "user,admin".to_owned(),
            admin: true,
        };

        let token = generate_token(&claims, "test_secret");
        assert!(token.is_ok());

        let result = decode_token(&token.unwrap(), "test_secret");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApiError::Auth(AuthError::TokenExpired))
        ));
    }
}
