// SPDX-License-Identifier: MIT
//! This module provides functions to interact with the Moralis API,
//! fetching token holders and token metadata for ERC20 tokens.
//!
//! The API endpoints support retrieving a token holders list and token metadata.
//! It uses `reqwest` for HTTP requests and `anyhow` for error handling.
//!
//! Environment variables are used to obtain the API key and base URL.
//!
//! Note: The module assumes the existence of the following environment variables:
//! - `MORALIS_API_KEY`
//! - `MORALIS_API`

use std::fmt::Debug;

use anyhow::anyhow;
use ethers::types::{Chain, U256};
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    app_config::CHAIN,
    token_check::{check_token_lock::TokenHolders, external_api::etherscan_api::TokenWebData},
};

/// Defines the type of API call for the Moralis endpoint.
pub enum MoralisApiCallType {
    /// API call for retrieving token holders.
    GetTokenHolders,
    /// API call for retrieving token metadata.
    GetTokenMetaData,
}

/// Internal structure to deserialize Moralis API responses for token holders.
///
/// The response contains pagination data along with a `result` field holding the data.
#[derive(Debug, Deserialize)]
struct MoralisResponse<T> {
    page: u32,
    page_size: u32,
    cursor: String,
    result: Vec<T>,
}

/// Internal representation of a token holder as returned by the Moralis API.
#[derive(Debug, Deserialize, Default)]
struct MoralisTokenHolder {
    owner_address: String,
    owner_address_label: Option<String>,
    entity: Option<String>,
    entity_logo: Option<String>,
    balance: String,
    balance_formatted: String,
    usd_value: Option<String>,
    is_contract: bool,
    percentage_relative_to_total_supply: f64,
}

/// Metadata information for an ERC20 token retrieved from the Moralis API.
#[derive(Debug, Deserialize)]
pub struct MoralisTokenMetadata {
    /// The token's contract address.
    pub address: String,
    /// Optional label associated with the token address.
    pub address_label: Option<String>,
    /// The name of the token.
    pub name: String,
    /// The symbol of the token.
    pub symbol: String,
    /// Optional decimals information.
    pub decimals: Option<String>,
    /// Optional logo URL.
    pub logo: Option<String>,
    /// Optional logo hash.
    pub logo_hash: Option<String>,
    /// Optional thumbnail URL.
    pub thumbnail: Option<String>,
    /// Optional total supply as a string.
    pub total_supply: Option<String>,
    /// Optional formatted total supply.
    pub total_supply_formatted: Option<String>,
    /// Optional fully diluted valuation.
    pub fully_diluted_valuation: Option<String>,
    /// Optional block number where data was fetched.
    pub block_number: Option<String>,
    /// Optional validation flag.
    pub validated: Option<u32>,
    /// Optional creation date.
    pub created_at: Option<String>,
    /// Optional flag for potential spam.
    pub possible_spam: Option<bool>,
    /// Optional flag indicating contract verification.
    pub verified_contract: Option<bool>,
    /// Categories associated with the token.
    pub categories: Vec<String>,
    /// Associated links and social media.
    pub links: Links,
    /// Optional security score.
    pub security_score: Option<u32>,
    /// Optional description of the token.
    pub description: Option<String>,
    /// Optional circulating supply information.
    pub circulating_supply: Option<String>,
    /// Optional market capitalization info.
    pub market_cap: Option<String>,
}

/// Contains various social media and official links related to a token.
#[derive(Debug, Deserialize)]
pub struct Links {
    pub discord: Option<String>,
    pub reddit: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub website: Option<String>,
    pub moralis: Option<String>,
    pub medium: Option<String>,
    pub github: Option<String>,
    pub bitbucket: Option<String>,
    pub facebook: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub tiktok: Option<String>,
    pub youtube: Option<String>,
}

/// Retrieves a list of token holders with their corresponding balances (in `U256`).
///
/// This function calls the Moralis API using the provided token address and parses
/// the response into a vector of `TokenHolders`. The token balance is parsed as a `U256` value.
///
/// # Arguments
///
/// * `token_address` - A string slice representing the token's contract address.
///
/// # Returns
///
/// * `anyhow::Result<Vec<TokenHolders>>` - A result wrapping a vector of token holder data on success,
///   or an error if the request or parsing fails.
///
/// # Errors
///
/// Returns an error if the API call fails or the token balance parsing fails.
pub async fn get_token_holder_list(token_address: &str) -> anyhow::Result<Vec<TokenHolders>> {
    let token_holders =
        moralis_api_call::<MoralisTokenHolder>(token_address, MoralisApiCallType::GetTokenHolders)
            .await?;
    // Convert API response into Vec<TokenHolders>
    let mut holders = Vec::with_capacity(token_holders.len());
    for entry in token_holders {
        // Parse the balance string into U256.
        let tokens_held = U256::from_dec_str(&entry.balance)?;

        holders.push(TokenHolders {
            holder: entry.owner_address,
            quantity: tokens_held,
        });
    }

    Ok(holders)
}

