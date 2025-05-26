mod from_row;

use sqlx::{AnyPool, Executor, SqlitePool};

const SQL_SCHEMA: &str = include_str!("../../sql/schema.sql");

pub async fn init_db(pool: &SqlitePool) -> Result<(), ()> {
    pool.execute_many(SQL_SCHEMA);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::sql::init_db;
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
    #[tokio::test]
    async fn test_init_db() {
        Sqlite::create_database("sqlite://test.db").await.unwrap();
        let pool = SqlitePool::connect("sqlite://test.db").await.unwrap();
        init_db(&pool).await.unwrap();
    }
}