use std::cmp::Ordering;

use anyhow::anyhow;
use ethers::{abi::Address, types::Chain};

use crate::{app_config::DEXES, data::dex::Dex};

use super::thegraph::{
    uniswap_v2::get_top_uniswap_v2_pair_by_token_and_chain,
    uniswap_v3::get_top_uniswap_v3_pool_by_token_and_chain,
};

#[derive(Clone, Default, Debug)]
pub struct TokenDexData {
    pub dex: Dex,
    pub pair_address: Address, // pair or pool address for token
    pub token_0: Address,
    pub token_1: Address,
    /// A flag indicating whether the token is the first token (token_0)
    /// in the Uniswap pair; if false the token is token_1.
    pub is_token_0: bool,
    pub base_token_address: String, // address of base token provides the pair liquidity, typically WETH,
    pub base_token_symbol: String, // address of base token provides the pair liquidity, typically WETH,
    // USDC , DAI, or WBTC. the liquidity pair is token / base_token
    pub fee: u32,
    pub liquidity_in_usd: f64,
    pub create_at_block_number: u64,
}

pub const PAIR_CREATED_SIGNATURE: &str = "PairCreated(address,address,address,uint256)";
pub const POOLCREATED_SIGNATURE: &str = "PoolCreated(address,address,uint24,int24,address)";

pub async fn find_top_dex_for_token(
    token_address: Address,
    chain: &Chain,
) -> anyhow::Result<Option<TokenDexData>> {
    let mut full_dex_data = Vec::<TokenDexData>::new();

    for dex in DEXES {
        let dex_data = match dex {
            Dex::UniswapV3 => {
                get_top_uniswap_v3_pool_by_token_and_chain(token_address, chain).await?
            }
            Dex::UniswapV2 => {
                get_top_uniswap_v2_pair_by_token_and_chain(token_address, chain).await?
            }
            _ => return Err(anyhow!("get_all_dex_data_for_token: dex not found")),
        };

        if let Some(data) = dex_data {
            full_dex_data.push(data);
        }
    }

    if !full_dex_data.is_empty() && full_dex_data.len() > 1 {
        full_dex_data = sort_token_dex_data_vec(full_dex_data);
    } else if full_dex_data.is_empty() {
        return Ok(None);
    }

    // first element of full_dex_data has highest liquidity , pull and return that
    let top_dex = full_dex_data
        .first()
        .expect("get_all_dex_data_for_token: could not extract top dex data");

    Ok(Some(top_dex.clone()))
}

pub fn sort_token_dex_data_vec(mut dex_data_array: Vec<TokenDexData>) -> Vec<TokenDexData> {
    dex_data_array.sort_by(|a, b| {
        b.liquidity_in_usd
            .partial_cmp(&a.liquidity_in_usd)
            .unwrap_or(Ordering::Equal)
    });
    dex_data_array
}
