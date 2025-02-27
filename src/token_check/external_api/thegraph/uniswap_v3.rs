use anyhow::Result;
use ethers::types::{Address, U256};
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

use crate::token_check::check_token_lock::TokenHolders;

use super::shared::{get_thegraph_api_key, THEGRAPH_BASE_URL, UNISWAP_V3_SUBGRAPH_ID};

/// Alternative URL if using a gateway
// const THEGRAPH_GATEWAY_URL: &str = "https://gateway.thegraph.com/api";
// const UNISWAP_V3_SUBGRAPH_ID: &str = "5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV";

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
    _id: String,
    /// The owner's address
    owner: String,
    /// The liquidity amount as a string (to handle large numbers)
    liquidity: String,
    /// Token0 information
    #[serde(rename = "token0")]
    _token0: Token,
    /// Token1 information
    #[serde(rename = "token1")]
    _token1: Token,
    /// Pool information
    pool: Pool,
    /// Information about lower tick boundary
    #[serde(rename = "tickLower")]
    _tick_lower: Tick,
    /// Information about upper tick boundary
    #[serde(rename = "tickUpper")]
    _tick_upper: Tick,
    /// Amount of token0 deposited
    #[serde(rename = "depositedToken0")]
    _deposited_token0: String,
    /// Amount of token1 deposited
    #[serde(rename = "depositedToken1")]
    _deposited_token1: String,
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
    _fee_tier: String,
    /// Current tick
    tick: String,
    /// Current sqrt price
    #[serde(rename = "sqrtPrice")]
    _sqrt_price: String,
}

/// Structure representing tick information
#[derive(Debug, Deserialize)]
struct Tick {
    /// Tick index
    #[serde(rename = "tickIdx")]
    _tick_idx: String,
}

pub async fn fetch_uniswap_v3_positions(pool_address: Address) -> Result<Vec<TokenHolders>> {
    // Prepare the where clause based on optional parameters
    let mut where_clause = String::from("liquidity_gt: \"0\"");

    // Construct the GraphQL query for positions
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

    // Retrieve the API key for TheGraph; propagates an error if it's missing.
    let thegraph_api_key = get_thegraph_api_key()?;
    // Construct the full GraphQL URL including the API key and subgraph ID.
    let graphql_url = format!(
        "{}/{}/subgraphs/id/{}",
        THEGRAPH_BASE_URL, thegraph_api_key, UNISWAP_V3_SUBGRAPH_ID
    );

    // Send the POST request and ensure the HTTP status is 200 (OK).
    let resp = client
        .post(graphql_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    // Deserialize the JSON response
    let parsed: GraphQlResponse<PositionsData> = resp.json().await?;

    // Process each position
    let mut result_vec = Vec::<TokenHolders>::new();
    for position in parsed.data.positions {
        // Convert the liquidity string to U256
        let liquidity = match U256::from_str(&position.liquidity) {
            Ok(val) => val,
            Err(_) => continue, // Skip this position if parsing fails
        };

        // Add to the results vector
        result_vec.push(TokenHolders {
            holder: position.owner,
            quantity: liquidity,
        });
    }

    Ok(result_vec)
}
