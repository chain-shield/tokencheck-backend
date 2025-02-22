use crate::token_check::check_token_lock::TokenHolders;
use crate::utils::type_conversion::{address_to_string, f64_to_u256};
use anyhow::Result;
use ethers::types::Address;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

/// Base URL for TheGraph API.
const THEGRAPH_BASE_URL: &str = "https://gateway.thegraph.com/api";
/// Uniswap V2 subgraph ID used for querying liquidity positions.
const UNISWAP_V2_SUBGRAPH_ID: &str = "EYCKATKGBKLWvSfwvBjzfCBmGwYNdVkduYXVivCsLRFu";

/// Generic structure for parsing GraphQL responses.
///
/// # Type Parameters
///
/// - `T`: The type into which the GraphQL response `data` field will be deserialized.
#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: T,
}

/// Structure representing the data block containing liquidity positions.
#[derive(Debug, Deserialize)]
struct LiquidityPositionData {
    #[serde(rename = "liquidityPositions")]
    liquidity_positions: Vec<LiquidityPosition>,
}

/// Structure representing an individual liquidity position.
///
/// This includes user information and the token balance in string format.
#[derive(Debug, Deserialize)]
struct LiquidityPosition {
    /// The user holding the liquidity, represented by their address.
    user: LiquidityUser,
    /// The liquidity token balance in subgraph's decimal format.
    #[serde(rename = "liquidityTokenBalance")]
    liquidity_token_balance: String,
}

/// Structure representing a liquidity provider (user).
#[derive(Debug, Deserialize)]
struct LiquidityUser {
    /// The user's address identifier.
    ///
    /// Note: This is typically provided as a lowercase hexadecimal string.
    id: String,
}

/// Fetches the liquidity positions for a given Uniswap V2 pair from TheGraph API.
///
/// This function sends a GraphQL query to retrieve liquidity positions for the specified pair,
/// parses the token balance, converts it to a U256 format using a custom conversion function, and
/// returns a list of `TokenHolders` containing the balance information.
///
/// # Arguments
///
/// - `pair_address`: The Ethereum address of the token pair for which liquidity positions are queried.
///
/// # Returns
///
/// - `Result<Vec<TokenHolders>>`: A vector of `TokenHolders` on success, or an error in case of failure.
///
/// # Errors
///
/// This function returns an error if:
/// - TheGraph API call fails or returns a non-successful HTTP status.
/// - The response parsing fails.
/// - Parsing or converting the liquidity token balance fails.
pub async fn fetch_uniswap_lp_holders(pair_address: Address) -> Result<Vec<TokenHolders>> {
    // Construct the GraphQL query for liquidity positions filtered by the given pair address.
    let query = format!(
        r#"{{
            liquidityPositions(where: {{ pair: "{pair}" }}, first: 100) {{
                user {{ id }}
                liquidityTokenBalance
            }}
        }}"#,
        pair = address_to_string(pair_address).to_lowercase(),
    );

    // Create an HTTP client instance.
    let client = Client::new();
    // Prepare the request body containing the GraphQL query.
    let body = HashMap::from([("query", query)]);

    // Retrieve the API key for TheGraph; propagates an error if it's missing.
    let thegraph_api_key = get_thegraph_api_key()?;
    // Construct the full GraphQL URL including the API key and subgraph ID.
    let graphql_url = format!(
        "{}/{}/subgraphs/id/{}",
        THEGRAPH_BASE_URL, thegraph_api_key, UNISWAP_V2_SUBGRAPH_ID
    );

    // Send the POST request and ensure the HTTP status is 200 (OK).
    let resp = client
        .post(graphql_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    // Deserialize the JSON response into the expected GraphQlResponse structure.
    let parsed: GraphQlResponse<LiquidityPositionData> = resp.json().await?;

    // Process each liquidity position, parsing the balance and converting it to U256.
    let mut result_vec = Vec::<TokenHolders>::new();
    for lp in parsed.data.liquidity_positions {
        // Convert the token balance from its string representation to f64.
        let liquidity_balance = lp.liquidity_token_balance.parse::<f64>()?;
        // Convert the f64 balance to U256 using the provided conversion function.
        let liquidity_balance_u256 = f64_to_u256(liquidity_balance)?;
        // Assemble the TokenHolders struct and add it to the results.
        result_vec.push(TokenHolders {
            holder: lp.user.id,
            quantity: liquidity_balance_u256,
        });
    }

    Ok(result_vec)
}

/// Retrieves TheGraph API key from the environment.
///
/// The function expects the `THEGRAPH_API_KEY` environment variable to be set. If it's not set,
/// an error is returned.
///
/// # Returns
///
/// - `anyhow::Result<String>`: The API key as a string, or an error if the environment variable is not set.
///
/// # Errors
///
/// Returns an error if the `THEGRAPH_API_KEY` environment variable is missing.
fn get_thegraph_api_key() -> anyhow::Result<String> {
    // Retrieve the API key from the environment variable, propagating the error if not set.
    let thegraph_key = std::env::var("THEGRAPH_API_KEY")?;
    Ok(thegraph_key)
}
