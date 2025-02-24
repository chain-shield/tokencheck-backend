use serde::Deserialize;
///! Module for OpenAI API response structures and helper functions.
///
/// This module contains data structures that mirror the JSON responses
/// from OpenAI, including assistant messages and token usage details.
/// It also provides utility functions, such as retrieving the OpenAI API key
/// from environment variables.

/// Represents a message from the OpenAI assistant.
///
/// This struct mirrors the entire JSON response from OpenAI's API for an assistant message.
/// The message content is assumed to strictly match the ScamCheck schema.
#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    /// Role of the sender (e.g., "assistant", "user").
    pub role: String,
    /// The textual content of the message, if available.
    pub content: Option<String>,
    /// Optional field indicating a refusal. Present when the assistant is declining to respond.
    #[serde(default)]
    pub refusal: Option<()>,
}

/// Provides token usage details returned by OpenAI for cost analysis.
///
/// This struct contains information about the number of tokens consumed
/// during prompt processing and completion generation, as well as detailed breakdowns.
#[derive(Deserialize, Debug, Default)]
pub struct Usage {
    /// Number of tokens contained in the prompt.
    #[serde(default)]
    pub prompt_tokens: i64,
    /// Number of tokens generated in the completion.
    #[serde(default)]
    pub completion_tokens: i64,
    /// Total number of tokens used (typically the sum of prompt and completion tokens).
    #[serde(default)]
    pub total_tokens: i64,
    /// Detailed breakdown of prompt token usage.
    #[serde(default)]
    pub prompt_tokens_details: PromptTokensDetails,
    /// Detailed breakdown of completion token usage.
    #[serde(default)]
    pub completion_tokens_details: CompletionTokensDetails,
}

/// Detailed usage information for prompt tokens.
///
/// This struct provides additional details such as the number of cached tokens.
#[derive(Deserialize, Debug, Default)]
pub struct PromptTokensDetails {
    /// Number of cached tokens from previous computations.
    #[serde(default)]
    pub cached_tokens: i64,
}

/// Detailed usage information for completion tokens.
///
/// This struct provides a breakdown of the tokens used in completions, including
/// reasoning tokens and those from accepted or rejected predictions.
#[derive(Deserialize, Debug, Default)]
pub struct CompletionTokensDetails {
    /// Number of tokens used for reasoning.
    #[serde(default)]
    pub reasoning_tokens: i64,
    /// Number of tokens from accepted predictions.
    #[serde(default)]
    pub accepted_prediction_tokens: i64,
    /// Number of tokens from rejected predictions.
    #[serde(default)]
    pub rejected_prediction_tokens: i64,
}

/// Retrieves the OpenAI API key from the environment variables.
///
/// This function attempts to read the API key from the "OPENAI_API_KEY" environment variable
/// and returns an error if the variable is not set.
///
/// # Errors
///
/// Returns an error if the API key is not found in the environment.
pub fn get_openai_api_key() -> anyhow::Result<String> {
    // Use the ? operator for error propagation instead of `expect`.
    let api_key = std::env::var("OPENAI_API_KEY")?;
    Ok(api_key)
}
