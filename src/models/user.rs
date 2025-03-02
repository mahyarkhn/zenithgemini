use crate::db::database::Database;
use serde::{Deserialize, Serialize};
use sqlx::{self, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub chat_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[allow(dead_code)]
impl User {
    pub fn new(
        chat_id: i64,
        username: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Self {
        User {
            chat_id,
            username,
            first_name,
            last_name,
        }
    }

    pub async fn insert(&self, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO users (chat_id, username, first_name, last_name) VALUES (?, ?, ?, ?)",
        )
        .bind(self.chat_id)
        .bind(&self.username)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .execute(db.pool()) 
        .await?;

        Ok(())
    }

    pub async fn find_by_id(chat_id: i64, db: &Database) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query("SELECT chat_id, username, first_name, last_name FROM users WHERE chat_id = ?")
            .bind(chat_id)
            .fetch_optional(db.pool())
            .await?;

        match row {
            Some(row) => {
                let user = User {
                    chat_id: row.try_get("chat_id")?,
                    username: row.try_get("username")?,
                    first_name: row.try_get("first_name")?,
                    last_name: row.try_get("last_name")?,
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_by_id(chat_id: i64, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE chat_id = ?")
            .bind(chat_id)
            .execute(db.pool()) 
            .await?;
        Ok(())
    }

    pub async fn update(&self, db: &Database) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET username = ?, first_name = ?, last_name = ? WHERE chat_id = ?")
            .bind(&self.username)
            .bind(&self.first_name)
            .bind(&self.last_name)
            .bind(self.chat_id)
            .execute(db.pool()) 
            .await?;

        Ok(())
    }

    pub async fn find_by_username(
        username: &str,
        db: &Database,
    ) -> Result<Option<User>, sqlx::Error> {
        let row =
            sqlx::query("SELECT chat_id, username, first_name, last_name FROM users WHERE username = ?")
                .bind(username)
                .fetch_optional(db.pool())
                .await?;

        match row {
            Some(row) => {
                let user = User {
                    chat_id: row.try_get("chat_id")?,
                    username: row.try_get("username")?,
                    first_name: row.try_get("first_name")?,
                    last_name: row.try_get("last_name")?,
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}
