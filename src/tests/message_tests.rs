use crate::db::database::Database;
use crate::models::message::Message;
use crate::utils::time::unix_timestamp;
use sqlx::migrate::Migrator;
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[allow(dead_code)]
async fn setup_test_database() -> Result<Database, sqlx::Error> {
    let db = Database::new("sqlite::memory:").await?;

    MIGRATOR.run(db.pool()).await.unwrap();

    Ok(db)
}

#[tokio::test]
async fn test_migration_and_history_insert() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await.unwrap();

    let table_exists: Result<(i64,), sqlx::Error> =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages'")
            .fetch_one(db.pool())
            .await;

    assert!(table_exists.is_ok());
    assert_eq!(table_exists.unwrap().0, 1);

    let message = Message::new(1, 2, 4, Some("content".to_string()), Some("response".to_string()), unix_timestamp());

    message.insert(&db).await.unwrap();


    Ok(())
}

#[tokio::test]
async fn test_message_insert() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await?;

    let message = Message::new(1, 2, 4, Some("content".to_string()), Some("response".to_string()), unix_timestamp());

    message.insert(&db).await.unwrap();

    let found_message = Message::find_by_sender_id(2, &db).await?;
    assert!(found_message.is_some());

    Ok(())
}

#[tokio::test]
async fn test_message_insert_delete_sender_id() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await?;

    let message = Message::new(1, 2, 4, Some("content".to_string()), Some("response".to_string()), unix_timestamp());

    message.insert(&db).await.unwrap();

    let found_message = Message::find_by_sender_id(message.sender_id, &db).await?;
    assert!(found_message.is_some());

    Message::delete_by_id(found_message.as_ref().unwrap().id, &db).await?;
    let found_message = Message::find_by_sender_id(found_message.unwrap().sender_id, &db).await?;
    assert!(found_message.is_none());

    Ok(())
}

#[tokio::test]
async fn test_message_insert_delete_id() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await?;

    let message = Message::new(1, 2, 4, Some("content".to_string()), Some("response".to_string()), unix_timestamp());

    message.insert(&db).await.unwrap();

    let found_message = Message::find_by_sender_id(message.sender_id, &db).await?;
    assert!(found_message.is_some());

    Message::delete_by_id(found_message.as_ref().unwrap().id, &db).await?;
    let found_message = Message::find_by_id(found_message.unwrap().id, &db).await?;
    assert!(found_message.is_none());

    Ok(())
}
