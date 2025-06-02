pub mod error;
pub mod router;
pub mod schemas;

use sqlx::{Pool, Sqlite};

pub async fn migrate(db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    Ok(sqlx::migrate!("./migrations").run(db).await?)
}
