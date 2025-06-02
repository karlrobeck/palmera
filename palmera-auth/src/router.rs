use axum::{
    Extension, Form, Json, Router,
    routing::{get, post},
};
use chrono::Duration;
use hmac::{Hmac, Mac};
use sea_query::{Alias, Asterisk, Expr, Query, SqliteQueryBuilder};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::{
    error::AuthError,
    schemas::{AuthUserSchema, JWTClaims, JWTResponsePayload, RegisterFormPayload},
};

#[derive(Debug, Deserialize)]
pub struct LoginFormPayload {
    email: String,
    password: String,
}

async fn register(
    Extension(db): Extension<Pool<Sqlite>>,
    Form(mut payload): Form<RegisterFormPayload>,
) -> Result<(), AuthError> {
    let mut trx = db.begin().await.map_err(|err| AuthError::Sqlx(err))?;

    if payload.verify_password() {
        return Err(AuthError::InvalidCredentials(
            "confirm password and password does not match".into(),
        ));
    }

    payload = payload.hash_password();

    let sql = Query::insert()
        .into_table(Alias::new("auth_users"))
        .columns([Alias::new("email"), Alias::new("password")])
        .values([payload.email.into(), payload.password.into()])
        .unwrap()
        .to_string(SqliteQueryBuilder);

    _ = sqlx::query(&sql)
        .execute(&mut *trx)
        .await
        .map_err(|err| AuthError::Sqlx(err))?;

    trx.commit().await.map_err(|err| AuthError::Sqlx(err))?;

    Ok(())
}

async fn basic_login(
    Extension(db): Extension<Pool<Sqlite>>,
    Form(payload): Form<LoginFormPayload>,
) -> Result<Json<JWTResponsePayload>, AuthError> {
    let sql = Query::select()
        .from(Alias::new("auth_users"))
        .column(Asterisk)
        .and_where(Expr::col(Alias::new("email")).eq(payload.email))
        .to_string(SqliteQueryBuilder);

    let result = sqlx::query_as::<_, AuthUserSchema>(&sql)
        .fetch_one(&db)
        .await
        .map_err(|err| AuthError::Sqlx(err))?;

    if !result.verify_password(&payload.password) {
        return Err(AuthError::InvalidCredentials("Invalid Credentials".into()));
    }

    let claims = JWTClaims::new(
        &result.email,
        "audience", // TODO: custom audience
        "issuer",
        Duration::seconds(3600),
    );

    let token = claims
        .encrypt(&Hmac::new_from_slice(b"password-123").unwrap())
        .map_err(|err| AuthError::Jwt(err))?;

    Ok(Json(JWTResponsePayload {
        access_token: token.clone(),
        refresh_token: token,
    }))
}

async fn refresh(
    Extension(db): Extension<Pool<Sqlite>>,
) -> Result<Json<JWTResponsePayload>, AuthError> {
    todo!("")
}

async fn me(Extension(db): Extension<Pool<Sqlite>>) {}

pub fn router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(basic_login))
        .route("/refresh", post(refresh))
        .route("/me", get(me))
}
