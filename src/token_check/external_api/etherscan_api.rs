//! This module provides asynchronous functions to interact with the Etherscan API.
//!
//! It enables fetching token holder lists, contract source code, contract owner information,
//! and token information. The module uses `reqwest` for HTTP requests and `serde` for JSON deserialization.
//!
//! Make sure to set the environment variables `ETHERSCAN_API_KEY` and `ETHERSCAN_API` before using these functions.

use anyhow::{anyhow, Result};
use ethers::types::{Address, Chain, U256};
use log::warn;
use reqwest::Client;
use serde::Deserialize;

use crate::utils::type_conversion::{address_to_string, string_to_bool};

use crate::token_check::check_token_lock::TokenHolders;

/// Represents the generic response from the Etherscan API.
///
/// The API response is expected to contain:
/// - `status`: a string indicating the status (usually "1" for success),
/// - `message`: a message from the API,
/// - `result`: a vector containing the actual data, which is generic.
#[derive(Debug, Deserialize)]
pub struct EtherscanResponse<T> {
    status: String,
    message: String,
    result: Vec<T>,
}

/// Contains contract creation information as returned by Etherscan.
///
/// Note that most fields are prefixed with an underscore as they may not be used.
#[derive(Debug, Deserialize)]
pub struct ContractOwner {
    #[serde(rename = "contractAddress")]
    _contract_address: String,

    #[serde(rename = "contractCreator")]
    contract_creator: String,

    #[serde(rename = "txHash")]
    _tx_hash: String,

    #[serde(rename = "blockNumber")]
    _block_number: String,

    #[serde(rename = "timestamp")]
    _timestamp: String,

    #[serde(rename = "contractFactory")]
    _contract_factory: String,

    #[serde(rename = "creationBytecode")]
    _creation_byte_code: String,
}

/// Represents the contract source code information returned by Etherscan.
///
/// This struct mirrors the JSON structure of the source code details and is used internally.
#[derive(Debug, Deserialize)]
struct ContractSourceCode {
    #[serde(rename = "SourceCode")]
    source_code: String,

    #[serde(rename = "ABI")]
    _abi: String,

    #[serde(rename = "ContractName")]
    _contract_name: String,

    #[serde(rename = "CompilerVersion")]
    _compiler_version: String,

    #[serde(rename = "OptimizationUsed")]
    _optimization_used: String,

    #[serde(rename = "Runs")]
    _runs: String,

    #[serde(rename = "ConstructorArguments")]
    _constructor_arguments: String,

    #[serde(rename = "EVMVersion")]
    _evm_version: String,

    #[serde(rename = "Library")]
    _library: String,

    #[serde(rename = "LicenseType")]
    _license_type: String,

    #[serde(rename = "Proxy")]
    _proxy: String,

    #[serde(rename = "Implementation")]
    _implementation: String,

    #[serde(rename = "SwarmSource")]
    _swarm_source: String,
}

/// Represents the token information as returned by Etherscan.
///
/// This struct includes details like website, blue checkmark status,
/// and social media handles.
#[derive(Debug, Deserialize)]
pub struct TokenInfo {
    #[serde(rename = "contractAddress")]
    _contract_address: String,

    #[serde(rename = "tokenName")]
    _token_name: String,

    #[serde(rename = "symbol")]
    _symbol: String,

    #[serde(rename = "divisor")]
    _divisor: String,

    #[serde(rename = "tokenType")]
    _token_type: String,

    #[serde(rename = "totalSupply")]
    _total_supply: String,

    #[serde(rename = "blueCheckmark")]
    pub blue_checkmark: String,

    #[serde(rename = "description")]
    _description: String,

    #[serde(rename = "website")]
    pub website: String,

    #[serde(rename = "email")]
    _email: String,

    #[serde(rename = "blog")]
    _blog: String,

    #[serde(rename = "reddit")]
    _reddit: String,

    #[serde(rename = "slack")]
    _slack: String,

    #[serde(rename = "facebook")]
    _facebook: String,

    #[serde(rename = "twitter")]
    pub twitter: String,

    #[serde(rename = "bitcointalk")]
    _bitcointalk: String,

    #[serde(rename = "github")]
    _github: String,

    #[serde(rename = "telegram")]
    _telegram: String,

    #[serde(rename = "wechat")]
    _wechat: String,

    #[serde(rename = "linkedin")]
    _linkedin: String,

    #[serde(rename = "discord")]
    pub discord: String,

    #[serde(rename = "whitepaper")]
    pub whitepaper: String,

    #[serde(rename = "tokenPriceUSD")]
    _token_price_usd: String,

    #[serde(rename = "image")]
    _image: String,
}

