// Services module
use crate::app::config;
use crate::gemini::models;
use reqwest;
use serde_json::json;
use std::sync::Arc;

pub async fn query_gemini_api(
    query: &str,
    instructions: Option<Vec<&str>>,
    config: Arc<config::AppConfig>,
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

    let mut response = reqwest::Client::new()
            .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", &config.gemini_api_key))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&data).unwrap())
            .send()
            .await
            .unwrap();

    let mut response_text = String::new();

    while let Some(chunk) = &response.chunk().await.unwrap() {
        response_text.push_str(&String::from_utf8_lossy(&chunk));
        #[cfg(debug_assertions)]
        println!("Chunk: {}", &String::from_utf8_lossy(&chunk));
    }

    let result: models::GeminiResponse = serde_json::from_str(&response_text).unwrap();

    let mut result_text = result.candidates[0].content.parts[0].text.clone();

    if let Some(citation) = &result.candidates[0].citation_metadata {
        let links: Vec<Option<String>> = citation
            .citation_sources
            .iter()
            .filter(|x| x.uri != None)
            .map(|x| x.uri.clone())
            .collect();

        for link in links {
            if let Some(url) = link {
                result_text.push_str(format!("{}\r\n", url).as_str());
            }
        }
    }

    escape_markdown(&result_text)
}

pub fn escape_markdown(s: &str) -> String {
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
}
