use anyhow::anyhow;
use ethers::{
    abi::{Address, RawLog},
    core::types::Filter,
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, Chain, Log, H256, U256},
};

use log::debug;
use std::sync::Arc;

use crate::data::chain_data::CHAIN_DATA;

use super::uniswap_v2::get_signature_hash;

pub const POOLCREATED_SIGNATURE: &str = "PoolCreated(address,address,uint24,int24,address)";

#[derive(Debug, Clone)]
pub struct PoolCreatedEvent {
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub pool: Address,
}

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

    // Filter for token0 = our token
    let filter_token0 = Filter::new()
        .address(factory_address)
        .topic0(event_signature)
        .topic1(ethers::types::ValueOrArray::Value(H256::from(
            token_address,
        )))
        .from_block(from_block.unwrap_or(BlockNumber::Earliest))
        .to_block(to_block.unwrap_or(BlockNumber::Latest));

    // Filter for token1 = our token
    let filter_token1 = Filter::new()
        .address(factory_address)
        .topic0(event_signature)
        .topic2(ethers::types::ValueOrArray::Value(H256::from(
            token_address,
        )))
        .from_block(from_block.unwrap_or(BlockNumber::Earliest))
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

pub fn decode_poolcreated_event(log: &Log) -> anyhow::Result<PoolCreatedEvent> {
    let token0: Address = log.topics[1].into();
    let token1: Address = log.topics[2].into();
    // fee is indexed and stored in topics[3]
    let fee_u256 = U256::from_big_endian(log.topics[3].as_bytes());
    // Extract lower 24 bits for fee
    let fee = (fee_u256.low_u32() & 0xFFFFFF) as u32;

    // Assuming the data contains the rest in order: user, amount, interestRateMode, borrowRate
    // Proceed with decoding data which is just raw binary (not RLP encoded)
    let raw_log: RawLog = RawLog::from(log.clone());
    let data_slice = raw_log.data;
    if data_slice.len() < 64 {
        return Err(anyhow!("Data field too short to decode all fields"));
    }

    // tickSpacing (int24) is in the first 32 bytes of data
    let tick_spacing_bytes = &data_slice[0..32];
    let tick_spacing_u256 = U256::from_big_endian(tick_spacing_bytes);
    let tick_spacing_raw = tick_spacing_u256.low_u32() & 0xFFFFFF;

    // Sign-extend for int24
    let tick_spacing = if (tick_spacing_raw & 0x800000) != 0 {
        (tick_spacing_raw as i32) - 0x1000000
    } else {
        tick_spacing_raw as i32
    };
    let pool = Address::from_slice(&data_slice[44..64]);

    let poolcreated_event = PoolCreatedEvent {
        token0,
        token1,
        fee,
        tick_spacing,
        pool,
    };

    Ok(poolcreated_event)
}
