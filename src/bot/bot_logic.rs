// Bot logic module
use crate::gemini::services::{escape_markdown, query_gemini_api};
use crate::AppConfig;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::dispatching::DefaultKey;
use teloxide::prelude::*;
use teloxide::types::{
    InputFile, InputMessageContent, InputMessageContentText, ReplyParameters, Update,
};
use teloxide::utils::command::BotCommands;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub type UserStates = Arc<Mutex<HashMap<i64, UserState>>>;

// #[derive(Clone)]
pub struct UserState {
    query: String,
    task: Option<tokio::task::JoinHandle<()>>,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Generates a response from Gemini api.")]
    Generate(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

pub async fn setup_dispatcher(
    bot: Bot,
    config: Arc<AppConfig>,
    user_states: UserStates,
) -> Dispatcher<Bot, teloxide::RequestError, DefaultKey> {
    let handler = dptree::entry()
        .inspect(|_u: Update| {
            #[cfg(debug_assertions)]
            eprintln!("{_u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(message_handler),
            ),
        )
        .branch(Update::filter_inline_query().branch(dptree::entry().endpoint(inline_handler)));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config, user_states])
        .enable_ctrlc_handler()
        .build()
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    config: Arc<AppConfig>,
) -> ResponseResult<()> {
    match cmd {
        Command::Start | Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Generate(text) => {
            generate_response(bot, msg, text, config).await?;
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?;
        }
    };

    respond(())
}

async fn generate_response(
    bot: Bot,
    msg: Message,
    text: String,
    config: Arc<AppConfig>,
) -> ResponseResult<()> {
    let response_message = bot
        .send_message(
            msg.chat_id().unwrap(),
            escape_markdown("🔮 *Please stay patient...*"),
        )
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await
        .unwrap();

    let gemini_response = query_gemini_api(&text, None, config).await;

    if gemini_response.len() >= 4096 {
        let file_path = std::env::temp_dir()
            .join(std::path::Path::new(&format!("{}.txt", msg.chat.id)))
            .display()
            .to_string();

        if let Ok(mut file) = File::create(&file_path).await {
            file.write_all(gemini_response.as_bytes()).await?;
            file.flush().await?;
            file.shutdown().await?;

            bot.send_document(msg.chat.id, InputFile::file(&file_path))
                    .caption(format!(
                        "Due to limitations of telegram we had to generate a text file for your response.\r\n\r\n🌟 _*@zenithgeminibot*_",
                    ))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;

            tokio::fs::remove_file(&file_path).await?;
        } else {
            bot.edit_message_text(
                    msg.chat_id().unwrap(),
                    response_message.id,
                    format!(
                        "Something went wrong while generating you answer...\r\n\r\n❌ _*@zenithgeminibot*_",
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    } else {
        bot.edit_message_text(
            msg.chat_id().unwrap(),
            response_message.id,
            format!("{}\r\n\r\n🌟 _*@zenithgeminibot*_", gemini_response),
        )
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    }

    Ok(())
}

async fn inline_handler(
    bot: Bot,
    query: InlineQuery,
    config: Arc<AppConfig>,
    user_states: UserStates,
) -> ResponseResult<()> {
    let user_id = query.from.id.0;
    let new_query = query.query.clone();

    let mut states = user_states.lock().await;
    let state = states.entry(user_id as i64).or_insert(UserState {
        query: String::new(),
        task: None,
    });

    state.query = new_query.clone();

    if let Some(task) = state.task.take() {
        task.abort(); // Cancel the previous task
    }

    let bot_clone = bot.clone();
    let query_id = query.id.clone();
    let user_states_clone = user_states.clone();
    let current_query = state.query.clone();

    let task = tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;

        // User finished typing, process the query
        if current_query.ends_with("!!") {
            let mut current_query_trimmed = current_query.clone();
            current_query_trimmed.truncate(current_query_trimmed.len() - 2);
            process_gemini_request(
                bot_clone,
                query_id,
                user_id as i64,
                current_query_trimmed,
                config,
                user_states_clone,
            )
            .await;
        }
    });

    state.task = Some(task);

    respond(())
}

async fn process_gemini_request(
    bot: Bot,
    query_id: String,
    user_id: i64,
    query: String,
    deps: Arc<AppConfig>,
    user_states: UserStates,
) {
    let results = vec![teloxide::types::InlineQueryResultArticle::new(
        "1",
        format!("Ask Gemini: {}", &query),
        InputMessageContent::Text(
            InputMessageContentText::new(format!(
                "{}\r\n\r\n🌟 _*@zenithgeminibot*_",
                query_gemini_api(
                    &query,
                    Some(vec![
                        "be extra precise",
                        "do not exceed 4700 chars at any chance"
                    ]),
                    deps
                )
                .await
            ))
            .parse_mode(teloxide::types::ParseMode::MarkdownV2),
        ),
    )
    .into()];

    if let Err(e) = bot.answer_inline_query(&query_id, results).await {
        eprintln!("Error answering inline query: {}", e);
    }

    // Clean up state
    let states = user_states.lock();
    states.await.remove(&user_id);
}
