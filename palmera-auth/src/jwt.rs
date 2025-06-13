//! # JWT (JSON Web Token) utilities for Palmera Auth
//!
//! This module provides the `JWTClaims` struct and associated methods for creating,
//! signing, and verifying JWTs using HMAC-SHA256. It leverages the `chrono` crate for
//! time handling, `uuid` for unique identifiers, and `serde` for serialization.
//!
//! # Example
//!
//! ```rust
//! use palmera_auth::jwt::JWTClaims;
//! use chrono::Duration;
//! use uuid::Uuid;
//!
//! let subject = Uuid::new_v4();
//! let claims = JWTClaims::new(
//!     subject,
//!     Duration::minutes(60),
//!     "issuer".to_string(),
//!     "audience".to_string(),
//! );
//! let token = claims.clone().sign("secret").unwrap();
//! let verified = JWTClaims::verify(&token, "secret").unwrap();
//! assert_eq!(claims.subject, verified.subject);
//! ```

use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

/// Represents the standard claims contained in a JWT (JSON Web Token).
///
/// This struct is serializable and deserializable via Serde, and is compatible
/// with the standard JWT claim names via serde renaming.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JWTClaims {
    /// Subject (the user ID or entity the token refers to).
    #[serde(rename = "sub")]
    pub subject: Uuid,
    /// Expiration time (as UTC datetime).
    #[serde(rename = "exp")]
    pub expiration: DateTime<Utc>,
    /// Issued at (as UTC datetime).
    #[serde(rename = "iat")]
    pub issued_at: DateTime<Utc>,
    /// Issuer (the entity that issued the token).
    #[serde(rename = "iss")]
    pub issuer: String,
    /// Audience (the intended recipient of the token).
    #[serde(rename = "aud")]
    pub audience: String,
    /// Not before time (the time before which the token is invalid).
    #[serde(rename = "nbf")]
    pub not_before_time: DateTime<Utc>,
    /// JWT ID (unique identifier for the token).
    #[serde(rename = "jti")]
    pub jwt_token_id: Uuid,
}

impl JWTClaims {
    /// Creates a new set of JWT claims.
    ///
    /// # Arguments
    ///
    /// * `subject` - The subject (user/entity) UUID.
    /// * `exp_duration` - Duration until expiration from now.
    /// * `issuer` - The issuer string.
    /// * `audience` - The audience string.
    ///
    /// # Returns
    ///
    /// A new `JWTClaims` instance with the specified parameters and sensible defaults for issued_at, not_before_time, and jwt_token_id.
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

    /// Signs the claims and returns a JWT string using the provided secret key.
    ///
    /// # Arguments
    ///
    /// * `key` - The secret key as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if signing fails or the key is invalid.
    pub fn sign(self, key: &str) -> Result<String, anyhow::Error> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes())?;

        Ok(self.sign_with_key(&key)?)
    }

    /// Verifies a JWT string and returns the decoded claims if valid.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT string to verify.
    /// * `key` - The secret key as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails or the key is invalid.
    pub fn verify(token: &str, key: &str) -> Result<Self, anyhow::Error> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes())?;

        let claims: JWTClaims = token.verify_with_key(&key)?;

        let now = Utc::now();

        // Check if the token is expired
        if now > claims.expiration {
            return Err(anyhow::anyhow!("Token expired"));
        }

        // Check if the token is not yet valid
        if now < claims.not_before_time {
            return Err(anyhow::anyhow!("Token not yet valid"));
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use sqlx::types::Uuid;

    const SECRET: &str = "supersecretkey";

    #[test]
    fn test_jwt_sign_and_verify_success() {
        let subject = Uuid::new_v4();
        let claims = JWTClaims::new(
            subject,
            Duration::minutes(10),
            "issuer".to_string(),
            "audience".to_string(),
        );
        let token = claims.clone().sign(SECRET).expect("signing failed");
        let verified = JWTClaims::verify(&token, SECRET).expect("verification failed");
        assert_eq!(claims, verified);
    }

    #[test]
    fn test_jwt_expired_token() {
        let subject = Uuid::new_v4();
        let claims = JWTClaims::new(
            subject,
            Duration::seconds(-1), // already expired
            "issuer".to_string(),
            "audience".to_string(),
        );
        let token = claims.sign(SECRET).expect("signing failed");
        let err = JWTClaims::verify(&token, SECRET).unwrap_err();
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn test_jwt_not_yet_valid_token() {
        let subject = Uuid::new_v4();
        let mut claims = JWTClaims::new(
            subject,
            Duration::minutes(10),
            "issuer".to_string(),
            "audience".to_string(),
        );
        // Set not_before_time to 10 minutes in the future
        claims.not_before_time = Utc::now() + Duration::minutes(10);
        let token = claims.sign(SECRET).expect("signing failed");
        let err = JWTClaims::verify(&token, SECRET).unwrap_err();
        assert!(err.to_string().contains("not yet valid"));
    }

    #[test]
    fn test_jwt_invalid_signature() {
        let subject = Uuid::new_v4();
        let claims = JWTClaims::new(
            subject,
            Duration::minutes(10),
            "issuer".to_string(),
            "audience".to_string(),
        );
        let token = claims.sign(SECRET).expect("signing failed");
        // Use a different key for verification
        let err = JWTClaims::verify(&token, "wrongkey").unwrap_err();

        println!("{}", err);

        assert!(err.to_string().to_lowercase().contains("mismatch"));
    }

    #[test]
    fn test_jwt_fields_roundtrip() {
        let subject = Uuid::new_v4();
        let issuer = "issuer-test".to_string();
        let audience = "audience-test".to_string();
        let claims = JWTClaims::new(
            subject,
            Duration::minutes(5),
            issuer.clone(),
            audience.clone(),
        );
        let token = claims.clone().sign(SECRET).expect("signing failed");
        let verified = JWTClaims::verify(&token, SECRET).expect("verification failed");
        assert_eq!(verified.subject, subject);
        assert_eq!(verified.issuer, issuer);
        assert_eq!(verified.audience, audience);
        assert_eq!(verified.jwt_token_id, claims.jwt_token_id);
    }
}
