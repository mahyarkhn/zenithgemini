use serde::{Deserialize, Serialize};
use sqlx::{self, Row};

use crate::db::database::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub message_id: i64,
    pub content: Option<String>,
    pub response: Option<String>,
    pub created_at: i64, // Unix timestamp
}

#[allow(dead_code)]
impl Message {
    pub fn new(
        chat_id: i64,
        sender_id: i64,
        message_id: i64,
        content: Option<String>,
        response: Option<String>,
        created_at: i64,
    ) -> Self {
        Message {
            id: 0,
            chat_id,
            sender_id,
            message_id,
            content,
            response,
            created_at,
        }
    }

    pub async fn insert(&self, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO messages (chat_id, sender_id, message_id, content, response, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(self.chat_id)
        .bind(self.sender_id)
        .bind(self.message_id)
        .bind(&self.content)
        .bind(&self.response)
        .bind(&self.created_at)
        .execute(db.pool()) 
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: i64, db: &Database) -> Result<Option<Message>, sqlx::Error> {
        let row = sqlx::query("SELECT chat_id, sender_id, message_id, content, response, created_at FROM messages WHERE id = ?")
            .bind(id)
            .fetch_optional(db.pool()) 
            .await?;

        match row {
            Some(row) => {
                let user = Message {
                    id,
                    chat_id: row.try_get("chat_id")?,
                    sender_id: row.try_get("sender_id")?,
                    message_id: row.try_get("message_id")?,
                    content: row.try_get("content")?,
                    response: row.try_get("response")?,
                    created_at: row.try_get("created_at")?,
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_by_id(id: i64, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(id)
            .execute(db.pool()) 
            .await?;
        Ok(())
    }

    pub async fn find_by_sender_id(
        id: i64,
        db: &Database,
    ) -> Result<Option<Message>, sqlx::Error> {
        let row =
            sqlx::query("SELECT id, chat_id, sender_id, message_id, content, response, created_at FROM messages WHERE sender_id = ?")
                .bind(id)
                .fetch_optional(db.pool())
                .await?;

        match row {
            Some(row) => {
                let user = Message {
                    id: row.try_get("id")?,
                    chat_id: row.try_get("chat_id")?,
                    sender_id: id,
                    message_id: row.try_get("message_id")?,
                    content: row.try_get("content")?,
                    response: row.try_get("response")?,
                    created_at: row.try_get("created_at")?,
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn find_by_message_and_chat_id(
        message_id: i64,
        chat_id: i64,
        db: &Database,
    ) -> Result<Option<Message>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, chat_id, sender_id, message_id, content, response, created_at FROM messages WHERE message_id = ? AND chat_id = ?",
        )
        .bind(message_id)
        .bind(chat_id)
        .fetch_optional(db.pool())
        .await?;
    
        match row {
            Some(row) => {
                let message = Message {
                    id: row.try_get("id")?,
                    chat_id: chat_id,
                    sender_id: row.try_get("sender_id")?,
                    message_id,
                    content: row.try_get("content")?,
                    response: row.try_get("response")?,
                    created_at: row.try_get("created_at")?,
                };
                Ok(Some(message))
            }
            None => Ok(None),
        }
    }
    
}
