//! # Palmera Auth: User Authentication Schema and Utilities
//!
//! This module provides the [`AuthUser`] struct, representing a user in the Palmera authentication system.
//! It includes secure password hashing and verification using Argon2, as well as database operations
//! for inserting and retrieving users. All timestamps are handled in UTC.
//!
//! ## Features
//!
//! - Secure password storage with Argon2 and random salt
//! - Password verification
//! - Insert and query users from a PostgreSQL database
//!
//! ## Example
//!
//! ```rust
//! use palmera_auth::schemas::AuthUser;
//! let user = AuthUser::new("user@example.com", "password123");
//! assert!(user.verify_password("password123").is_ok());
//! ```

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use password_hash::{SaltString, rand_core::OsRng};
use sea_query::{Alias, Asterisk, Expr, PostgresQueryBuilder, Query};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
/// Represents an authenticated user in the Palmera system.
///
/// Fields:
/// - `id`: Unique identifier (UUID)
/// - `email`: User's email address
/// - `password`: Argon2-hashed password (with salt and parameters)
/// - `created`: UTC timestamp of creation
/// - `updated`: UTC timestamp of last update
pub struct AuthUser {
    /// Unique identifier for the user (UUID).
    pub id: Uuid,
    /// User's email address.
    pub email: String,
    /// Argon2-hashed password (including salt and parameters).
    pub password: String,
    /// Timestamp of when the user was created (UTC).
    pub created: DateTime<Utc>,
    /// Timestamp of when the user was last updated (UTC).
    pub updated: DateTime<Utc>,
}

impl AuthUser {
    /// Create a new `AuthUser` with a securely hashed password and generated salt.
    ///
    /// # Arguments
    ///
    /// * `email` - The user's email address.
    /// * `password` - The user's plaintext password.
    ///
    /// # Returns
    ///
    /// A new `AuthUser` instance with a hashed password and current timestamps.
    pub fn new(email: &str, password: &str) -> Self {
        let now = Utc::now();

        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        Self {
            id: Uuid::new_v4(),
            email: email.to_string(),
            password: argon2
                .hash_password(password.as_bytes(), &salt)
                .unwrap()
                .to_string(),
            created: now,
            updated: now,
        }
    }

    /// Verify a plaintext password against the stored Argon2 hash.
    ///
    /// # Arguments
    ///
    /// * `password` - The plaintext password to verify.
    ///
    /// # Errors
    ///
    /// Returns an error if the password does not match or the hash is invalid.
    pub fn verify_password(&self, password: &str) -> anyhow::Result<()> {
        let pw_hash = PasswordHash::new(&self.password).unwrap();

        Argon2::default()
            .verify_password(password.as_bytes(), &pw_hash)
            .map_err(|_| anyhow::anyhow!("Password not match"))
    }

    // database operation

    /// Insert this `AuthUser` into the database and return the inserted user.
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to a SQLx Postgres connection pool.
    ///
    /// # Returns
    ///
    /// The inserted `AuthUser` as stored in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn insert(self, db: &Pool<Postgres>) -> anyhow::Result<Self> {
        let sql = Query::insert()
            .into_table((Alias::new("auth"), Alias::new("users")))
            .columns([
                Alias::new("id"),
                Alias::new("email"),
                Alias::new("password"),
                Alias::new("created"),
                Alias::new("updated"),
            ])
            .values([
                self.id.into(),
                self.email.into(),
                self.password.into(),
                self.created.into(),
                self.updated.into(),
            ])?
            .returning_all()
            .to_string(PostgresQueryBuilder);

        let result = sqlx::query_as::<_, Self>(&sql).fetch_one(db).await?;

        Ok(result)
    }

    /// Find an `AuthUser` by their unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The user's unique identifier as a string.
    /// * `db` - Reference to a SQLx Postgres connection pool.
    ///
    /// # Returns
    ///
    /// The `AuthUser` if found, or an error if not found or if the database operation fails.
    pub async fn find_by_id(id: &str, db: &Pool<Postgres>) -> anyhow::Result<Self> {
        let sql = Query::select()
            .from((Alias::new("auth"), Alias::new("users")))
            .column(Asterisk)
            .and_where(Expr::col("id").eq(id))
            .to_string(PostgresQueryBuilder);

        let result = sqlx::query_as::<_, Self>(&sql).fetch_one(db).await?;

        Ok(result)
    }

    /// Find an `AuthUser` by their email address.
    ///
    /// # Arguments
    ///
    /// * `email` - The user's email address.
    /// * `db` - Reference to a SQLx Postgres connection pool.
    ///
    /// # Returns
    ///
    /// The `AuthUser` if found, or an error if not found or if the database operation fails.
    pub async fn find_by_email(email: &str, db: &Pool<Postgres>) -> anyhow::Result<Self> {
        let sql = Query::select()
            .from((Alias::new("auth"), Alias::new("users")))
            .column(Asterisk)
            .and_where(Expr::col("email").eq(email))
            .to_string(PostgresQueryBuilder);

        let result = sqlx::query_as::<_, Self>(&sql).fetch_one(db).await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Pool, Postgres};

    #[test]
    fn test_new_user_and_verify_password_success() {
        let email = "test@example.com";
        let password = "securepassword";
        let user = AuthUser::new(email, password);
        assert_eq!(user.email, email);
        assert!(user.verify_password(password).is_ok());
    }

    #[test]
    fn test_verify_password_failure() {
        let user = AuthUser::new("user@example.com", "correct_password");
        let result = user.verify_password("wrong_password");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Password not match")
        );
    }

    #[test]
    fn test_password_is_hashed() {
        let password = "mysecret";
        let user = AuthUser::new("user@example.com", password);
        // The stored password should not be the plaintext
        assert_ne!(user.password, password);
        // The hash should start with the Argon2 prefix
        assert!(user.password.starts_with("$argon2"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_insert_and_find_by_id_and_email(db: Pool<Postgres>) -> anyhow::Result<()> {
        let email = "findme@example.com";
        let password = "findmypassword";
        let user = AuthUser::new(email, password);
        // Insert user
        let inserted = user.clone().insert(&db).await?;
        assert_eq!(inserted.email, email);
        // Find by id
        let found_by_id = AuthUser::find_by_id(&inserted.id.to_string(), &db).await?;
        assert_eq!(found_by_id.email, email);
        // Find by email
        let found_by_email = AuthUser::find_by_email(email, &db).await?;
        assert_eq!(found_by_email.id, inserted.id);
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_id_not_found(db: Pool<Postgres>) -> anyhow::Result<()> {
        let result = AuthUser::find_by_id("00000000-0000-0000-0000-000000000000", &db).await;
        assert!(result.is_err());
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_email_not_found(db: Pool<Postgres>) -> anyhow::Result<()> {
        let result = AuthUser::find_by_email("notfound@example.com", &db).await;
        assert!(result.is_err());
        Ok(())
    }
}
