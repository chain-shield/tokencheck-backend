use anyhow::Result;
use ethers::types::{Address, U256};
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

use crate::{
    app_config::{THEGRAPH_BASE_URL, UNISWAP_V3_MAINNET_SUBGRAPH_ID},
    token_check::check_token_lock::TokenHolders,
    utils::type_conversion::address_to_string,
};

use super::shared::get_thegraph_api_key;

/// Generic structure for parsing GraphQL responses.
#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: T,
}

/// Structure representing the data block containing positions.
#[derive(Debug, Deserialize)]
struct PositionsData {
    positions: Vec<Position>,
}

/// Structure representing an individual liquidity position.
#[derive(Debug, Deserialize)]
struct Position {
    /// The position ID
    #[serde(rename = "id")]
    _id: Option<String>,
    /// The owner's address
    owner: String,
    /// The liquidity amount as a string (to handle large numbers)
    liquidity: String,
    /// Token0 information
    #[serde(rename = "token0")]
    _token0: Option<Token>,
    /// Token1 information
    #[serde(rename = "token1")]
    _token1: Option<Token>,
    /// Pool information
    pool: Pool,
    /// Information about lower tick boundary
    #[serde(rename = "tickLower")]
    _tick_lower: Option<Tick>,
    /// Information about upper tick boundary
    #[serde(rename = "tickUpper")]
    _tick_upper: Option<Tick>,
    /// Amount of token0 deposited
    #[serde(rename = "depositedToken0")]
    _deposited_token0: Option<String>,
    /// Amount of token1 deposited
    #[serde(rename = "depositedToken1")]
    _deposited_token1: Option<String>,
}

/// Structure representing token information
#[derive(Debug, Deserialize)]
struct Token {
    /// Token address
    id: String,
    /// Token symbol
    symbol: String,
    /// Token decimals
    decimals: String,
}

/// Structure representing pool information
#[derive(Debug, Deserialize)]
struct Pool {
    /// Pool address
    id: String,
    /// Fee tier
    #[serde(rename = "feeTier")]
    _fee_tier: Option<String>,
    /// Current tick
    tick: Option<String>,
    /// Current sqrt price
    #[serde(rename = "sqrtPrice")]
    _sqrt_price: Option<String>,
}

/// Structure representing tick information
#[derive(Debug, Deserialize)]
struct Tick {
    /// Tick index
    #[serde(rename = "tickIdx")]
    _tick_idx: String,
}

/// Fetches Uniswap V3 liquidity positions for a specific pool.
///
/// This function queries TheGraph API to retrieve information about liquidity providers
/// for a given Uniswap V3 pool. It returns a vector of `TokenHolders` containing the
/// address of each liquidity provider and their liquidity amount.
///
/// # Arguments
///
/// * `pool_address` - The Ethereum address of the Uniswap V3 pool
///
/// # Returns
///
/// * `Result<Vec<TokenHolders>>` - A vector of token holders with their liquidity amounts
///   or an error if the API request fails
pub async fn fetch_uniswap_v3_positions(pool_address: Address) -> Result<Vec<TokenHolders>> {
    // Convert the pool address to a lowercase hex string without '0x' prefix
    let pool_id = address_to_string(pool_address);

    // Create a filter to only get positions with positive liquidity in the specified pool
    let where_clause = format!("liquidity_gt: \"0\", pool: \"{}\"", pool_id);

    // Construct the GraphQL query for positions
    // Limited to 1000 results, ordered by liquidity in descending order
    let query = format!(
        r#"{{
            positions(
                first: 1000,
                orderBy: liquidity,
                orderDirection: desc,
                where: {{ {where_clause} }}
            ) {{
                id
                owner
                liquidity
                token0 {{
                    id
                    symbol
                    decimals
                }}
                token1 {{
                    id
                    symbol
                    decimals
                }}
                pool {{
                    id
                    feeTier
                    tick
                    sqrtPrice
                }}
                tickLower {{
                    tickIdx
                }}
                tickUpper {{
                    tickIdx
                }}
                depositedToken0
                depositedToken1
            }}
        }}"#,
        where_clause = where_clause
    );

    // Create an HTTP client instance
    let client = Client::new();

    // Prepare the request body containing the GraphQL query
    let body = HashMap::from([("query", query)]);

    // Retrieve the API key for TheGraph
    let thegraph_api_key = get_thegraph_api_key()?;

    // Construct the full GraphQL URL including the API key and subgraph ID
    let graphql_url = format!(
        "{}/{}/subgraphs/id/{}",
        THEGRAPH_BASE_URL, thegraph_api_key, UNISWAP_V3_MAINNET_SUBGRAPH_ID
    );

    // Send the POST request and handle HTTP errors
    let resp = client
        .post(graphql_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    // Deserialize the JSON response into our data structures
    let parsed: GraphQlResponse<PositionsData> = resp.json().await?;

    // Process each position and convert to TokenHolders format
    let mut result_vec = Vec::<TokenHolders>::new();
    for position in parsed.data.positions {
        // Convert the liquidity string to U256, skipping positions where conversion fails
        let liquidity = match U256::from_str(&position.liquidity) {
            Ok(val) => val,
            Err(_) => continue,
        };

        // Add to the results vector
        result_vec.push(TokenHolders {
            holder: position.owner,
            quantity: liquidity,
        });
    }

    Ok(result_vec)
}
