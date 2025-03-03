use anyhow::anyhow;
use ethers::{
    abi::{Address, RawLog},
    core::types::Filter,
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, Chain, Log, H256, U256},
};

use log::debug;
use std::sync::Arc;

use crate::data::{chain_data::CHAIN_DATA, dex::Dex};

use super::shared::{
    get_deployment_block_for_dex_, get_signature_hash, get_token_dex_data, TokenDexData,
    PAIR_CREATED_SIGNATURE, POOLCREATED_SIGNATURE,
};

#[derive(Debug, Clone)]
pub struct PoolCreatedEvent {
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub pool: Address,
}

#[derive(Debug, Clone)]
pub struct PairCreatedEvent {
    pub token0: Address,
    pub token1: Address,
    pub pair: Address,
    pub noname: U256,
}

pub async fn get_uniswap_v3_token_dex_data(
    token_address: Address,
    chain: &Chain,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<Vec<TokenDexData>> {
    get_token_dex_data(
        token_address,
        chain,
        &Dex::UniswapV3,
        POOLCREATED_SIGNATURE,
        client,
        None,
        None,
    )
    .await
}

pub async fn get_uniswap_v2_token_dex_data(
    token_address: Address,
    chain: &Chain,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<Vec<TokenDexData>> {
    get_token_dex_data(
        token_address,
        chain,
        &Dex::UniswapV2,
        PAIR_CREATED_SIGNATURE,
        client,
        None,
        None,
    )
    .await
}

pub fn decode_uniswap_v3_pool_created_event(log: &Log) -> anyhow::Result<PoolCreatedEvent> {
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

pub fn decode_uniswap_v2_pair_created_event(log: &Log) -> anyhow::Result<PairCreatedEvent> {
    let token0: Address = log.topics[1].into();
    let token1: Address = log.topics[2].into();

    // Assuming the data contains the rest in order: user, amount, interestRateMode, borrowRate
    // Proceed with decoding data which is just raw binary (not RLP encoded)
    let raw_log: RawLog = RawLog::from(log.clone());
    let data_slice = raw_log.data;
    if data_slice.len() < 64 {
        return Err(anyhow!("Data field too short to decode all fields"));
    }

    let pair = Address::from_slice(&data_slice[12..32]);
    let noname = U256::from_big_endian(&data_slice[32..64]);

    let pair_created_event = PairCreatedEvent {
        token0,
        token1,
        pair,
        noname,
    };

    Ok(pair_created_event)
}
