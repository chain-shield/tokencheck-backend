use ethers::abi::Address;

use crate::{
    data::{dex::Dex, token_data::ERC20Token},
    token_check::check_token_lock::TokenHolders,
};

use super::{uniswap_v2::fetch_uniswap_v2_lp_holders, uniswap_v3::fetch_uniswap_v3_positions};

/// Base URL for TheGraph API.
pub const THEGRAPH_BASE_URL: &str = "https://gateway.thegraph.com/api";
/// Uniswap V2 & V3     subgraph ID used for querying liquidity positions.
pub const UNISWAP_V2_SUBGRAPH_ID: &str = "EYCKATKGBKLWvSfwvBjzfCBmGwYNdVkduYXVivCsLRFu";
pub const UNISWAP_V3_SUBGRAPH_ID: &str = "5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV";

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
    match token.token_dex.dex {
        Dex::UniswapV2 => fetch_uniswap_v2_lp_holders(token.token_dex.pair_or_pool_address).await,
        Dex::UniswapV3 => fetch_uniswap_v3_positions(token.token_dex.pair_or_pool_address).await,
        _ => Ok(Vec::<TokenHolders>::new()),
    }
}
