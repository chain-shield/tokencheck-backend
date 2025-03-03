use anyhow::anyhow;
use ethers::{
    abi::{Address, RawLog},
    core::types::Filter,
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, Chain, Log, H256, U256},
};
use std::sync::Arc;

use crate::data::{chain_data::CHAIN_DATA, dex::Dex};

use super::shared::{get_deployment_block_for_dex_, get_signature_hash};

pub const PAIR_CREATED_SIGNATURE: &str = "PairCreated(address,address,address,uint256)";

#[derive(Debug, Clone)]
pub struct PairCreatedEvent {
    pub token0: Address,
    pub token1: Address,
    pub pair: Address,
    pub noname: U256,
}

pub async fn get_uniswap_v2_pairs_for_token(
    client: &Arc<Provider<Ws>>,
    token_address: Address,
    chain: &Chain,
    from_block: Option<BlockNumber>,
    to_block: Option<BlockNumber>,
) -> anyhow::Result<Vec<PairCreatedEvent>> {
    let factory_address = CHAIN_DATA
        .get_address(chain)
        .uniswap_v2_factory
        .clone()
        .parse::<Address>()?;

    // Create a signature hash for the event
    let event_signature = get_signature_hash(PAIR_CREATED_SIGNATURE);

    // The token address could be in either topic1 (token0) or topic2 (token1)
    // So we create two filters and combine the results

    // get deployment block for uniswap
    let deployment_block = get_deployment_block_for_dex_(&Dex::UniswapV2, &chain);

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
        let pair_created_event = decode_pair_created_event(&log)?;

        pools.push(pair_created_event);
    }

    Ok(pools)
}
pub fn decode_pair_created_event(log: &Log) -> anyhow::Result<PairCreatedEvent> {
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
