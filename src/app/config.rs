use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::database::Database;

// Configuration module
// #[derive(Clone)]
pub struct AppConfig {
    pub gemini_api_key: String,
    pub database: Mutex<Database>,
}

impl AppConfig {
    pub fn new(database: Database) -> Arc<Self> {
        let gemini_api_key =
            env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY was not found in env");
        Arc::new(Self {
            gemini_api_key,
            database: Mutex::new(database),
        })
    }
}
