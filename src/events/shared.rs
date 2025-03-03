use anyhow::anyhow;
use ethers::{
    abi::{Address, RawLog},
    core::types::Filter,
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, Chain, Log, H256, U256},
    utils::keccak256,
};
use std::sync::Arc;

use crate::data::{
    chain_data::CHAIN_DATA,
    dex::{Dex, UNISWAP_V2_FEE},
};

use super::{
    dex_liquidity::{get_liquidity_uniswap_v2, get_liquidity_uniswap_v3},
    uniswap::{decode_uniswap_v2_pair_created_event, decode_uniswap_v3_pool_created_event},
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
    pub fee: u32,
    pub liquidity: u128,
}

pub const PAIR_CREATED_SIGNATURE: &str = "PairCreated(address,address,address,uint256)";
pub const POOLCREATED_SIGNATURE: &str = "PoolCreated(address,address,uint24,int24,address)";

pub async fn get_token_dex_data(
    token_address: Address,
    chain: &Chain,
    dex: &Dex,
    event_signature: &str,
    client: &Arc<Provider<Ws>>,
    from_block: Option<BlockNumber>,
    to_block: Option<BlockNumber>,
) -> anyhow::Result<Vec<TokenDexData>> {
    // Get factory address and Create a signature hash for the event
    let factory_address = get_factory_address(dex, chain)?;
    let event_signature = get_signature_hash(event_signature);

    // get deployment block for uniswap
    let deployment_block = get_deployment_block_for_dex_(dex, chain);

    // The token address could be in either topic1 (token0) or topic2 (token1)
    // So we create two filters and combine the results

    // Filter for token0 = our token
    let filter_token0 = Filter::new()
        .address(factory_address)
        .topic0(event_signature)
        .topic1(ethers::types::ValueOrArray::Value(H256::from(
            token_address,
        )))
        .from_block(from_block.unwrap_or(deployment_block.into()))
        .to_block(to_block.unwrap_or(BlockNumber::Latest));

    // Filter for token1 = our token
    let filter_token1 = Filter::new()
        .address(factory_address)
        .topic0(event_signature)
        .topic2(ethers::types::ValueOrArray::Value(H256::from(
            token_address,
        )))
        .from_block(from_block.unwrap_or(deployment_block.into()))
        .to_block(to_block.unwrap_or(BlockNumber::Latest));

    // Get logs for both filters
    let logs_token0 = client.get_logs(&filter_token0).await?;
    let logs_token1 = client.get_logs(&filter_token1).await?;
    let all_logs = [logs_token1, logs_token0].concat();

    // Combine and parse the logs
    let mut pools = Vec::<TokenDexData>::new();

    // Parse logs where token is token0 or token1
    for log in all_logs {
        let token_pair_event =
            extract_token_dex_data_from_event_log(token_address, &log, dex, client).await?;
        pools.push(token_pair_event);
    }

    Ok(pools)
}

pub async fn extract_token_dex_data_from_event_log(
    token_address: Address,
    log: &Log,
    dex: &Dex,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<TokenDexData> {
    match dex {
        Dex::UniswapV3 => {
            let pool_event = decode_uniswap_v3_pool_created_event(&log)?;

            let dex_data = TokenDexData {
                dex: dex.clone(),
                pair_address: pool_event.pool,
                token_0: pool_event.token0,
                token_1: pool_event.token1,
                fee: pool_event.fee,
                is_token_0: pool_event.token0 == token_address,
                ..Default::default()
            };

            let liquidity = get_liquidity_uniswap_v3(&dex_data, client).await?;

            //update with liquidity and return
            Ok(TokenDexData {
                liquidity,
                ..dex_data
            })
        }
        Dex::UniswapV2 => {
            let pair_event = decode_uniswap_v2_pair_created_event(&log)?;

            let dex_data = TokenDexData {
                dex: dex.clone(),
                pair_address: pair_event.pair,
                token_0: pair_event.token0,
                token_1: pair_event.token1,
                fee: UNISWAP_V2_FEE,
                is_token_0: pair_event.token0 == token_address,
                ..Default::default()
            };

            let liquidity = get_liquidity_uniswap_v2(&dex_data, client).await?;

            //update with liquidity and return
            Ok(TokenDexData {
                liquidity,
                ..dex_data
            })
        }
        _ => return Err(anyhow!("unsupported dex")),
    }
}

pub fn get_factory_address(dex: &Dex, chain: &Chain) -> anyhow::Result<Address> {
    let factory_address = match dex {
        Dex::UniswapV3 => CHAIN_DATA
            .get_address(chain)
            .uniswap_v3_factory
            .clone()
            .parse::<Address>()?,
        Dex::UniswapV2 => CHAIN_DATA
            .get_address(chain)
            .uniswap_v3_factory
            .clone()
            .parse::<Address>()?,
        _ => return Err(anyhow!("unsupported dex")),
    };

    Ok(factory_address)
}

pub fn get_signature_hash(interface: &str) -> H256 {
    H256::from(keccak256(interface.as_bytes()))
}

pub fn get_deployment_block_for_dex_(dex: &Dex, chain: &Chain) -> u64 {
    match chain {
        Chain::Mainnet => match dex {
            Dex::UniswapV2 => 10_000_000,
            Dex::UniswapV3 => 12_000_000,
            _ => 10_000_000,
        },
        Chain::Base => match dex {
            Dex::UniswapV2 => 100_000,
            Dex::UniswapV3 => 100_000,
            _ => 100_000,
        },
        _ => 100_000,
    }
}
