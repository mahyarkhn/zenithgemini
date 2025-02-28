use sqlx::migrate::Migrator;
use crate::db::database::Database;
use crate::models::user::User;
static MIGRATOR: Migrator = sqlx::migrate!("./test_migrations");


#[allow(dead_code)]
async fn setup_test_database() -> Result<Database, sqlx::Error> {
    let db = Database::new("sqlite::memory:").await?;

    MIGRATOR.run(db.pool()).await.unwrap();

    Ok(db)
}

#[tokio::test]
async fn test_migration_and_user_insert() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await.unwrap();

    // Verify table exists
    let table_exists: Result<(i64,), sqlx::Error> =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'")
            .fetch_one(db.pool())
            .await;

    assert!(table_exists.is_ok());
    assert_eq!(table_exists.unwrap().0, 1);

    // Create a test user
    let user = User::new(1, Some("test_user".to_string()), None, None);

    // Insert the user
    user.insert(&db).await.unwrap();

    // ... rest of the test

    Ok(())
}

#[tokio::test]
async fn test_user_insert_find_delete_update() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await?;

    // Create a test user
    let user = User::new(
        1,
        Some("test_user".to_string()),
        Some("Test".to_string()),
        None,
    );

    // Insert the user
    user.insert(&db).await?;

    // Find the user by ID
    let found_user = User::find_by_id(1, &db).await?.unwrap();
    assert_eq!(found_user.username, Some("test_user".to_string()));
    assert_eq!(found_user.first_name, Some("Test".to_string()));

    // Update the user
    let updated_user = User::new(
        1,
        Some("updated_user".to_string()),
        Some("Updated".to_string()),
        Some("Last".to_string()),
    );

    updated_user.update(&db).await?;
    let found_updated_user = User::find_by_id(1, &db).await?.unwrap();
    assert_eq!(found_updated_user.username, Some("updated_user".to_string()));
    assert_eq!(found_updated_user.first_name, Some("Updated".to_string()));
    assert_eq!(found_updated_user.last_name, Some("Last".to_string()));

    // Delete the user
    User::delete_by_id(1, &db).await?;

    // Verify the user is deleted
    let deleted_user = User::find_by_id(1, &db).await?;
    assert!(deleted_user.is_none());

    Ok(())
}

#[tokio::test]
async fn test_find_by_username() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await?;

    let user = User::new(
        1,
        Some("username_test".to_string()),
        Some("Test".to_string()),
        None,
    );

    user.insert(&db).await?;

    let found_user = User::find_by_username("username_test", &db).await?.unwrap();
    assert_eq!(found_user.id, 1);
    assert_eq!(found_user.username, Some("username_test".to_string()));

    let not_found_user = User::find_by_username("non_existent_user", &db).await?;
    assert!(not_found_user.is_none());

    Ok(())
}
