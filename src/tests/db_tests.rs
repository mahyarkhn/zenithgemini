use crate::db::database::Database;
use sqlx::Row;
#[cfg(test)]
#[allow(unused)]
use tokio::fs::{metadata, remove_file};

#[tokio::test]
async fn test_database_connection_and_query() -> Result<(), sqlx::Error> {
    let db = Database::new("sqlite::memory:").await?;

    // Create a test table
    db.execute_query(
        "CREATE TABLE test_table (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            )",
    )
    .await?;

    // Insert test data
    db.execute_query("INSERT INTO test_table (name) VALUES ('Test Name')")
        .await?;

    // Query the test data
    let row = db
        .fetch_one_row("SELECT name FROM test_table WHERE id = 1")
        .await?;

    // Extract the name from the row
    let name: String = row.try_get("name")?;

    // Assert that the name is correct
    assert_eq!(name, "Test Name");

    // Test for empty table
    db.execute_query("DELETE FROM test_table;").await?;

    let empty_result = db.fetch_optional_row("SELECT * FROM test_table").await?;

    assert!(empty_result.is_none());

    Ok(())
}
