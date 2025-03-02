use crate::db::database::Database;
use crate::models::message_history::MessageHistory;
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

    let table_exists: Result<(i64,), sqlx::Error> = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='message_history'",
    )
    .fetch_one(db.pool())
    .await;

    assert!(table_exists.is_ok());
    assert_eq!(table_exists.unwrap().0, 1);

    let history = MessageHistory {
        id: 0,
        user_id: 1,
        messages: vec![9, 8, 7, 1, 2],
    };

    history.insert(&db).await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_history_insert_find_delete_update() -> Result<(), sqlx::Error> {
    let db = setup_test_database().await?;

    let history = MessageHistory {
        id: 0,
        user_id: 2,
        messages: vec![9, 8, 7, 1, 2, 3, 4, 5],
    };

    history.insert(&db).await.unwrap();

    let found_history = MessageHistory::find_by_user_id(2, &db).await?.unwrap();
    assert_eq!(found_history.messages, vec![9, 8, 7, 1, 2, 3, 4, 5]);

    let mut history = found_history.clone();
    history.messages = vec![9, 8, 7, 1, 2, 3, 4, 5, 10, 12, 16];

    history.update(&db).await?;
    let found_updated_history = MessageHistory::find_by_user_id(found_history.user_id, &db)
        .await?
        .unwrap();
    assert_eq!(
        found_updated_history.messages,
        vec![9, 8, 7, 1, 2, 3, 4, 5, 10, 12, 16]
    );

    MessageHistory::delete_by_id(history.id, &db).await?;

    let deleted_history = MessageHistory::find_by_user_id(2, &db).await?;
    assert!(deleted_history.is_none());

    Ok(())
}
