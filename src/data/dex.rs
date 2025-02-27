use crate::abi::erc20::ERC20;
use crate::abi::uniswap_factory_v2::UNISWAP_V2_FACTORY;
use crate::abi::uniswap_pair::UNISWAP_PAIR;
use crate::abi::uniswap_pool::UNISWAP_V3_POOL;
use crate::abi::uniswap_v3_factory::UNISWAP_V3_FACTORY;
use crate::app_config::DEXES;
use ethers::providers::{Provider, Ws};
use ethers::types::{Address, Chain};
use std::sync::Arc;

use super::chain_data::CHAIN_DATA;

// list of dexes
#[derive(Clone, Default, Debug)]
pub enum Dex {
    #[default]
    UniswapV2,
    UniswapV3,
    UniswapV4,
    Aerodrome,
    Sushiswap,
    Balancer,
    Curve,
    DackieSwap,
    BasedSwap,
    AlienBase,
    OasisSwap,
    LFGSwap,
    IcecreamSwap,
    Glacier,
    CrescentSwap,
    Throne,
    EtherVista,
    KokonutSwap,
    BakerySwap,
    CbsSwap,
    MoonBase,
    DegenBrains,
    Fwx,
    CandySwap,
    Memebox,
    BasoFinance,
    DerpDex,
    Satori,
    HorizonDex,
    BaseX,
    LeetSwap,
    RobotsFram,
    CitadelSwap,
    Velocimeter,
    DiamondSwap,
    SharkSwap,
    Infusion,
    NineMm,
    RocketSwap,
    Solidly,
    GammaSwap,
    Synthswap,
    IziSwap,
    Equalizer,
    SwapBased,
    Unknown,
}

/// the top dex the token is listed on
#[derive(Clone, Default, Debug)]
pub struct TokenDex {
    pub dex: Dex,
    pub pair_or_pool_address: Address,
    /// A flag indicating whether the token is the first token (token_0)
    /// in the Uniswap pair; if false the token is token_1.
    pub is_token_0: bool,
}

pub async fn find_top_dex_pair_address_and_is_token_0(
    token_address: Address,
    client: &Arc<Provider<Ws>>,
    chain: &Chain,
) -> anyhow::Result<Option<(Dex, Address, bool)>> {
    let mut top_dex: Option<Dex> = None;
    let mut top_pair_address = Address::zero();
    let mut is_token_0 = false;
    let mut top_dex_liquidity = 0;

    // loop through L1,L2 clients to find which chain token is from
    for dex in DEXES {
        let dex_pair_address = match dex {
            Dex::UniswapV2 => is_token_listed_on_uniswap_v2(token_address, client, chain).await?,
            Dex::UniswapV3 => is_token_listed_on_uniswap_v3(token_address, client, chain).await?,
            _ => None,
        };

        if let Some(pair_address) = dex_pair_address {
            // check liquidity of token on dex
            let (liquidity, token_0) =
                get_token_liquidity_and_is_token_0_on_(&dex, pair_address, client, chain).await?;
            if liquidity > top_dex_liquidity {
                top_dex_liquidity = liquidity;
                top_dex = Some(dex.clone());
                is_token_0 = token_0;
                top_pair_address = pair_address;
            }
        }
    }

    if let Some(top_dex_unwrapped) = top_dex {
        Ok(Some((top_dex_unwrapped, top_pair_address, is_token_0)))
    } else {
        Ok(None)
    }
}

pub async fn is_token_listed_on_uniswap_v2(
    token_address: Address,
    client: &Arc<Provider<Ws>>,
    chain: &Chain,
) -> anyhow::Result<Option<Address>> {
    // Retrieve configuration addresses from contracts.
    let uniswap_v2_factory_address: Address =
        CHAIN_DATA.get_address(chain).uniswap_v2_factory.parse()?;
    let weth_address: Address = CHAIN_DATA.get_address(chain).weth.parse()?;

    // Initialize the Uniswap V2 factory contract to query for pair data.
    let uniswap_factory = UNISWAP_V2_FACTORY::new(uniswap_v2_factory_address, client.clone());

    // Call getPair and check the result
    let pair_address = uniswap_factory
        .get_pair(token_address, weth_address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query Uniswap V2 pair: {}", e))?;

    // Return true if a pair exists (non-zero address), false otherwise
    if pair_address != Address::zero() {
        Ok(Some(pair_address))
    } else {
        Ok(None)
    }
}

pub async fn is_token_listed_on_uniswap_v3(
    token_address: Address,
    client: &Arc<Provider<Ws>>,
    chain: &Chain,
) -> anyhow::Result<Option<Address>> {
    // Retrieve configuration addresses from contracts
    let uniswap_v3_factory_address: Address =
        CHAIN_DATA.get_address(chain).uniswap_v3_factory.parse()?;
    let weth_address: Address = CHAIN_DATA.get_address(chain).weth.parse()?;

    // Initialize the Uniswap V3 factory contract
    let uniswap_factory = UNISWAP_V3_FACTORY::new(uniswap_v3_factory_address, client.clone());

    // Common Uniswap V3 fee tiers
    let fee_tiers = vec![500, 3000, 10000];

    // Check each fee tier for an existing pool
    for fee in fee_tiers {
        let pool_address = uniswap_factory
            .get_pool(token_address, weth_address, fee)
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to query Uniswap V3 pool for fee {}: {}", fee, e)
            })?;

        if pool_address != Address::zero() {
            return Ok(Some(pool_address)); // Pool found at this fee tier
        }
    }

    // No pools found across all fee tiers
    Ok(None)
}

