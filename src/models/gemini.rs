use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    #[serde(rename = "citationMetadata")]
    pub citation_metadata: Option<CitationMetadata>,
    #[serde(rename = "avgLogprobs")]
    pub avg_logprobs: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CitationMetadata {
    #[serde(rename = "citationSources")]
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CitationSource {
    #[serde(rename = "startIndex")]
    pub start_index: Option<i64>,
    #[serde(rename = "endIndex")]
    pub end_index: Option<i64>,
    pub uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<i64>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<i64>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<i64>,
    #[serde(rename = "promptTokensDetails")]
    pub prompt_tokens_details: Option<Vec<TokenDetails>>,
    #[serde(rename = "candidatesTokensDetails")]
    pub candidates_tokens_details: Option<Vec<TokenDetails>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub modality: Option<String>,
    #[serde(rename = "tokenCount")]
    pub token_count: Option<i64>,
}
