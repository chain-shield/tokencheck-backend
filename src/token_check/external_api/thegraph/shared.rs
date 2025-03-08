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

pub async fn fetch_lp_holders(token: &ERC20Token) -> anyhow::Result<Vec<TokenHolders>> {
    let token_dex = token
        .clone()
        .token_dex
        .expect("is_liquidity_locked: no token dex found");
    match token_dex.dex {
        Dex::UniswapV2 => fetch_uniswap_v2_lp_holders(token_dex.pair_address).await,
        Dex::UniswapV3 => fetch_uniswap_v3_positions(token_dex.pair_address).await,
        _ => Ok(Vec::<TokenHolders>::new()),
    }
}
