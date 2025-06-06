use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TableDetails {
    pub name: String,
    pub r#type: Option<String>,
    pub schema: Option<String>,
    pub sql: Option<String>,
    #[sqlx(json)]
    pub policies: Vec<Policy>,
    #[sqlx(json)]
    pub columns: Vec<ColumnDetails>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Policy {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_enabled: i16,
    pub operation: Option<String>,
    pub policy_type: Option<String>,
    pub using_expr: Option<String>,
    pub check_expr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ColumnDetails {
    pub column_id: Option<i64>,
    pub column_name: String,
    pub data_type: String,
    pub is_not_null: i16,
    pub default_value: Option<String>,
    pub is_primary_key: i16,
    pub primary_key_order: Option<i64>,
    pub generated_column_type: Option<i64>,
    pub is_foreign_key: i16,
    pub reference_table: Option<String>,
    pub reference_column: Option<String>,
    pub foreign_key_on_update: Option<String>,
    pub foreign_key_on_delete: Option<String>,
    pub part_of_index: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct TableOutput {
    #[sqlx(json)]
    pub table_details: TableDetails,
}

pub async fn get_table_info(db: &Pool<Sqlite>, name: &str) -> Result<TableOutput, sqlx::Error> {
    let sql = r#"
    SELECT
      json_object(
            'name', m.name,
            'type', m.type,
            'schema', 'main', -- The default schema for tables without a specific prefix
            'sql', m.sql, -- The original CREATE TABLE statement
            'policies', (
                SELECT json_group_array(
                    json_object(
                        'id', p.id,
                        'name', p.name,
                        'description', p.description,
                        'is_enabled', p.is_enabled,
                        'operation', p.operation,
                        'policy_type', p.policy_type,
                        'using_expr', p.using_expr,
                        'check_expr', p.check_expr
                    )
                )
                FROM _policies p
                WHERE p.table_name = m.name AND p.is_enabled = 1
            ),
            'columns', (
                SELECT json_group_array(
                    json_object(
                        'column_id', txi.cid,
                        'column_name', txi.name,
                        'data_type', txi.type,
                        'is_not_null', txi."notnull",
                        'default_value', txi.dflt_value,
                        'is_primary_key', CASE WHEN txi.pk > 0 THEN 1 ELSE 0 END,
                        'primary_key_order', txi.pk,
                        'generated_column_type', txi.hidden, -- 0: normal, 1: virtual, 2: stored
                        'is_foreign_key', CASE WHEN fkl."from" IS NOT NULL THEN 1 ELSE 0 END,
                        'reference_table', fkl."table",
                        'reference_column', fkl."to",
                        'foreign_key_on_update', fkl.on_update,
                        'foreign_key_on_delete', fkl.on_delete,
                        'part_of_index', (
                            SELECT group_concat(il.name)
                            FROM pragma_index_list(m.name) AS il
                            JOIN pragma_index_info(il.name) AS ii ON ii.name = txi.name
                        )
                    )
                )
                FROM pragma_table_xinfo(m.name) AS txi
                LEFT JOIN pragma_foreign_key_list(m.name) AS fkl ON fkl."from" = txi.name
            )
        ) AS table_details
    FROM
        sqlite_master AS m
    WHERE
        m.type = 'table' AND m.name = ?;
    "#;

    let result = sqlx::query_as::<Sqlite, TableOutput>(sql)
        .bind(name)
        .fetch_one(db)
        .await?;

    Ok(result)
}
