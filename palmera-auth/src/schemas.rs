use chrono::{DateTime, Duration, Utc};
use hmac::Hmac;
use jwt::{SignWithKey, VerifyWithKey};
use password_auth::{generate_hash, verify_password};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct AuthUserSchema {
    pub id: Uuid,
    pub email: String,
    #[serde(skip)]
    password: String,
    created: String,
    updated: String,
}

impl AuthUserSchema {
    pub fn verify_password(&self, password: &str) -> bool {
        verify_password(password, &self.password).is_ok()
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterFormPayload {
    pub email: String,
    pub password: String,
    confirm_password: String,
}

impl RegisterFormPayload {
    pub fn verify_password(&self) -> bool {
        self.password == self.confirm_password
    }

    pub fn hash_password(mut self) -> Self {
        self.password = generate_hash(self.password);
        self
    }
}

#[derive(Debug, Serialize)]
pub struct JWTResponsePayload {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    sub: String,
    aud: String,
    exp: DateTime<Utc>,
    iat: DateTime<Utc>,
    nbf: DateTime<Utc>,
    jti: Uuid,
    iss: String,
}

impl JWTClaims {
    pub fn new(sub: &str, aud: &str, iss: &str, exp_duration: Duration) -> Self {
        let now = Utc::now().to_utc();
        Self {
            sub: sub.to_string(),
            aud: aud.to_string(),
            exp: now + exp_duration,
            iat: now,
            nbf: now - Duration::milliseconds(300),
            jti: Uuid::new_v4(),
            iss: iss.to_string(),
        }
    }

    pub fn encrypt(self, key: &Hmac<Sha256>) -> Result<String, jwt::Error> {
        Ok(self.sign_with_key(key)?)
    }

    pub fn decrypt(
        token: &str,
        key: &Hmac<Sha256>,
        aud: &str,
        iss: &str,
    ) -> Result<Self, jwt::Error> {
        let claims: Self = token.verify_with_key(key).unwrap();

        let now = Utc::now().to_utc();

        if claims.exp > now {
            todo!("return error")
        }

        if claims.nbf < now {
            todo!("return error")
        }

        if claims.aud != aud {
            todo!("return error")
        }

        if claims.iss != iss {
            todo!("return error")
        }

        Ok(claims)
    }
}
