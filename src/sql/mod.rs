//! Code for serialising FIRDS data to and from an SQL database.

mod error;
mod to_db;

use sqlx::{Executor, SqlitePool};

const SQL_SCHEMA: &str = include_str!("../../sql/schema.sql");

pub async fn init_db(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    pool.execute(SQL_SCHEMA).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::sql::init_db;
    use sqlx::{Sqlite, sqlite::SqlitePoolOptions, migrate::MigrateDatabase};
    
    const SQLITE_URL : &str = "sqlite://test_output/test.db";
    
    #[tokio::test]
    async fn test_init_db() {
        Sqlite::create_database(SQLITE_URL).await.unwrap();
        let pool = SqlitePoolOptions::new()
            .connect(SQLITE_URL)
            .await.unwrap();
        init_db(&pool).await.unwrap();
    }
}