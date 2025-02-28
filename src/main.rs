use std::collections::HashMap;
use std::sync::Arc;
use bot::bot_logic::{self, UserStates};
use teloxide::prelude::*;
use tokio::sync::Mutex;
use app::config::AppConfig;

#[cfg(test)]
mod tests;
mod bot;
mod gemini;
mod app;
mod db;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config = AppConfig::new();
    let user_states: UserStates = Arc::new(Mutex::new(HashMap::new()));

    let bot = Bot::from_env();
    let mut dispatcher = bot_logic::setup_dispatcher(bot, config, user_states).await;

    dispatcher.dispatch().await;
}