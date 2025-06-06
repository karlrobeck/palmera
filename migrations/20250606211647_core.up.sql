-- Add up migration script here
CREATE TABLE _policies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    table_name TEXT NOT NULL,
    operation TEXT CHECK(operation IN ('select', 'update', 'insert', 'delete', 'all')) NOT NULL,
    policy_type TEXT CHECK(policy_type IN ('PERMISSIVE', 'RESTRICTIVE')) NOT NULL DEFAULT 'PERMISSIVE',
    using_expr TEXT, -- The expression for SELECT operations (like Postgres' USING)
    check_expr TEXT, -- The expression for INSERT/UPDATE operations (like Postgres' WITH CHECK)
    CHECK(using_expr IS NOT NULL OR check_expr IS NOT NULL) -- A policy must have at least one expression
);