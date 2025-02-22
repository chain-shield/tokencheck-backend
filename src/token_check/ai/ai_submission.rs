use std::fmt::Debug;

use anyhow::anyhow;
use log::warn;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

use crate::{
    app_config::{CODE_CHECK_PROMPT, WEBSITE_CHECK_PROMPT},
    utils::type_conversion::truncate_code_unicode,
};

use super::ai_structs::{
    AiChatCompletion, AiErrorResponse, ChatCompletionRequest, HasContent, MessageToSend,
    PromptType, TokenCodeCheck, TokenWebsiteCheck,
};

use crate::token_check::{
    deepseek::structs::{get_deepseek_api_key, AssistantMessageDeepSeek, UsageDeepSeek},
    openai::structs::{get_openai_api_key, AssistantMessage, Usage},
};

/// Represents the supported AI models.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AIModel {
    /// DeepSeek AI model.
    DeepSeek,
    /// OpenAI model.
    OpenAi,
}

/// Holds the data necessary for constructing an AI chat completion request.
///
/// This struct is used for both code and website reviews by the AI.
#[derive(Clone, Debug, Default)]
pub struct AIChat {
    /// The instructions provided to the AI.
    pub prompt_instructions: String,
    /// The AI's persona, describing its role.
    pub ai_persona: String,
    /// The content to be reviewed (e.g., Solidity code or website data).
    pub prompt_content_to_review: String,
    /// The type of prompt indicating what the review is for.
    pub prompt_type: PromptType,
}

/// Checks the provided code using the specified AI model.
///
/// This function truncates the code (if needed), creates a chat request using a code review prompt,
/// then forwards the submission to the AI model.
///
/// # Arguments
///
/// * `code` - The Solidity code to be checked.
/// * `ai_model` - The AI model to use for checking, either `AIModel::OpenAi` or `AIModel::DeepSeek`.
///
/// # Returns
///
/// * `anyhow::Result<Option<TokenCodeCheck>>` - The result containing token code check information, if any.
pub async fn check_code_with_ai(
    code: String,
    ai_model: &AIModel,
) -> anyhow::Result<Option<TokenCodeCheck>> {
    // Truncate the code to a maximum allowed size while preserving Unicode
    let truncated_code = truncate_code_unicode(&code, 115_000);

    // Prepare the chat content for code review
    let website_openai_chat = AIChat {
        prompt_instructions: CODE_CHECK_PROMPT.to_string(),
        ai_persona: "You are a solidity security expert and token analyst.".to_string(),
        prompt_content_to_review: truncated_code,
        prompt_type: PromptType::Code,
    };

    // Submit the prepared chat to the AI assistant and await its response
    let code_check = chat_submission::<TokenCodeCheck>(website_openai_chat, ai_model).await?;

    Ok(code_check)
}

/// Checks the provided website content using the specified AI model.
///
/// This function creates a chat request using a website review prompt,
/// then sends it to the AI model for evaluation.
///
/// # Arguments
///
/// * `website_content` - The content (scraped from a website) to be checked.
/// * `ai_model` - The AI model to use, either `AIModel::OpenAi` or `AIModel::DeepSeek`.
///
/// # Returns
///
/// * `anyhow::Result<Option<TokenWebsiteCheck>>` - The result containing website check information, if any.
pub async fn check_website_with_ai(
    website_content: String,
    ai_model: &AIModel,
) -> anyhow::Result<Option<TokenWebsiteCheck>> {
    // Prepare the chat content for website review
    let website_openai_chat = AIChat {
        prompt_instructions: WEBSITE_CHECK_PROMPT.to_string(),
        ai_persona: "You are an expert crypto investigator specializing in evaluating crypto website credibility.".to_string(),
        prompt_content_to_review: website_content,
        prompt_type: PromptType::Website,
    };

    // Submit the prepared chat to the AI assistant and await its response
    let website_check = chat_submission::<TokenWebsiteCheck>(website_openai_chat, ai_model).await?;

    Ok(website_check)
}