/// Retrieves token information such as social media profiles and website.
///
/// This function fetches token metadata from the Moralis API and extracts the first entry.
/// If found, it constructs a `TokenWebData` with available link information.
///
/// # Arguments
///
/// * `token_address` - A string slice representing the token's contract address.
///
/// # Returns
///
/// * `anyhow::Result<Option<TokenWebData>>` - On success, returns an Option with token information;
///   if no metadata is found, returns `None`.
pub async fn get_token_info(token_address: &str) -> anyhow::Result<Option<TokenWebData>> {
    let token_metadata = moralis_api_call::<MoralisTokenMetadata>(
        token_address,
        MoralisApiCallType::GetTokenMetaData,
    )
    .await?;

    match token_metadata.first() {
        Some(result) => Ok(Some(TokenWebData {
            website: result.links.website.clone().unwrap_or_default(),
            twitter: result.links.twitter.clone().unwrap_or_default(),
            discord: result.links.discord.clone().unwrap_or_default(),
            whitepaper: "".to_string(),
            ..Default::default()
        })),
        None => Ok(None),
    }
}

/// Makes a call to the Moralis API and parses the JSON response into a vector of type `T`.
///
/// This generic function supports two API call types:
/// - For token metadata, it expects a JSON array directly.
/// - For token holders, it expects an object with a `result` field containing the data.
///
/// # Type Parameters
///
/// * `T`: The type into which the JSON response is deserialized. Must implement `DeserializeOwned` and `Debug`.
///
/// # Arguments
///
/// * `token_address` - A string slice representing the token's contract address.
/// * `api_call_type` - Specifies the type of API call: either token holders or metadata.
///
/// # Returns
///
/// * `anyhow::Result<Vec<T>>` - A vector of deserialized objects on success.
///
/// # Errors
///
/// Returns an error if the API call fails or if JSON deserialization fails.
pub async fn moralis_api_call<T>(
    token_address: &str,
    api_call_type: MoralisApiCallType,
) -> anyhow::Result<Vec<T>>
where
    T: DeserializeOwned + Debug,
{
    // Retrieve the API key and base URL from environment variables.
    let api_key = get_moralis_api_key()?;
    let root_url = get_moralis_api()?;
    let chain_id = get_moralis_chain_id();

    // Construct the URL based on the type of API call.
    let url = match api_call_type {
        MoralisApiCallType::GetTokenMetaData => format!(
            "{}/erc20/metadata?chain={}&addresses={}&order=DESC",
            root_url, chain_id, token_address,
        ),
        MoralisApiCallType::GetTokenHolders => format!(
            "{}/erc20/{}/owners?chain={}&order=DESC",
            root_url, token_address, chain_id
        ),
    };

    println!("url => {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("accept", "application/json")
        .header("X-API-Key", api_key.clone())
        .send()
        .await?;

    // Check if the response is successful; else, capture the error message and return an error.
    if !response.status().is_success() {
        let response_text = response.text().await?;
        return Err(anyhow!("Request failed with: {}", response_text));
    }

    // Parse JSON response based on the API call type.
    let vec_t: Vec<T> = match api_call_type {
        // For metadata endpoint, the response is a JSON array.
        MoralisApiCallType::GetTokenMetaData => match response.json::<Vec<T>>().await {
            Ok(metadata) => metadata,
            Err(error) => {
                // Retry the request to log full response text for debugging.
                let response = client
                    .get(&url)
                    .header("accept", "application/json")
                    .header("X-API-Key", api_key.clone())
                    .send()
                    .await?;

                let response_text = response.text().await?;
                println!("response_text => {}", response_text);
                panic!("Failed parsing token metadata as Vec<T>: {}", error);
            }
        },
        // For holders endpoint, the response is an object with a "result" field.
        MoralisApiCallType::GetTokenHolders => {
            let moralis_response: MoralisResponse<T> = match response.json().await {
                Ok(token_holders) => token_holders,
                Err(_) => {
                    // Retry the request and return an empty vector if it fails.
                    let response = client
                        .get(&url)
                        .header("accept", "application/json")
                        .header("X-API-Key", api_key.clone())
                        .send()
                        .await?;

                    let response_text = response.text().await?;
                    println!("response_text => {}", response_text);
                    return Ok(Vec::new());
                }
            };

            moralis_response.result
        }
    };

    Ok(vec_t)
}

/// Retrieves the chain identifier for the Moralis API based on the configured chain.
///
/// # Returns
///
/// A `String` representing the chain id (`"base"` for `Chain::Base`, or `"eth"` otherwise).
pub fn get_moralis_chain_id() -> String {
    match CHAIN {
        Chain::Base => "base".to_string(),
        _ => "eth".to_string(),
    }
}

/// Retrieves the Moralis API key from the environment variable.
///
/// # Returns
///
/// * `anyhow::Result<String>` that contains the API key on success.
///
/// # Errors
///
/// Returns an error if the `MORALIS_API_KEY` environment variable is not set.
fn get_moralis_api_key() -> anyhow::Result<String> {
    let api_key = std::env::var("MORALIS_API_KEY")
        .map_err(|_| anyhow!("MORALIS_API_KEY is not set in .env"))?;
    Ok(api_key)
}

/// Retrieves the base URL for the Moralis API from the environment variable.
///
/// # Returns
///
/// * `anyhow::Result<String>` that contains the API base URL on success.
///
/// # Errors
///
/// Returns an error if the `MORALIS_API` environment variable is not set.
fn get_moralis_api() -> anyhow::Result<String> {
    let api_url =
        std::env::var("MORALIS_API").map_err(|_| anyhow!("MORALIS_API is not set in .env"))?;
    Ok(api_url)
}
