use std::collections::HashMap;

use axum::{
    Extension, Json,
    extract::{Multipart, Path},
    http::StatusCode,
};
use sea_query::{Alias, Expr, Func, PostgresQueryBuilder, Query, QueryStatementWriter, SimpleExpr};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Executor, Pool, Postgres, prelude::FromRow};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::utils::json_to_sea;

#[derive(Debug, Deserialize)]
pub struct TableSchema {
    schema: String,
    table: String,
}

#[derive(Debug, FromRow)]
pub struct TableResult {
    data: serde_json::Value,
}

#[utoipa::path(post, path = "/{schema}/{table}")]
async fn create(
    Extension(db): Extension<Pool<Postgres>>,
    Path(table_ref): Path<TableSchema>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut trx = db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut fields = HashMap::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if let Some(file_name) = field.file_name() {
            todo!("upload to minio");
        }

        let name = field.name().ok_or(StatusCode::BAD_REQUEST)?.to_string();
        let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        fields.insert(
            name,
            serde_json::to_value(value).map_err(|_| StatusCode::BAD_REQUEST)?,
        );
    }

    let (columns, values): (Vec<_>, Vec<_>) = fields
        .into_iter()
        .map(|(k, v)| (Alias::new(k), SimpleExpr::Value(json_to_sea(&v))))
        .unzip();

    let sql = Query::insert()
        .into_table((
            Alias::new(table_ref.schema),
            Alias::new(table_ref.table.clone()),
        ))
        .columns(columns)
        .values(values)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .returning(Query::returning().expr(Expr::cust(format!(
            "row_to_json({}.*) as data",
            table_ref.table
        ))))
        .to_string(PostgresQueryBuilder);

    let result = sqlx::query_as::<_, TableResult>(&sql)
        .fetch_one(&mut *trx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    _ = trx
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result.data))
}

pub fn router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(create))
}