/// Submits the chat request to the AI interface and processes the response.
///
/// This generic function builds the request, calls the appropriate API endpoint, and parses the response.
///
/// # Type Parameters
///
/// * `T`: The expected type for the parsed response.
///
/// # Arguments
///
/// * `chat` - The chat request details.
/// * `ai_model` - The AI model to use for submission.
///
/// # Returns
///
/// * `anyhow::Result<Option<T>>` - The parsed result returned by the AI, if any.
pub async fn chat_submission<T>(chat: AIChat, ai_model: &AIModel) -> anyhow::Result<Option<T>>
where
    T: DeserializeOwned,
{
    // If the prompt content is empty, log a warning and return None.
    if chat.prompt_content_to_review.is_empty() {
        warn!("no {}", chat.prompt_type);
        return Ok(None);
    }

    // Retrieve the appropriate API key depending on the AI model
    let api_key = match ai_model {
        AIModel::OpenAi => get_openai_api_key()?,
        AIModel::DeepSeek => get_deepseek_api_key()?,
    };

    // Create a new HTTP client instance
    let client = Client::new();

    // Combine prompt instructions and content into one message
    let content = format!(
        "{}\n\n{}:\n{}",
        chat.prompt_instructions, chat.prompt_type, chat.prompt_content_to_review
    );

    // Choose model-specific parameters
    let (model, url, max_tokens) = match ai_model {
        AIModel::DeepSeek => (
            "deepseek-reasoner".to_string(),
            "https://api.deepseek.com",
            8_000,
        ),
        AIModel::OpenAi => ("gpt-4o".to_string(), "https://api.openai.com/v1", 16_000),
    };

    // Build the request body as a typed struct for the chat completion API
    let request_body = ChatCompletionRequest {
        model, // May use models like "gpt-4" / "gpt-4o-2024-08-06" depending on requirements.
        messages: vec![
            MessageToSend {
                role: "system".to_string(),
                content: chat.ai_persona,
            },
            MessageToSend {
                role: "user".to_string(),
                content,
            },
        ],
        temperature: 0.3,
        max_tokens,
        top_p: 1.0,
    };

    let api_url = format!("{}/chat/completions", url);

    // Send a POST request to the AI service with proper authentication and JSON body.
    let response = client
        .post(api_url)
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .await?;

    // If the response status is not successful, parse the error and return it.
    if !response.status().is_success() {
        let err_json: AiErrorResponse = response.json().await?;
        return Err(anyhow!(
            "OpenAI Error: {} (type={:?}, code={:?})",
            err_json.error.message,
            err_json.error.r#type,
            err_json.error.code
        ));
    }

    // Depending on the AI model, process the response using appropriate types.
    if *ai_model == AIModel::OpenAi {
        return process_response::<T, AssistantMessage, Usage>(response).await;
    } else {
        return process_response::<T, AssistantMessageDeepSeek, UsageDeepSeek>(response).await;
    }
}

/// Processes the HTTP response from the AI service.
///
/// This function deserializes the response, extracts the first choice from the returned list,
/// and attempts to parse the assistant's message content as JSON.
///
/// # Type Parameters
///
/// * `T`: Expected type for the audit result.
/// * `K`: Type representing the assistant's message; must implement `HasContent` and `Debug`.
/// * `M`: Type representing the usage data; must implement `Default`, `DeserializeOwned`, and `Debug`.
///
/// # Arguments
///
/// * `response` - The HTTP response from the AI service.
///
/// # Returns
///
/// * `anyhow::Result<Option<T>>` - The parsed audit result, or `None` if parsing fails.
async fn process_response<T, K, M>(response: Response) -> anyhow::Result<Option<T>>
where
    T: DeserializeOwned,
    K: DeserializeOwned + HasContent + Debug,
    M: DeserializeOwned + Default + Debug,
{
    // Parse the entire response as an AiChatCompletion struct.
    let resp: AiChatCompletion<K, M> = response.json().await?;

    // Get the first choice from the response. Return an error if no choices are provided.
    let first_choice = resp
        .choices
        .get(0)
        .ok_or_else(|| anyhow!("No choices returned from OpenAI"))?;

    // Access the assistant's message content, which should contain JSON.
    let audit_string = first_choice
        .message
        .get_content()
        .ok_or_else(|| anyhow!("No 'content' field in the assistant's message"))?;

    // Attempt to deserialize the content into the expected type T.
    let audit: T = match serde_json::from_str(&audit_string) {
        Ok(parsed) => parsed,
        Err(e) => {
            // Log the issue if the assistant's response is not valid JSON and return None.
            println!(
                "Assistant didn't return expected JSON. Received:\n{}\n\nError was: {}",
                audit_string, e
            );
            return Ok(None);
        }
    };

    Ok(Some(audit))
}
