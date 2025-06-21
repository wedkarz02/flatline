use std::{collections::HashMap, fmt::Display, str::FromStr};

use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    RequestPartsExt,
};
use serde::de::Error;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug)]
pub enum ApiVersion {
    V1,
    V2,
}

impl FromStr for ApiVersion {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(Self::V1),
            "v2" => Ok(Self::V2),
            v => Err(ApiError::BadRequest(format!(
                "version ({}) not supported",
                v
            ))),
        }
    }
}

impl Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
        }
    }
}

impl<'de> Deserialize<'de> for ApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ApiVersion::from_str(&s).map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

impl<S> FromRequestParts<S> for ApiVersion
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> = parts
            .extract()
            .await
            .map_err(|e| ApiError::BadRequest(format!("path rejection error: {}", e)))?;

        let version = params
            .get("version")
            .ok_or_else(|| ApiError::BadRequest("version param missing".to_string()))?;

        ApiVersion::from_str(version)
    }
}

pub struct VerIdParams {
    pub version: ApiVersion,
    pub id: Uuid,
}

impl<S> FromRequestParts<S> for VerIdParams
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path((version, id)): Path<(ApiVersion, Uuid)> = Path::from_request_parts(parts, state)
            .await
            .map_err(|rejection| {
                let msg = format!("{}", rejection);
                if msg.contains("version (") && msg.contains(") not supported") {
                    ApiError::from(rejection)
                } else {
                    ApiError::BadRequest(format!("invalid path parameters: {}", rejection))
                }
            })?;

        Ok(VerIdParams { version, id })
    }
}
