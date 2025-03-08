use crate::{
    data::{dex::Dex, token_data::ERC20Token},
    token_check::check_token_lock::TokenHolders,
};

use super::{uniswap_v2::fetch_uniswap_v2_lp_holders, uniswap_v3::fetch_uniswap_v3_positions};

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
pub fn get_thegraph_api_key() -> anyhow::Result<String> {
    // Retrieve the API key from the environment variable, propagating the error if not set.
    let thegraph_key = std::env::var("THEGRAPH_API_KEY")?;
    Ok(thegraph_key)
}

/// Fetches liquidity pool holders for a given token.
///
/// This function retrieves the holders of liquidity pool tokens based on the token's
/// associated DEX (Decentralized Exchange). It supports Uniswap V2 and Uniswap V3.
///
/// # Arguments
///
/// * `token` - Reference to an ERC20Token containing DEX information
///
/// # Returns
///
/// * `anyhow::Result<Vec<TokenHolders>>` - A list of token holders or an error
///
/// # Errors
///
/// Returns an error if:
/// - The token has no DEX information
/// - The API request to fetch holders fails
/// - There's an issue parsing the response
pub async fn fetch_lp_holders(token: &ERC20Token) -> anyhow::Result<Vec<TokenHolders>> {
    let token_dex = token
        .clone()
        .token_dex
        .ok_or_else(|| anyhow::anyhow!("No token DEX information found"))?;

    match token_dex.dex {
        Dex::UniswapV2 => fetch_uniswap_v2_lp_holders(token_dex.pair_address).await,
        Dex::UniswapV3 => fetch_uniswap_v3_positions(token_dex.pair_address).await,
        // Return empty vector for unsupported DEXes
        _ => Ok(Vec::<TokenHolders>::new()),
    }
}
