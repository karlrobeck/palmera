use palmera_database::sqlite::schemas::get_table_info;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    // 1. Create a roles table
    sqlx::query(
        r#"
        CREATE TABLE roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 2. Create a users table with a foreign key to roles and an index on email
    sqlx::query(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            is_active INTEGER NOT NULL DEFAULT 1,
            role_id INTEGER,
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE SET NULL ON UPDATE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 3. Create an index on users.email
    sqlx::query(
        r#"
        CREATE INDEX idx_users_email ON users(email);
        "#,
    )
    .execute(&pool)
    .await?;

    // 4. Insert policies for the users table (select, update, insert, delete)
    sqlx::query(
        r#"
        INSERT INTO _policies (name, description, is_enabled, table_name, operation, policy_type, using_expr)
        VALUES (
            'active_users_select',
            'Only allow access to active users (SELECT)',
            1,
            'users',
            'select',
            'PERMISSIVE',
            'is_active = 1'
        );
        "#
    ).execute(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO _policies (name, description, is_enabled, table_name, operation, policy_type, check_expr)
        VALUES (
            'active_users_insert',
            'Only allow insert if user is active (INSERT)',
            1,
            'users',
            'insert',
            'PERMISSIVE',
            'is_active = 1'
        );
        "#
    ).execute(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO _policies (name, description, is_enabled, table_name, operation, policy_type, check_expr)
        VALUES (
            'active_users_update',
            'Only allow update if user remains active (UPDATE)',
            1,
            'users',
            'update',
            'PERMISSIVE',
            'is_active = 1'
        );
        "#
    ).execute(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO _policies (name, description, is_enabled, table_name, operation, policy_type, using_expr)
        VALUES (
            'active_users_delete',
            'Only allow delete of active users (DELETE)',
            1,
            'users',
            'delete',
            'PERMISSIVE',
            'is_active = 1'
        );
        "#
    ).execute(&pool).await?;

    // 5. Insert a policy for the roles table
    sqlx::query(
        r#"
        INSERT INTO _policies (name, description, is_enabled, table_name, operation, policy_type, using_expr)
        VALUES (
            'all_roles',
            'Allow access to all roles',
            1,
            'roles',
            'select',
            'PERMISSIVE',
            '1 = 1'
        );
        "#
    ).execute(&pool).await?;

    // 6. Use get_table_info and print the result for users
    let users_info = get_table_info(&pool, "users").await?;
    println!("Users Table Info: {:#?}", users_info);

    // 7. Use get_table_info and print the result for roles
    let roles_info = get_table_info(&pool, "roles").await?;
    println!("Roles Table Info: {:#?}", roles_info);

    Ok(())
}
