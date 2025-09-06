use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use uuid::Uuid;

use crate::migrate;

pub async fn setup_test_db() -> Result<DatabaseConnection, DbErr> {
    let mgmt_db_url = std::env::var("MANAGEMENT_DATABASE_URL")
        .expect("MANAGEMENT_DATABASE_URL must be set for tests");
    let db_name = format!("test_db_{}", Uuid::new_v4().simple());
    let (base_url, _) = mgmt_db_url
        .rsplit_once('/')
        .expect("MANAGEMENT_DATABASE_URL must include a database name, e.g., '/postgres'");

    let db = Database::connect(&mgmt_db_url).await?;
    db.execute(Statement::from_string(
        DbBackend::Postgres,
        format!(r#"CREATE DATABASE "{}";"#, db_name),
    ))
    .await?;

    // 2. Connect to the new test DB and run migrations
    let new_db_url = format!("{}/{}", base_url, db_name);
    let conn = Database::connect(&new_db_url).await?;
    migrate(&conn).await?;

    Ok(conn)
}
