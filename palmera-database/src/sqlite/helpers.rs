use sea_query::{Alias, ColumnDef, Expr, Table, TableCreateStatement};

pub fn create_policy_table() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("_policies"))
        .if_not_exists()
        .col(
            ColumnDef::new("id")
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new("name").string().not_null().unique_key())
        .col(ColumnDef::new("description").string().null())
        .col(ColumnDef::new("is_enabled").integer().not_null().default(1))
        .col(ColumnDef::new("table_name").string().not_null())
        .col(
            ColumnDef::new("operation")
                .string()
                .not_null()
                .check("operation IN ('select', 'update', 'insert', 'delete', 'all')"),
        )
        .col(
            ColumnDef::new("policy_type")
                .string()
                .not_null()
                .default("PERMISSIVE")
                .check("policy_type IN ('PERMISSIVE', 'RESTRICTIVE')"),
        )
        .col(ColumnDef::new("using_expr").string().null())
        .col(ColumnDef::new("check_expr").string().null())
        .check(Expr::cust(
            "using_expr IS NOT NULL OR check_expr IS NOT NULL",
        ))
        .to_owned()
}
