use sqlx::{Pool, Postgres};

pub mod jwt;
pub mod schemas;

pub async fn migrate(db: &Pool<Postgres>) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(db).await?)
}
