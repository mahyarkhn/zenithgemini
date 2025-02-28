use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageHistory {
    pub user_id: i64,
    pub messages: Vec<i64>, // Message ids
    pub last_interaction: i64, // Unix timestamp
}