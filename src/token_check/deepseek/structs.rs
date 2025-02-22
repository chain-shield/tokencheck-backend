//! This module defines response and usage structures for handling DeepSeek API outputs.
//! It includes representations for assistant messages and usage metrics returned by the API.

use serde::Deserialize;
/// ---------------------
///   Response Structures
/// ---------------------
/// Represents a message from the DeepSeek API response.
///
/// This structure holds the role of the sender along with any associated content or reasoning.
#[derive(Debug, Deserialize)]
pub struct AssistantMessageDeepSeek {
    /// Role of the sender (e.g., "assistant", "user", etc.).
    pub role: String,
    /// Optional message content from the assistant.
    pub content: Option<String>,
    /// Optional reasoning content provided by the assistant.
    pub reasoning_content: Option<String>,
    // Uncomment and implement when tool call details are needed.
    // pub tool_calls: Vec<()>,
}

/// Contains usage metrics provided by the DeepSeek API for cost analysis.
///
/// The metrics include token counts for prompts, completions, and related caching details.
#[derive(Deserialize, Debug, Default)]
pub struct UsageDeepSeek {
    /// The number of tokens in the prompt.
    #[serde(default)]
    pub prompt_tokens: i64,
    /// The number of tokens in the assistant's completion.
    #[serde(default)]
    pub completion_tokens: i64,
    /// The total number of tokens used.
    #[serde(default)]
    pub total_tokens: i64,
    /// Tokens count for prompt cache hits.
    #[serde(default)]
    pub prompt_cache_hit_tokens: i64,
    /// Tokens count for prompt cache misses.
    #[serde(default)]
    pub prompt_cache_miss_tokens: i64,
    /// Detailed information on tokens used in completions.
    #[serde(default)]
    pub completion_tokens_details: CompletionTokensDetailsDeepSeek,
}

/// Contains detailed token-usage information for completions.
///
/// Currently, it tracks the number of tokens used specifically for reasoning.
#[derive(Deserialize, Debug, Default)]
pub struct CompletionTokensDetailsDeepSeek {
    /// The number of tokens used by the reasoning part.
    #[serde(default)]
    pub reasoning_tokens: i64,
}

/// Retrieves the DeepSeek API key from the environment variable "DEEPSEEK_API_KEY".
///
/// # Errors
///
/// Returns an error if the "DEEPSEEK_API_KEY" environment variable is not set.
pub fn get_deepseek_api_key() -> anyhow::Result<String> {
    // Use the ? operator for error handling instead of expect.
    let deepseek_key = std::env::var("DEEPSEEK_API_KEY")?;
    Ok(deepseek_key)
}
