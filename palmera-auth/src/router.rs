use axum::{Extension, Form, http::StatusCode};
use chrono::Duration;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{AuthConfig, jwt::JWTClaims, schemas::AuthUser};

#[derive(Debug, ToSchema, Deserialize, Validate)]
pub struct LoginPayload {
    #[validate(email)]
    email: String,
    password: String,
}

#[utoipa::path(post, path = "/login")]
async fn login(
    Extension(db): Extension<Pool<Postgres>>,
    Extension(config): Extension<AuthConfig>,
    Form(form): Form<LoginPayload>,
) -> Result<String, StatusCode> {
    if form.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let db_user = AuthUser::find_by_email(&form.email, &db)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if db_user.verify_password(&form.password).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let claims = JWTClaims::new(
        db_user.id,
        Duration::seconds(3600),
        config.issuer,
        config.audience,
    );

    Ok(claims
        .sign(&config.key)
        .map_err(|_| StatusCode::BAD_REQUEST)?)
}

pub fn router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(login))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthConfig;
    use crate::jwt::JWTClaims;
    use crate::schemas::AuthUser;
    use axum::Form;
    use axum::extract::Extension;
    use chrono::Utc;
    use sqlx::{Pool, Postgres};

    fn test_config() -> AuthConfig {
        AuthConfig {
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            key: "test-secret-key".to_string(),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_login_success(db: Pool<Postgres>) -> anyhow::Result<()> {
        let email = "loginuser@example.com";
        let password = "testpassword";
        let user = AuthUser::new(email, password);
        user.clone().insert(&db).await?;
        let config = test_config();
        let payload = LoginPayload {
            email: email.to_string(),
            password: password.to_string(),
        };
        let result = login(Extension(db), Extension(config), Form(payload)).await;
        assert!(
            result.is_ok(),
            "Login should succeed with correct credentials"
        );
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_login_wrong_password(db: Pool<Postgres>) -> anyhow::Result<()> {
        let email = "wrongpass@example.com";
        let password = "rightpass";
        let user = AuthUser::new(email, password);
        user.clone().insert(&db).await?;
        let config = test_config();
        let payload = LoginPayload {
            email: email.to_string(),
            password: "wrongpass".to_string(),
        };
        let result = login(Extension(db), Extension(config), Form(payload)).await;
        assert!(result.is_err(), "Login should fail with wrong password");
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_login_nonexistent_email(db: Pool<Postgres>) -> anyhow::Result<()> {
        let config = test_config();
        let payload = LoginPayload {
            email: "doesnotexist@example.com".to_string(),
            password: "irrelevant".to_string(),
        };
        let result = login(Extension(db), Extension(config), Form(payload)).await;
        assert!(result.is_err(), "Login should fail for nonexistent user");
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_jwt_structure(db: Pool<Postgres>) -> anyhow::Result<()> {
        let email = "jwtstruct@example.com";
        let password = "jwtpass";
        let user = AuthUser::new(email, password);
        user.clone().insert(&db).await?;
        let config = test_config();
        let payload = LoginPayload {
            email: email.to_string(),
            password: password.to_string(),
        };
        let result = login(Extension(db), Extension(config), Form(payload))
            .await
            .unwrap();
        let parts: Vec<&str> = result.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts");
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_jwt_claims(db: Pool<Postgres>) -> anyhow::Result<()> {
        let email = "jwtclaims@example.com";
        let password = "jwtpass";
        let user = AuthUser::new(email, password);
        let inserted = user.clone().insert(&db).await?;
        let config = test_config();
        let payload = LoginPayload {
            email: email.to_string(),
            password: password.to_string(),
        };
        let jwt = login(Extension(db), Extension(config.clone()), Form(payload))
            .await
            .unwrap();
        let claims = JWTClaims::verify(&jwt, &config.key)?;

        assert_eq!(claims.subject, inserted.id);
        let now = Utc::now();
        assert!(claims.expiration > now, "exp should be in the future");
        Ok(())
    }
}