pub async fn get_token_liquidity_and_is_token_0_on_(
    dex: &Dex,
    pair_address: Address,
    client: &Arc<Provider<Ws>>,
    chain: &Chain,
) -> anyhow::Result<(u128, bool)> {
    match dex {
        Dex::UniswapV2 => {
            let (liquidity, is_token_0) =
                get_liquidity_and_is_token_0_uniswap_v2(pair_address, client, chain).await?;
            return Ok((liquidity, is_token_0));
        }
        Dex::UniswapV3 => {
            let (liquidity, is_token_0) =
                get_liquidity_and_is_token_0_uniswap_v3(pair_address, client, chain).await?;
            return Ok((liquidity, is_token_0));
        }
        _ => {}
    }

    Ok((0, false))
}

pub async fn get_liquidity_and_is_token_0_uniswap_v2(
    pair_address: Address,
    client: &Arc<Provider<Ws>>,
    chain: &Chain,
) -> anyhow::Result<(u128, bool)> {
    let pool = UNISWAP_PAIR::new(pair_address, client.clone());

    // Retrieve the reserves data.
    let (reserve0, reserve1, _) = pool.get_reserves().call().await?;

    let token0 = pool
        .token_0()
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get token0: {}", e))?;
    let token1 = pool
        .token_1()
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get token1: {}", e))?;

    // Retrieve the WETH address from CHAIN_DATA
    let weth_address: Address = CHAIN_DATA.get_address(chain).weth.parse()?;

    // Determine which reserve corresponds to WETH
    let eth_reserve = if token0 == weth_address {
        reserve0
    } else if token1 == weth_address {
        reserve1
    } else {
        return Err(anyhow::anyhow!(
            "WETH not found in pair: token0={:?}, token1={:?}, weth={:?}",
            token0,
            token1,
            weth_address
        ));
    };

    // Convert U128 to u128 and return
    Ok((eth_reserve, token1 == weth_address))
}

pub async fn get_liquidity_and_is_token_0_uniswap_v3(
    pool_address: Address,
    client: &Arc<Provider<Ws>>,
    chain: &Chain,
) -> anyhow::Result<(u128, bool)> {
    // Initialize the Uniswap V3 Pool contract
    let pool = UNISWAP_V3_POOL::new(pool_address, client.clone());

    // Get the token addresses from the pool contract
    let token0 = pool
        .token_0()
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get token0: {}", e))?;
    let token1 = pool
        .token_1()
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get token1: {}", e))?;

    // Retrieve the WETH address (assuming CHAIN_DATA provides it)
    let weth_address: Address = CHAIN_DATA.get_address(chain).weth.parse()?;

    // Determine which token is WETH
    let is_weth_token0 = token0 == weth_address;
    let is_weth_token1 = token1 == weth_address;
    if !is_weth_token0 && !is_weth_token1 {
        return Err(anyhow::anyhow!(
            "WETH not found in pool: token0={:?}, token1={:?}, weth={:?}",
            token0,
            token1,
            weth_address
        ));
    }

    // Initialize the WETH contract to query its balance in the pool
    let weth_contract = ERC20::new(weth_address, client.clone());

    // Get the WETH balance of the pool (reserve)
    let weth_balance = weth_contract
        .balance_of(pool_address)
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get WETH balance: {}", e))?;

    // Convert U256 to u128 (assuming it fits)
    let eth_reserve = weth_balance.as_u128();

    Ok((eth_reserve, is_weth_token1))
}
