use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub jti: Uuid,
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_hash: String,
}

impl RefreshToken {
    pub fn new(jti: Uuid, sub: Uuid, exp: i64, iat: i64, token_hash: String) -> Self {
        Self {
            jti,
            sub,
            exp,
            iat,
            token_hash,
        }
    }
}
