use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub struct Database {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let connect_options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(connect_options).await?;
        Ok(Database { pool })
    }

    pub async fn run_migrations(
        &self,
        migrations_path: &str,
    ) -> Result<(), sqlx::migrate::MigrateError> {
        let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(migrations_path)).await?;
        migrator.run(&self.pool).await
    }

    pub async fn execute_query(
        &self,
        query: &str,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query(query).execute(&self.pool).await
    }

    pub async fn fetch_one_row(&self, query: &str) -> Result<sqlx::sqlite::SqliteRow, sqlx::Error> {
        sqlx::query(query).fetch_one(&self.pool).await
    }

    pub async fn fetch_optional_row(
        &self,
        query: &str,
    ) -> Result<Option<sqlx::sqlite::SqliteRow>, sqlx::Error> {
        sqlx::query(query).fetch_optional(&self.pool).await
    }

    pub async fn fetch_all_rows(
        &self,
        query: &str,
    ) -> Result<Vec<sqlx::sqlite::SqliteRow>, sqlx::Error> {
        sqlx::query(query).fetch_all(&self.pool).await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
