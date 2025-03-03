use anyhow::anyhow;
use ethers::{
    abi::{Address, RawLog},
    core::types::Filter,
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, Chain, Log, H256, U256},
    utils::keccak256,
};
use std::sync::Arc;

use crate::data::{chain_data::CHAIN_DATA, dex::Dex};

use super::uniswap_v3::{decode_poolcreated_event, PoolCreatedEvent};

pub const PAIR_CREATED_SIGNATURE: &str = "PairCreated(address,address,address,uint256)";
pub const POOLCREATED_SIGNATURE: &str = "PoolCreated(address,address,uint24,int24,address)";

pub async fn get_uniswap_v3_pools_for_token(
    client: &Arc<Provider<Ws>>,
    token_address: Address,
    chain: &Chain,
    from_block: Option<BlockNumber>,
    to_block: Option<BlockNumber>,
) -> anyhow::Result<Vec<PoolCreatedEvent>> {
    let factory_address = CHAIN_DATA
        .get_address(chain)
        .uniswap_v3_factory
        .clone()
        .parse::<Address>()?;

    // Create a signature hash for the event
    let event_signature = get_signature_hash(POOLCREATED_SIGNATURE);

    // The token address could be in either topic1 (token0) or topic2 (token1)
    // So we create two filters and combine the results

    // get deployment block for uniswap
    let deployment_block = get_deployment_block_for_dex_(&Dex::UniswapV3, &chain);

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
    let mut pools = Vec::new();

    // Parse logs where token is token0 or token1
    for log in all_logs {
        let pool_created_event = decode_poolcreated_event(&log)?;

        pools.push(pool_created_event);
    }

    Ok(pools)
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
