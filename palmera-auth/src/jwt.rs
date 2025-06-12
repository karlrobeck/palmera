use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JWTClaims {
    #[serde(rename = "sub")]
    subject: Uuid,
    #[serde(rename = "expiration")]
    expiration: DateTime<Utc>,
    #[serde(rename = "iat")]
    issued_at: DateTime<Utc>,
    #[serde(rename = "issuer")]
    issuer: String,
    #[serde(rename = "aud")]
    audience: String,
    #[serde(rename = "nbf")]
    not_before_time: DateTime<Utc>,
    #[serde(rename = "jti")]
    jwt_token_id: Uuid,
}
