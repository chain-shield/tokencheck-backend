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
    pools: Vec<Pool>,
}

#[derive(Debug, Deserialize)]
pub struct Pool {
    id: String,
    token0: Token,
    token1: Token,
    #[serde(rename = "feeTier")]
    fee_tier: String,
    #[serde(rename = "totalValueLockedUSD")]
    total_value_locked_usd: String,
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

pub async fn get_top_uniswap_v3_pool_by_token_and_chain(
    token_address: Address,
    chain: &Chain,
) -> Result<Option<TokenDexData>> {
    match chain {
        Chain::Mainnet => {
            get_top_uniswap_v3_pool_for_token(token_address, SubGraph::UniswapV3Mainnet).await
        }

        Chain::Base => {
            get_top_uniswap_v3_pool_for_token(token_address, SubGraph::UniswapV3Base).await
        }
        _ => {
            return Err(anyhow!(
                "get_top_uniswap_v3_pool_by_token_and_chain: chain not found"
            ))
        }
    }
}

/// Get top Uniswap V3 pool for a token (either as token0 or token1)
///
/// # Arguments
///
/// * `token_address` - The Ethereum address of the token
/// * `limit_per_query` - Maximum number of pools to return per direction (token0/token1)
///
/// # Returns
///
/// A Result containing a Vec of Pool structs if successful
pub async fn get_top_uniswap_v3_pool_for_token(
    token_address: Address,
    subgraph: SubGraph,
) -> Result<Option<TokenDexData>> {
    let token_address = address_to_string(token_address);

    // Get pools where token is token0
    let token0_pool =
        get_uniswap_v3_pools_by_position(&token_address, TokenPosition::Token0, subgraph).await?;

    // Get pools where token is token1
    let token1_pool =
        get_uniswap_v3_pools_by_position(&token_address, TokenPosition::Token1, subgraph).await?;

    if token0_pool.is_empty() && token1_pool.is_empty() {
        return Ok(None);
    }

    // Combine both sets of pools
    let mut all_pools = Vec::with_capacity(2);
    all_pools.extend(token0_pool);
    all_pools.extend(token1_pool);

    // Sort by TVL descending
    all_pools.sort_by(|a, b| {
        let a_tvl = a.total_value_locked_usd.parse::<f64>().unwrap_or(0.0);
        let b_tvl = b.total_value_locked_usd.parse::<f64>().unwrap_or(0.0);
        b_tvl
            .partial_cmp(&a_tvl)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_pool = all_pools
        .first()
        .expect("could not get top pool from uniswap v3");

    let is_token_0 = token_address.to_lowercase() == top_pool.token0.id;
    let (base_token_address, base_token_symbol) = if is_token_0 {
        (top_pool.token1.id.clone(), top_pool.token1.symbol.clone())
    } else {
        (top_pool.token0.id.clone(), top_pool.token0.symbol.clone())
    };

    let token_0 = top_pool.token0.id.parse::<Address>()?;
    let token_1 = top_pool.token1.id.parse::<Address>()?;

    let liquidity_in_usd = top_pool
        .total_value_locked_usd
        .parse::<f64>()
        .unwrap_or_default();

    let fee = top_pool.fee_tier.parse::<u32>()?;
    let create_at_block_number = top_pool.created_at_block_number.parse::<u64>()?;

    Ok(Some(TokenDexData {
        dex: Dex::UniswapV3,
        pair_address: top_pool.id.parse()?,
        token_0,
        token_1,
        is_token_0,
        base_token_address,
        base_token_symbol,
        liquidity_in_usd,
        fee,
        create_at_block_number,
    }))
}

/// Fetches Uniswap V3 pools where the specified token is in the specified position
///
/// # Arguments
///
/// * `token_address` - The Ethereum address of the token
/// * `position` - Whether the token is token0 or token1
/// * `limit` - Maximum number of pools to return
///
/// # Returns
///
/// A Result containing a Vec of Pool structs if successful
pub async fn get_uniswap_v3_pools_by_position(
    token_address: &str,
    position: TokenPosition,
    subgraph: SubGraph,
) -> Result<Vec<Pool>> {
    let client = Client::new();

    // Normalize the token address to lowercase
    let token_address = token_address.to_lowercase();

    // Get the field name based on position
    let field_name = position.field_name();

    // Create the GraphQL query
    let query = format!(
        r#"
    query GetPools($token_address: String!) {{
        pools(
            where: {{
                {}: {{ id: $token_address }},
                totalValueLockedUSD_gt: "0"
            }},
            orderBy: totalValueLockedUSD,
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
            feeTier
            totalValueLockedUSD
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

    // let response_text = response.text().await?;
    // return Err(anyhow!("could not deserialize => {}", response_text));

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

    Ok(graphql_response.data.pools)
}
