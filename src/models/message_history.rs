use serde::{Deserialize, Serialize};
use sqlx::{self, Row};

use crate::db::database::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageHistory {
    pub id: i64,
    pub user_id: i64,
    pub messages: Vec<i64>,
}

#[allow(dead_code)]
impl MessageHistory {
    pub async fn insert(&self, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO message_history (user_id, messages) VALUES (?, ?)")
            .bind(self.user_id)
            .bind(serde_json::to_string(&self.messages).unwrap())
            .execute(db.pool())
            .await?;

        Ok(())
    }

    pub async fn find_by_user_id(
        user_id: i64,
        db: &Database,
    ) -> Result<Option<MessageHistory>, sqlx::Error> {
        let row =
            sqlx::query("SELECT id, user_id, messages FROM message_history WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(db.pool())
                .await?;

        match row {
            Some(row) => {
                let user = MessageHistory {
                    id: row.try_get("id")?,
                    user_id: row.try_get("user_id")?,
                    messages: serde_json::from_str(row.try_get("messages").unwrap()).unwrap(),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn find_by_id(id: i64, db: &Database) -> Result<Option<MessageHistory>, sqlx::Error> {
        let row = sqlx::query("SELECT id, user_id, messages FROM message_history WHERE id = ?")
            .bind(id)
            .fetch_optional(db.pool())
            .await?;

        match row {
            Some(row) => {
                let user = MessageHistory {
                    id: row.try_get("id")?,
                    user_id: row.try_get("user_id")?,
                    messages: serde_json::from_str(row.try_get("messages").unwrap()).unwrap(),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_by_id(id: i64, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM message_history WHERE id = ?")
            .bind(id)
            .execute(db.pool())
            .await?;
        Ok(())
    }

    pub async fn delete_by_user_id(id: i64, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM message_history WHERE user_id = ?")
            .bind(id)
            .execute(db.pool())
            .await?;
        Ok(())
    }

    pub async fn update(&self, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE message_history SET messages = ? WHERE id = ?")
            .bind(serde_json::to_string(&self.messages).unwrap())
            .bind(self.id)
            .execute(db.pool())
            .await?;

        Ok(())
    }
}
