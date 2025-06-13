use sqlx::{Pool, Postgres};

pub mod jwt;
pub mod router;
pub mod schemas;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    issuer: String,
    audience: String,
    key: String,
}

pub async fn migrate(db: &Pool<Postgres>) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(db).await?)
}