/// Holds token web data extracted from the Etherscan token info.
///
/// This includes website URL, blue checkmark status, and social media links.
#[derive(Debug, Clone, Default)]
pub struct TokenWebData {
    pub blue_checkmark: bool,
    pub website: String,
    pub scraped_web_content: String,
    pub twitter: String,
    pub discord: String,
    pub whitepaper: String,
}

/// Fetches the token holder list from the Etherscan API for a given contract address.
///
/// # Arguments
///
/// * `contract_address` - The ERC-20 token contract address.
///
/// # Returns
///
/// A `Result` containing a vector of `TokenHolders` (with holder address and quantity in `U256`)
/// on success, or an error if the request fails or parsing the data encounters an issue.
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP request fails.
/// - The API returns a non-success status.
/// - The JSON response cannot be parsed correctly.
pub async fn get_token_holder_list(
    contract_address: Address,
    chain: &Chain,
) -> Result<Vec<TokenHolders>> {
    // Retrieve the necessary API key and convert the contract address to a string.
    let etherscan_api_key = get_etherscan_api_key()?;
    let contract_address_str = address_to_string(contract_address);

    // Get chain id from application configuration.
    let chain_id = *chain as u64;
    let etherscan_api = get_etherscan_api()?;

    // Build the Etherscan URL for fetching token holders.
    let url = format!(
        "{}?chainid={}&module=token&action=tokenholderlist&contractaddress={}&apikey={}",
        etherscan_api, chain_id, contract_address_str, etherscan_api_key
    );

    // Create a new HTTP client and send the GET request.
    let client = Client::new();
    let resp = client.get(&url).send().await?;

    // Ensure the request succeeded.
    if !resp.status().is_success() {
        return Err(anyhow!("Request failed with status: {}", resp.status()));
    }

    // Deserialize the JSON response into the expected structure.
    let parsed: EtherscanResponse<EtherscanHolderEntry> = resp.json().await?;

    // Verify that Etherscan returned a successful status.
    if parsed.status != "1" {
        return Err(anyhow!(
            "Etherscan returned status={}, message={}",
            parsed.status,
            parsed.message
        ));
    }

    // Convert the parsed JSON entries into a vector of TokenHolders.
    let mut holders = Vec::with_capacity(parsed.result.len());
    for entry in parsed.result {
        // Parse the token quantity from decimal string to U256.
        let tokens_held = U256::from_dec_str(&entry.token_holder_quantity)?;

        holders.push(TokenHolders {
            holder: entry.token_holder_address,
            quantity: tokens_held,
        });
    }

    Ok(holders)
}

/// Fetches the source code of a contract from the Etherscan API.
///
/// # Arguments
///
/// * `contract_address` - A string slice representing the contract address.
///
/// # Returns
///
/// A `Result` containing the contract source code as a `String` on success, or an error if
/// the request or parsing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP request fails.
/// - The API returns a non-success status.
/// - The JSON response cannot be parsed.
pub async fn get_source_code(contract_address: &str, chain: &Chain) -> Result<String> {
    let etherscan_api_key = get_etherscan_api_key()?;

    let chain_id = *chain as u64;
    let etherscan_api = get_etherscan_api()?;

    // Build the URL for fetching the contract source code.
    let url = format!(
        "{}?chainid={}&module=contract&action=getsourcecode&\
address={}&apikey={}",
        etherscan_api, chain_id, contract_address, etherscan_api_key
    );

    // Send the HTTP GET request.
    let client = Client::new();
    let resp = client.get(&url).send().await?;

    // Check if the response status is successful.
    if !resp.status().is_success() {
        return Err(anyhow!("Request failed with status: {}", resp.status()));
    }

    // Deserialize the JSON response.
    let parsed: EtherscanResponse<ContractSourceCode> = resp.json().await?;

    // Ensure the returned status signifies success.
    if parsed.status != "1" {
        return Err(anyhow!(
            "Etherscan returned status={}, message={}",
            parsed.status,
            parsed.message
        ));
    }

    // Return the source code from the first item in the result, if available.
    let source_code = match parsed.result.first() {
        Some(result) => result.source_code.clone(),
        None => String::new(),
    };

    Ok(source_code)
}

