use std::{collections::HashMap, process::exit};
use std::sync::Arc;
use bot::bot_logic::{self, UserStates};
use db::database::Database;
use log::*;
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
    dotenv::dotenv().expect("Failed to load .env variables!");
    pretty_env_logger::init();
    info!("Initializing bot!");

    let database: Option<Database> = match Database::new("sqlite:./database.db").await {
        Err(err) => {
            error!("Could not connect to databse: {}", err.to_string());
            exit(1);
        },
        Ok(connection) => {
            Some(connection)
        }
    };

    let config = AppConfig::new(database.unwrap());
    let user_states: UserStates = Arc::new(Mutex::new(HashMap::new()));
    let bot = Bot::from_env();
    let mut dispatcher = bot_logic::setup_dispatcher(bot, config, user_states).await;
    dispatcher.dispatch().await;
}