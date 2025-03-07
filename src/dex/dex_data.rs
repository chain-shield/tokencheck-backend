use std::cmp::Ordering;

use anyhow::anyhow;
use ethers::{abi::Address, types::Chain};

use crate::{app_config::DEXES, data::dex::Dex};

use super::thegraph::{
    uniswap_v2::get_top_uniswap_v2_pool_by_token_and_chain,
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
                get_top_uniswap_v2_pool_by_token_and_chain(token_address, chain).await?
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

// pub async fn get_token_dex_data(
//     token_address: Address,
//     chain: &Chain,
//     dex: &Dex,
//     event_signature: &str,
//     client: &Arc<Provider<Ws>>,
//     from_block: Option<BlockNumber>,
//     to_block: Option<BlockNumber>,
// ) -> anyhow::Result<Vec<TokenDexData>> {
//     // Get factory address and Create a signature hash for the event
//     let factory_address = get_factory_address(dex, chain)?;
//     let event_signature = get_signature_hash(event_signature);
//
//     // get deployment block for uniswap
//     debug!("getting deployment block for dex..");
//     let deployment_block = get_deployment_block_for_dex_(dex, chain);
//
//     // The token address could be in either topic1 (token0) or topic2 (token1)
//     // So we create two filters and combine the results
//
//     // Filter for token0 = our token
//     let filter_token0 = Filter::new()
//         .address(factory_address)
//         .topic0(event_signature)
//         .topic1(ethers::types::ValueOrArray::Value(H256::from(
//             token_address,
//         )))
//         .from_block(from_block.unwrap_or(deployment_block.into()))
//         .to_block(to_block.unwrap_or(BlockNumber::Latest));
//
//     // Filter for token1 = our token
//     let filter_token1 = Filter::new()
//         .address(factory_address)
//         .topic0(event_signature)
//         .topic2(ethers::types::ValueOrArray::Value(H256::from(
//             token_address,
//         )))
//         .from_block(from_block.unwrap_or(deployment_block.into()))
//         .to_block(to_block.unwrap_or(BlockNumber::Latest));
//
//     // Get logs for both filters
//     debug!("getting pair/pool event created logs from blockchain...");
//     let logs_token0 = client.get_logs(&filter_token0).await?;
//     let logs_token1 = client.get_logs(&filter_token1).await?;
//     let all_logs = [logs_token1, logs_token0].concat();
//
//     // Combine and parse the logs
//     let mut pools = Vec::<TokenDexData>::new();
//
//     // Parse logs where token is token0 or token1
//     debug!("extracting dex data from pair/pool event...");
//     for log in all_logs {
//         let token_pair_event =
//             extract_token_dex_data_from_event_log(token_address, &log, dex, chain, client).await?;
//         pools.push(token_pair_event);
//     }
//
//     // SORT pools by liquidity_in_usd DESC
//     let pools = sort_token_dex_data_vec(pools);
//
//     Ok(pools)
// }
//
// pub async fn extract_token_dex_data_from_event_log(
//     token_address: Address,
//     log: &Log,
//     dex: &Dex,
//     chain: &Chain,
//     client: &Arc<Provider<Ws>>,
// ) -> anyhow::Result<TokenDexData> {
//     match dex {
//         Dex::UniswapV3 => {
//             debug!("decoding uniswap v3 pool event...");
//             let pool_event = decode_uniswap_v3_pool_created_event(&log)?;
//
//             let is_token_0 = pool_event.token0 == token_address;
//
//             let base_token_address = if is_token_0 {
//                 pool_event.token1
//             } else {
//                 pool_event.token0
//             };
//
//             let base_token_address = address_to_string(base_token_address.clone());
//
//             let dex_data = TokenDexData {
//                 dex: dex.clone(),
//                 pair_address: pool_event.pool,
//                 token_0: pool_event.token0,
//                 token_1: pool_event.token1,
//                 fee: pool_event.fee,
//                 base_token_address: base_token_address.clone(),
//                 is_token_0,
//                 ..Default::default()
//             };
//             debug!("dex_data => {:#?}", dex_data);
//
//             debug!("getting uniswap v3 liquidity..");
//             let liquidity = get_liquidity_uniswap_v3(&dex_data, client).await?;
//             debug!("liquidity is {}", liquidity);
//
//             //convert liquidity to USD and add to dex_data
//             debug!("getting uniswap v3 liquidity..");
//             let base_token =
//                 get_and_save_base_token_by_address(&base_token_address, chain, client).await?;
//
//             debug!("base token => {:#?}", base_token);
//
//             let liquidity_in_usd = get_base_token_value_in_usd(&base_token, &liquidity).await?;
//             debug!("liquidity in usd is {}", liquidity_in_usd);
//
//             //update with liquidity and return
//             Ok(TokenDexData {
//                 liquidity_in_usd,
//                 ..dex_data
//             })
//         }
//         Dex::UniswapV2 => {
//             let pair_event = decode_uniswap_v2_pair_created_event(&log)?;
//
//             let dex_data = TokenDexData {
//                 dex: dex.clone(),
//                 pair_address: pair_event.pair,
//                 token_0: pair_event.token0,
//                 token_1: pair_event.token1,
//                 fee: UNISWAP_V2_FEE,
//                 is_token_0: pair_event.token0 == token_address,
//                 ..Default::default()
//             };
//
//             let liquidity = get_liquidity_uniswap_v2(&dex_data, client).await?;
//
//             //update with liquidity and return
//             Ok(TokenDexData { ..dex_data })
//         }
//         _ => return Err(anyhow!("unsupported dex")),
//     }
// }
//
//
// pub fn get_factory_address(dex: &Dex, chain: &Chain) -> anyhow::Result<Address> {
//     let factory_address = match dex {
//         Dex::UniswapV3 => CHAIN_DATA
//             .get_address(chain)
//             .uniswap_v3_factory
//             .clone()
//             .parse::<Address>()?,
//         Dex::UniswapV2 => CHAIN_DATA
//             .get_address(chain)
//             .uniswap_v3_factory
//             .clone()
//             .parse::<Address>()?,
//         _ => return Err(anyhow!("unsupported dex")),
//     };
//
//     Ok(factory_address)
// }
//
// pub fn get_signature_hash(interface: &str) -> H256 {
//     H256::from(keccak256(interface.as_bytes()))
// }
//
// pub fn get_deployment_block_for_dex_(dex: &Dex, chain: &Chain) -> u64 {
//     match chain {
//         Chain::Mainnet => match dex {
//             Dex::UniswapV2 => 10_000_000,
//             Dex::UniswapV3 => 12_000_000,
//             _ => 10_000_000,
//         },
//         Chain::Base => match dex {
//             Dex::UniswapV2 => 100_000,
//             Dex::UniswapV3 => 100_000,
//             _ => 100_000,
//         },
//         _ => 100_000,
//     }
// }
