use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, services::auth::AuthError};

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
