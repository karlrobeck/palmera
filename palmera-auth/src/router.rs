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