/// Retrieves the contract owner's address from the Etherscan API.
///
/// # Arguments
///
/// * `contract_address` - A string slice representing the contract address.
///
/// # Returns
///
/// A `Result` containing an `Option<String>`. On success, it returns `Some(contract_creator)` if found,
/// or `None` if there is no contract owner information. If the HTTP request or JSON deserialization fails,
/// then an error is returned.
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP request fails.
/// - The API returns a non-success status.
/// - The JSON response cannot be parsed.
pub async fn get_contract_owner(contract_address: &str, chain: &Chain) -> Result<Option<String>> {
    let etherscan_api_key = get_etherscan_api_key()?;

    let chain_id = *chain as u64;
    let etherscan_api = get_etherscan_api()?;

    // Build the URL for fetching the contract owner.
    let url = format!(
        "{}?chainid={}&module=contract&action=getcontractcreation&\
contractaddresses={}&apikey={}",
        etherscan_api, chain_id, contract_address, etherscan_api_key
    );

    let client = Client::new();
    let response = client.get(&url).send().await?;

    // Ensure the response is successful.
    if !response.status().is_success() {
        return Err(anyhow!("Request failed with status: {}", response.status()));
    }

    // Attempt to deserialize the JSON response.
    // On failure, log a warning and attempt to re-fetch the data.
    let parsed: EtherscanResponse<ContractOwner> = match response.json().await {
        Ok(parsed) => parsed,
        Err(error) => {
            warn!("could not decode => {}", error);

            // Retry fetching the data (this part is for testing purposes).
            let response = client.get(&url).send().await?;
            let response_text = response.text().await?;
            println!("response_text => {}", response_text);

            return Ok(None);
        }
    };

    // Verify a successful status in the returned response.
    if parsed.status != "1" {
        return Err(anyhow!(
            "Etherscan returned status={}, message={}",
            parsed.status,
            parsed.message
        ));
    }

    // Return the contract creator if available.
    match parsed.result.first() {
        Some(result) => Ok(Some(result.contract_creator.clone())),
        None => Ok(None),
    }
}

/// Fetches token information from the Etherscan API and converts it into `TokenWebData`.
///
/// # Arguments
///
/// * `contract_address` - A string slice representing the contract address.
///
/// # Returns
///
/// A `Result` containing an `Option<TokenWebData>` on success. If parsing or the HTTP request fails,
/// or if no token information is found, `None` is returned (with an appropriate error logged).
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP request fails.
/// - The API returns a non-success status.
/// - The JSON response cannot be fully parsed.
pub async fn get_token_info(contract_address: &str, chain: &Chain) -> Result<Option<TokenWebData>> {
    let etherscan_api_key = get_etherscan_api_key()?;

    let chain_id = *chain as u64;
    let etherscan_api = get_etherscan_api()?;

    // Build the URL for fetching token info.
    let url = format!(
        "{}?chainid={}&module=token&action=tokeninfo&\
contractaddress={}&apikey={}",
        etherscan_api, chain_id, contract_address, etherscan_api_key
    );

    let client = Client::new();
    let response = client.get(&url).send().await?;

    // Ensure the HTTP request was successful.
    if !response.status().is_success() {
        return Err(anyhow!("Request failed with status: {}", response.status()));
    }

    // Attempt to deserialize the JSON response.
    let parsed: EtherscanResponse<TokenInfo> = match response.json().await {
        Ok(parsed) => parsed,
        Err(error) => {
            warn!("could not decode => {}", error);

            // Retry fetching the data if needed.
            let response = client.get(&url).send().await?;
            let response_text = response.text().await?;
            println!("response_text => {}", response_text);

            return Ok(None);
        }
    };

    // Ensure the response indicates success.
    if parsed.status != "1" {
        return Err(anyhow!(
            "Etherscan returned status={}, message={}",
            parsed.status,
            parsed.message
        ));
    }

    // Convert the first token info result into TokenWebData.
    match parsed.result.first() {
        Some(result) => Ok(Some(TokenWebData {
            website: result.website.clone(),
            blue_checkmark: string_to_bool(&result.blue_checkmark)?,
            twitter: result.twitter.clone(),
            discord: result.discord.clone(),
            whitepaper: result.whitepaper.clone(),
            ..Default::default()
        })),
        None => Ok(None),
    }
}

/// Internal helper function to retrieve the Etherscan API key from the environment.
///
/// # Returns
///
/// A `Result` containing the API key as a `String` or an error if it is not set.
fn get_etherscan_api_key() -> anyhow::Result<String> {
    let etherscan_key =
        std::env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY is not set in .env");

    Ok(etherscan_key)
}

/// Internal helper function to retrieve the Etherscan API URL from the environment.
///
/// # Returns
///
/// A `Result` containing the API URL as a `String` or an error if it is not set.
fn get_etherscan_api() -> anyhow::Result<String> {
    let etherscan_key = std::env::var("ETHERSCAN_API").expect("ETHERSCAN_API is not set in .env");

    Ok(etherscan_key)
}

/// Internal struct representing a token holder entry as returned by the Etherscan API.
#[derive(Debug, Deserialize)]
struct EtherscanHolderEntry {
    #[serde(rename = "TokenHolderAddress")]
    token_holder_address: String,

    #[serde(rename = "TokenHolderQuantity")]
    token_holder_quantity: String,
}
