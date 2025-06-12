use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::types::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JWTClaims {
    #[serde(rename = "sub")]
    subject: Uuid,
    #[serde(rename = "exp")]
    expiration: DateTime<Utc>,
    #[serde(rename = "iat")]
    issued_at: DateTime<Utc>,
    #[serde(rename = "iss")]
    issuer: String,
    #[serde(rename = "aud")]
    audience: String,
    #[serde(rename = "nbf")]
    not_before_time: DateTime<Utc>,
    #[serde(rename = "jti")]
    jwt_token_id: Uuid,
}

impl JWTClaims {
    pub fn new(subject: Uuid, exp_duration: Duration, issuer: String, audience: String) -> Self {
        let now = Utc::now();

        Self {
            subject,
            expiration: now + exp_duration,
            issued_at: now,
            issuer,
            audience,
            not_before_time: now - Duration::milliseconds(250),
            jwt_token_id: Uuid::new_v4(),
        }
    }

    pub fn sign(self, key: &str) -> Result<String, anyhow::Error> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes())?;

        Ok(self.sign_with_key(&key)?)
    }

    pub fn verify(token: &str, key: &str) -> Result<Self, anyhow::Error> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes())?;

        Ok(token.verify_with_key(&key)?)
    }
}
