use anyhow::{anyhow, Context, Result};
use ethers::types::{Address, Chain};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    app_config::THEGRAPH_BASE_URL, data::dex::Dex, dex::dex_data::TokenDexData,
    token_check::external_api::thegraph::shared::get_thegraph_api_key,
    utils::type_conversion::address_to_string,
};

use super::shared::{SubGraph, TokenPosition};

// Structs to deserialize the GraphQL response
#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    pairs: Vec<Pair>,
}

#[derive(Debug, Deserialize)]
pub struct Pair {
    id: String,
    token0: Token,
    token1: Token,
    #[serde(rename = "reserveUSD")]
    reserve_usd: String,
    #[serde(rename = "createdAtBlockNumber")]
    created_at_block_number: String,
}

#[derive(Debug, Deserialize)]
struct Token {
    id: String,
    symbol: String,
}

// Input for the GraphQL query
#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    variables: Variables,
}

#[derive(Debug, Serialize)]
struct Variables {
    token_address: String,
}

pub async fn get_top_uniswap_v2_pool_by_token_and_chain(
    token_address: Address,
    chain: &Chain,
) -> Result<Option<TokenDexData>> {
    match chain {
        Chain::Mainnet => {
            get_top_uniswap_v2_pair_for_token(token_address, SubGraph::UniswapV2Mainnet).await
        }

        Chain::Base => {
            get_top_uniswap_v2_pair_for_token(token_address, SubGraph::UniswapV2Base).await
        }
        _ => {
            return Err(anyhow!(
                "get_top_uniswap_v2_pool_by_token_and_chain: chain not found"
            ))
        }
    }
}

/// Get top Uniswap V2 pair for a token (either as token0 or token1)
///
/// # Arguments
///
/// * `token_address` - The Ethereum address of the token
///
/// # Returns
///
/// A Result containing an Option<TokenDexData> - None if no pairs are found
pub async fn get_top_uniswap_v2_pair_for_token(
    token_address: Address,
    subgraph: SubGraph,
) -> Result<Option<TokenDexData>> {
    let token_address = address_to_string(token_address);
    // Get pairs where token is token0
    let token0_pairs =
        get_uniswap_v2_pairs_by_position(&token_address, TokenPosition::Token0, subgraph).await?;

    // Get pairs where token is token1
    let token1_pairs =
        get_uniswap_v2_pairs_by_position(&token_address, TokenPosition::Token1, subgraph).await?;

    if token0_pairs.is_empty() && token1_pairs.is_empty() {
        return Ok(None);
    }

    // Combine both sets of pairs
    let mut all_pairs = Vec::with_capacity(2);
    all_pairs.extend(token0_pairs);
    all_pairs.extend(token1_pairs);

    // Sort by reserve USD descending
    all_pairs.sort_by(|a, b| {
        let a_reserve = a.reserve_usd.parse::<f64>().unwrap_or(0.0);
        let b_reserve = b.reserve_usd.parse::<f64>().unwrap_or(0.0);
        b_reserve
            .partial_cmp(&a_reserve)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Get the top pair by liquidity
    let top_pair = all_pairs
        .first()
        .expect("could not get top pair from uniswap v2");

    // Determine if the token is token0 or token1 in the pair
    let is_token_0 = token_address.to_lowercase() == top_pair.token0.id;

    // Get the base token (the other token in the pair)
    let (base_token_address, base_token_symbol) = if is_token_0 {
        (top_pair.token1.id.clone(), top_pair.token1.symbol.clone())
    } else {
        (top_pair.token0.id.clone(), top_pair.token0.symbol.clone())
    };

    // Parse addresses
    let token_0 = top_pair.token0.id.parse::<Address>()?;
    let token_1 = top_pair.token1.id.parse::<Address>()?;

    // Parse liquidity
    let liquidity_in_usd = top_pair.reserve_usd.parse::<f64>().unwrap_or_default();

    // Parse creation block number
    let create_at_block_number = top_pair.created_at_block_number.parse::<u64>()?;

    // Create the TokenDexData
    Ok(Some(TokenDexData {
        dex: Dex::UniswapV2,
        pair_address: top_pair.id.parse()?,
        token_0,
        token_1,
        is_token_0,
        base_token_address,
        base_token_symbol,
        liquidity_in_usd,
        fee: 3000, // Uniswap V2 has a fixed 0.3% fee (represented as 3000 basis points)
        create_at_block_number,
    }))
}

/// Fetches Uniswap V2 pairs where the specified token is in the specified position
///
/// # Arguments
///
/// * `token_address` - The Ethereum address of the token
/// * `position` - Whether the token is token0 or token1
///
/// # Returns
///
/// A Result containing a Vec of Pair structs if successful
pub async fn get_uniswap_v2_pairs_by_position(
    token_address: &str,
    position: TokenPosition,
    subgraph: SubGraph,
) -> Result<Vec<Pair>> {
    let client = Client::new();

    // Normalize the token address to lowercase
    let token_address = token_address.to_lowercase();

    // Get the field name based on position
    let field_name = position.field_name();

    // Create the GraphQL query
    let query = format!(
        r#"
    query GetPairs($token_address: String!) {{
        pairs(
            where: {{
                {}: {{ id: $token_address }},
                reserveUSD_gt: "0"
            }},
            orderBy: reserveUSD,
            orderDirection: desc,
            first: 1
        ) {{
            id
            token0 {{
                id
                symbol
            }}
            token1 {{
                id
                symbol
            }}
            reserveUSD
            createdAtBlockNumber
        }}
    }}
    "#,
        field_name
    );

    // Retrieve the API key for TheGraph; propagates an error if it's missing.
    let thegraph_api_key = get_thegraph_api_key()?;

    // Construct the full GraphQL URL including the API key and subgraph ID.
    let graphql_url = format!(
        "{}/{}/subgraphs/id/{}",
        THEGRAPH_BASE_URL,
        thegraph_api_key,
        subgraph.value()
    );

    // Prepare the request body
    let request_body = GraphQLRequest {
        query,
        variables: Variables { token_address },
    };

    // Make the request to TheGraph API
    let response = client
        .post(graphql_url)
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to TheGraph API")?;

    // Check if the request was successful
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("API request failed with status {}: {}", status, error_text);
    }

    // Parse the response
    let graphql_response = response
        .json::<GraphQLResponse>()
        .await
        .context("Failed to deserialize GraphQL response")?;

    Ok(graphql_response.data.pairs)
}
