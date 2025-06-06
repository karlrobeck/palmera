use sea_query::{Alias, ColumnDef, Expr, Func, FunctionCall, Table, TableCreateStatement};

pub fn table_info(table_name: &str) -> FunctionCall {
    Func::cust("pragma_table_info").arg(table_name)
}

pub fn table_list() -> FunctionCall {
    Func::cust("pragma_table_list")
}

pub fn table_xinfo(table_name: &str) -> FunctionCall {
    Func::cust("pragma_table_xinfo").arg(table_name)
}

pub fn foreign_key_list(table_name: &str) -> FunctionCall {
    Func::cust("pragma_foreign_key_list").arg(table_name)
}

pub fn index_list(table_name: &str) -> FunctionCall {
    Func::cust("pragma_index_list").arg(table_name)
}

pub fn index_info(index_name: &str) -> FunctionCall {
    Func::cust("pragma_index_info").arg(index_name)
}

pub fn index_xinfo(index_name: &str) -> FunctionCall {
    Func::cust("pragma_index_xinfo").arg(index_name)
}

pub fn database_list() -> FunctionCall {
    Func::cust("pragma_database_list")
}

pub fn collation_list() -> FunctionCall {
    Func::cust("pragma_collation_list")
}

pub fn function_list() -> FunctionCall {
    Func::cust("pragma_function_list")
}

pub fn module_list() -> FunctionCall {
    Func::cust("pragma_module_list")
}

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
