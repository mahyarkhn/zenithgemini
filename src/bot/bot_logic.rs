// Bot logic module
use crate::gemini::services::{escape_markdown, query_gemini_api};
use crate::models::message_history::MessageHistory;
use crate::models::user::User;
use crate::utils::time::unix_timestamp;
use crate::{utils, AppConfig};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::dispatching::DefaultKey;
use teloxide::prelude::*;
use teloxide::types::{
    InputFile, InputMessageContent, InputMessageContentText, ReplyParameters, Update, UpdateKind,
};
use teloxide::utils::command::BotCommands;
use teloxide::utils::html;
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
    #[command(description = "print developer information")]
    DeveloperInfo,
    #[command(description = "create a new topic")]
    NewTopic,
}

pub async fn setup_dispatcher(
    bot: Bot,
    config: Arc<AppConfig>,
    user_states: UserStates,
) -> Dispatcher<Bot, teloxide::RequestError, DefaultKey> {
    let handler = dptree::entry()
        .inspect(|u: Update| {
            #[cfg(debug_assertions)]
            log::debug!("Incoming request: {:?}", &u);
        })
        .inspect_async(before_handling)
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(command_handler),
                )
                .endpoint(message_handler),
        )
        .branch(Update::filter_inline_query().branch(dptree::entry().endpoint(inline_handler)));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config, user_states])
        .enable_ctrlc_handler()
        .build()
}

async fn before_handling(_bot: Bot, update: Update, config: Arc<AppConfig>) {
    if let UpdateKind::Message(_msg) = &update.kind {
        let chat_id = update.chat_id().expect("Could not retrive chat id!");
        let chat = update.chat().unwrap();
        if !chat.is_private() {
            return;
        }

        let sender_id = update.from().expect("Could not retrive sender id!").id.0 as i64;
        let db = config.database.lock().await;
        let user = match User::find_by_id(sender_id, &db).await.unwrap() {
            Some(user) => user,
            None => {
                let user = User::new(
                    chat_id.0,
                    chat.username().map(String::from),
                    chat.first_name().map(String::from),
                    chat.last_name().map(String::from),
                );
                user.insert(&db).await.unwrap();
                user
            }
        };

        log::debug!("{:?}", user);
    }
}

async fn message_handler(bot: Bot, msg: Message, config: Arc<AppConfig>) -> ResponseResult<()> {
    if let Some(text) = msg.text().clone() {
        generate_response(bot, &msg, text.to_string(), config).await?;
    } else {
        bot.send_message(msg.chat.id, Command::descriptions().to_string())
            .await?;
    }

    respond(())
}

async fn command_handler(
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
            generate_response(bot, &msg, text, config).await?;
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?;
        }
        Command::DeveloperInfo => {
            send_developer_info(&bot, &msg).await?;
        }
        Command::NewTopic => {
            let db = config.database.lock().await;
            let sender_id = msg.from.unwrap().id.0 as i64;
            match MessageHistory::delete_by_user_id(sender_id, &db).await {
                Ok(()) => {
                    let history = MessageHistory::new(sender_id as i64, Vec::new());
                    history.insert(&db).await.ok();
                    bot.send_message(msg.chat.id, "Your previous topic has been flushed.")
                        .await?;
                }
                Err(err) => {
                    log::error!("Error while flushing tipoc: {:?}", err);
                    bot.send_message(
                        msg.chat.id,
                        "Something didnt go well, please try again later.",
                    )
                    .await?;
                }
            }
        }
    };

    respond(())
}

async fn send_developer_info(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let profile_link = format!("tg://user?id=6057706319");
    let github_link = format!("https://github.com/mahyarkhn");

    let text = format!(
        "ZenithGemini created by {}\r\n{}\r\n{}\r\nContact me at {}",
        html::bold("MahyarKhn"),
        html::link(&profile_link, "View Profile"),
        html::link(&github_link, "View Github"),
        html::italic("mahyarkhn@proton.me"),
    );

    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    respond(())
}

