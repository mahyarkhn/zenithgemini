use std::sync::Arc;
use std::env;

// Configuration module
#[derive(Clone)]
pub struct AppConfig {
    pub gemini_api_key: String,
}

impl AppConfig {
    pub fn new() -> Arc<Self> {
        dotenv::dotenv().ok();
        let gemini_api_key =
            env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY was not found in env");
        Arc::new(Self { gemini_api_key })
    }
}
