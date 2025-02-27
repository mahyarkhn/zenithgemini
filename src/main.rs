#![allow(unused)]
use reqwest::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::io::Error;
use std::ops::Rem;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{env, os};
use teloxide::adaptors::trace::{self, Settings};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::Request;
use teloxide::types::{
    InlineQueryResult, InlineQueryResultArticle, InputFile, InputMediaDocument,
    InputMessageContent, InputMessageContentText, ReplyParameters,
};
use teloxide::utils::command::BotCommands;
use teloxide::utils::markdown::escape;
use teloxide::{prelude::*, repl};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::main;
use tokio::sync::Mutex;
use tokio::time::sleep;

extern crate pretty_env_logger;

#[derive(Clone)]
struct ApplicationDeps {
    gemini_api_key: String,
}

struct UserState {
    query: String,
    task: Option<tokio::task::JoinHandle<()>>,
}

type UserStates = Arc<Mutex<HashMap<i64, UserState>>>;

#[main]
async fn main() {
    dotenv::dotenv().ok();
    let gemini_api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY was not found in env");
    let deps = Arc::new(ApplicationDeps { gemini_api_key });
    let user_states: UserStates = Arc::new(Mutex::new(HashMap::new()));
    let user_states_clone = user_states.clone();

    pretty_env_logger::init();

    // initiate trace settings
    _ = trace::Settings::TRACE_REQUESTS;

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .inspect(|u: Update| {
            eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(message_endpoint),
            ),
        )
        .branch(Update::filter_inline_query().branch(dptree::entry().endpoint(inline_endpoint)));

    let dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![deps.clone(), user_states.clone()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
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

async fn message_endpoint(
    bot: Bot,
    msg: Message,
    cmd: Command,
    deps: Arc<ApplicationDeps>,
) -> ResponseResult<()> {
    match cmd {
        Command::Start | Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Generate(text) => {
            generate(bot, msg, text, deps).await?;
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

async fn generate(
    bot: Bot,
    msg: Message,
    text: String,
    deps: Arc<ApplicationDeps>,
) -> ResponseResult<()> {
    let resp_msg = bot
        .send_message(msg.chat_id().unwrap(), escape_markdown("🔮 *Please stay patient...*"))
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await
        .unwrap();

    let res = query_from_gemini(&msg.text().unwrap(), None, deps).await;

    if res.len() >= 4096 {
        let path = env::temp_dir()
            .join(Path::new(&format!("{}.txt", msg.chat.id)))
            .display()
            .to_string();
        dbg!(&path);
        let file = File::create(&path).await;
        if let Ok(mut file) = file {
            // let mut file = File::open(&path).await.unwrap();
            file.write_all(res.as_bytes()).await;
            file.flush().await;
            file.shutdown().await;
            _ = bot
                .send_document(msg.chat.id, InputFile::file(&path))
                .caption(format!(
                    "Due to limitations of telegram we had to generate a text file for your response.\r\n\r\n🌟 _*@zenithgeminibot*_",
                ))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            fs::remove_file(&path).await;
        } else {
            _ = bot
            .edit_message_text(
                msg.chat_id().unwrap(),
                resp_msg.id,
                format!("Something went wrong while generating you answer...\r\n\r\n❌ _*@zenithgeminibot*_",),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
    } else {
        // dbg!(&res, escape_markdown(&res.clone()));
        _ = bot
            .edit_message_text(
                msg.chat_id().unwrap(),
                resp_msg.id,
                format!("{}\r\n\r\n🌟 _*@zenithgeminibot*_", escape_markdown(&res)),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    }

    Ok(())
}

async fn query_from_gemini(
    query: &str,
    instructions: Option<Vec<&str>>,
    deps: Arc<ApplicationDeps>,
) -> String {
    let data = json!({
        "system_instruction": {
            "parts": {
                "text": format!("Strictly use MarkdownV2 parse mode and dont include escape chars unless they are not a supposed to be a tag. Dont leave any open tags. {}", instructions.unwrap_or(vec![]).join(". "))
            }
        },
        "contents": [{
            "parts": [{
                "text": query
            }]
        }]
    });

    let mut res = reqwest::Client::new()
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={0}", &deps.gemini_api_key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap())
        .send()
        .await
        .unwrap();

    let mut res_text = String::new();

    while let Some(chunk) = &res.chunk().await.unwrap() {
        res_text.push_str(&String::from_utf8_lossy(&chunk));
        println!("Chunk: {}", &String::from_utf8_lossy(&chunk));
    }

    let result: GeminiResponse = serde_json::from_str(&res_text).unwrap();

    let mut res_text = result.candidates[0].content.parts[0].text.clone();

    if let Some(citation) = &result.candidates[0].citation_metadata {
        let links: Vec<Option<String>> = citation
            .citation_sources
            .iter()
            .filter(|x| x.uri != None)
            .map(|x| x.uri.clone())
            .collect();

        for ele in links {
            if let Some(link) = ele {
                res_text.push_str(format!("{}\r\n", link).as_str());
            }
        }
    }

    escape_markdown(&res_text)
}

async fn inline_endpoint(
    bot: Bot,
    query: InlineQuery,
    deps: Arc<ApplicationDeps>,
    user_states: UserStates,
) -> ResponseResult<()> {
    let user_id = query.from.id.0;
    let new_query = query.query.clone();

    let mut states = user_states.lock();
    let mut hash_map = states.await;
    let state = hash_map.entry(user_id as i64).or_insert(UserState {
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
            let mut current_query = current_query.clone();
            current_query.truncate(current_query.len() - 2);
            process_gemini_request(
                bot_clone,
                query_id,
                user_id as i64,
                current_query,
                deps,
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
    deps: Arc<ApplicationDeps>,
    user_states: UserStates,
) {
    // Process the query with Gemini API here
    println!("Processing query: {}", query);

    let results = vec![teloxide::types::InlineQueryResultArticle::new(
        "1",
        format!("Ask Gemini: {}", &query),
        InputMessageContent::Text(
            InputMessageContentText::new(format!(
                "{}\r\n\r\n🌟 _*@zenithgeminibot*_",
                query_from_gemini(
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
    let mut states = user_states.lock();
    states.await.remove(&user_id);
}

fn escape_markdown(s: &str) -> String {
    const CHARS: [char; 9] = ['.', '!', '|', '>', '-', '(', ')', '[', ']'];

    let mut is_escaped = false;

    let escaped = s.chars().fold(String::with_capacity(s.len()), |mut s, c| {
        if c.eq(&'\\') {
            is_escaped = true;
        }
        if CHARS.contains(&c) {
            if !is_escaped {
                s.push('\\');
            } else {
                is_escaped = false;
            }
        }
        s.push(c);
        s
    });

    escaped

    //escape_unclosed_tags(&escaped)

    //// abcd\\[ef\\]g\\* abcd\\[ef\\]
}

fn escape_unclosed_tags(text: &str) -> String {
    let mut result = String::new();
    let mut open_tags: std::collections::HashMap<char, usize> = std::collections::HashMap::new();
    let mut last_tag_pos: std::collections::HashMap<char, Option<usize>> =
        std::collections::HashMap::new();

    let tags: Vec<char> = vec!['*', '_', '~', '`', '[']; // Add other tags as needed

    for tag in &tags {
        open_tags.insert(*tag, 0);
        last_tag_pos.insert(*tag, None);
    }

    for (i, c) in text.char_indices() {
        if tags.contains(&c) {
            let open_count = open_tags.get_mut(&c).unwrap();
            let last_pos = last_tag_pos.get_mut(&c).unwrap();

            if let Some(last_p) = *last_pos {
                if last_p + 1 == i && c == '*' {
                    // Two consecutive asterisks, not a tag.
                    result.push_str("**");
                    *last_pos = None;
                    continue;
                }
            }

            if open_count.rem(2) == 0 {
                // Open tag
                *open_count += 1;
                *last_pos = Some(i);
            } else {
                // Close tag
                *open_count += 1;
                *last_pos = None;
            }
            result.push(c);
        } else {
            for tag in &tags {
                if let Some(last_p) = *last_tag_pos.get(tag).unwrap() {
                    // Unclosed tag
                    result.insert(last_p, '\\'); // Escape the tag
                    *last_tag_pos.get_mut(tag).unwrap() = None;
                }
            }
            result.push(c);
        }
    }

    for tag in &tags {
        if let Some(last_p) = *last_tag_pos.get(tag).unwrap() {
            // Unclosed tag at the end
            result.insert(last_p, '\\'); // Escape the tag
        }
    }

    result
}


#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
    #[serde(rename = "modelVersion")]
    model_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    content: Content,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    #[serde(rename = "citationMetadata")]
    citation_metadata: Option<CitationMetadata>,
    #[serde(rename = "avgLogprobs")]
    avg_logprobs: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
    role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CitationMetadata {
    #[serde(rename = "citationSources")]
    citation_sources: Vec<CitationSource>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CitationSource {
    #[serde(rename = "startIndex")]
    start_index: Option<i64>,
    #[serde(rename = "endIndex")]
    end_index: Option<i64>,
    uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<i64>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<i64>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<i64>,
    #[serde(rename = "promptTokensDetails")]
    prompt_tokens_details: Option<Vec<TokenDetails>>,
    #[serde(rename = "candidatesTokensDetails")]
    candidates_tokens_details: Option<Vec<TokenDetails>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenDetails {
    modality: Option<String>,
    #[serde(rename = "tokenCount")]
    token_count: Option<i64>,
}