async fn generate_response(
    bot: Bot,
    msg: &Message,
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

    let mut history_data: Option<Vec<(String, String)>> = None;
    let sender_id = msg.from.as_ref().unwrap().id.0 as i64;
    {
        let db = &config.database.lock().await;
        let histroy = MessageHistory::find_by_user_id(sender_id, &db)
            .await
            .unwrap();
        if let Some(history) = histroy {
            if history.messages.len() > 0 {
                history_data = Some(Vec::new());
                for i in history.messages {
                    if let Some(message) = crate::models::message::Message::find_by_id(i, &db)
                        .await
                        .unwrap()
                    {
                        history_data
                            .as_mut()
                            .unwrap()
                            .push((message.content.unwrap(), message.response.unwrap()));
                    }
                }
            }
        }
    }

    let gemini_response = query_gemini_api(&text, None, &config, history_data).await;

    {
        let db = &config.database.lock().await;
        let mut histroy = MessageHistory::find_by_user_id(sender_id, &db)
            .await
            .unwrap();
        if let None = &histroy {
            histroy = Some(MessageHistory::new(sender_id, Vec::new()));
            _ = histroy.as_ref().unwrap().insert(&db).await;
        }
        let message = crate::models::message::Message::new(
            msg.chat.id.0 as i64,
            sender_id,
            msg.id.0 as i64,
            Some(text.to_string()),
            Some(gemini_response.to_string()),
            msg.date.timestamp(),
        );
        _ = message.insert(&db).await;
        let _ = match crate::models::message::Message::find_by_message_and_chat_id(
            msg.id.0 as i64,
            msg.chat.id.0 as i64,
            &db,
        )
        .await
        .unwrap()
        {
            Some(msg) => {
                histroy.as_mut().unwrap().messages.push(msg.id);
                _ = histroy.as_ref().unwrap().update(&db).await;
                Some(msg)
            }
            None => {
                log::error!("Failed to store message in history! ");
                None
            }
        };
    }

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
        // let res = bot
        //     .edit_message_text(
        //         msg.chat_id().unwrap(),
        //         response_message.id,
        //         format!("{}\r\n\r\n🌟 _*@zenithgeminibot*_", gemini_response),
        //     )
        //     .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        //     .await;
        // if res.is_err() {
            // markdown especial chars may create error sometimes
            bot.edit_message_text(
                msg.chat_id().unwrap(),
                response_message.id,
                format!("{}\r\n\r\n🌟 @zenithgeminibot", gemini_response),
            )
            .await?;
        // }
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
            process_gemini_request(
                &bot_clone,
                query_id,
                user_id as i64,
                current_query,
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
    bot: &Bot,
    query_id: String,
    user_id: i64,
    query_text: String,
    config: Arc<AppConfig>,
    user_states: UserStates,
) {
    let mut history_data: Option<Vec<(String, String)>> = None;
    let sender_id = user_id as i64;
    {
        let db = &config.database.lock().await;
        let histroy = MessageHistory::find_by_user_id(sender_id, &db)
            .await
            .unwrap();
        if let Some(history) = histroy {
            if history.messages.len() > 0 {
                history_data = Some(Vec::new());
                for i in history.messages {
                    if let Some(message) = crate::models::message::Message::find_by_id(i, &db)
                        .await
                        .unwrap()
                    {
                        history_data
                            .as_mut()
                            .unwrap()
                            .push((message.content.unwrap(), message.response.unwrap()));
                    }
                }
            }
        }
    }

    let query_result = query_gemini_api(
        &query_text,
        Some(vec![
            "be extra precise",
            "do not exceed 4700 chars at any chance",
        ]),
        &config,
        history_data,
    )
    .await;

    {
        let db = &config.database.lock().await;
        let mut histroy = MessageHistory::find_by_user_id(sender_id, &db)
            .await
            .unwrap();
        if let None = &histroy {
            histroy = Some(MessageHistory::new(sender_id, Vec::new()));
            _ = histroy.as_ref().unwrap().insert(&db).await;
        }
        let _q = query_text.clone();
        let message = crate::models::message::Message::new(
            sender_id,
            sender_id,
            0,
            Some(_q),
            Some(query_result.to_string()),
            unix_timestamp(),
        );
        _ = message.insert(&db).await;
        let _ = match crate::models::message::Message::find_by_message_and_chat_id(
            sender_id,
            sender_id,
            &db,
        )
        .await
        .unwrap()
        {
            Some(msg) => {
                histroy.as_mut().unwrap().messages.push(msg.id);
                _ = histroy.as_ref().unwrap().update(&db).await;
                Some(msg)
            }
            None => {
                log::error!("Failed to store message in history! ");
                None
            }
        };
    }

    let mut current_query_trimmed = query_text.clone();
    current_query_trimmed.truncate(current_query_trimmed.len() - 2);

    let results = vec![teloxide::types::InlineQueryResultArticle::new(
        "1",
        format!(
            "Ask Gemini: {}\r\n{}",
            &current_query_trimmed,
            utils::string::truncate_text(&query_result, 100)
        ),
        InputMessageContent::Text(
            InputMessageContentText::new(format!(
                "{}\r\n\r\n🌟 @zenithgeminibot",
                query_result
            ))
        ),
    )
    .into()];

    if let Err(e) = bot.answer_inline_query(&query_id, results).await {
        log::error!("Error answering inline query: {}", e);
    }

    // Clean up state
    let states = user_states.lock();
    states.await.remove(&user_id);
}
