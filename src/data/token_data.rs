//! This module provides functionality to retrieve ERC20 token information
//! along with its associated Uniswap V2 pair data.

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_factory_v2::UNISWAP_V2_FACTORY;
use crate::abi::uniswap_pair::UNISWAP_PAIR;
use anyhow::Result;
use ethers::providers::{Provider, Ws};
use ethers::types::Address;
use log::info;
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

/// Represents an ERC20 token along with its associated Uniswap pair data.
#[derive(Clone, Default, Debug)]
pub struct ERC20Token {
    /// The token's full name.
    pub name: String,
    /// The token's symbol.
    pub symbol: String,
    /// Number of decimals the token uses.
    pub decimals: u8,
    /// The token's contract address.
    pub address: Address,

    pub token_dex: TokenDex,
}

/// Fetches and constructs an `ERC20Token` from a given token address.
///
/// # Arguments
///
/// * `token_address` - A string slice representing the token's address.
/// * `client` - An `Arc` wrapped provider of type `Provider<Ws>` used to interact
///   with the Ethereum node.
///
/// # Returns
///
/// * `Result<Option<ERC20Token>>` - On success, returns an `Option` with the token info.
///   If the token does not exist or an error occurs during the RPC calls, an appropriate error is returned.
///
/// # Example
///
/// ```ignore
/// let token = get_erc20_by_token_address("0x...", client).await?;
/// if let Some(token_info) = token {
///     println!("Token symbol: {}", token_info.symbol);
/// }
/// ```
///
pub async fn get_erc20_by_token_address(
    token_address: &str,
    client: &Arc<Provider<Ws>>,
) -> Result<Option<ERC20Token>> {
    info!("Setting up token contract...");

    // Parse the token address from a string to an Address.
    let token_address_h160: Address = token_address.parse()?;
    let token_contract = ERC20::new(token_address_h160, client.clone());

    // Retrieve the Uniswap V2 pair address and determine the token position (token_0 or token_1).
    let (pair_address, is_token_0) =
        get_token_uniswap_v2_pair_address(token_address_h160, client).await?;

    // Fetch the basic token data (name, symbol, decimals) from the ERC20 contract.
    info!("Getting basic token info...");
    let symbol = token_contract.symbol().call().await?;
    let decimals = token_contract.decimals().call().await?;
    let name = token_contract.name().call().await?;

    let token = ERC20Token {
        name,
        symbol,
        decimals,
        address: token_address_h160,
        token_dex: TokenDex {
            pair_or_pool_address: pair_address,
            is_token_0,
            ..Default::default()
        },
        ..Default::default()
    };

    Ok(Some(token))
}

/// Retrieves the Uniswap V2 pair address for a given token and determines the token's position within the pair.
///
/// # Arguments
///
/// * `token_address` - The address of the ERC20 token.
/// * `client` - An `Arc` wrapped provider of type `Provider<Ws>` used to interact
///   with the Ethereum node.
///
/// # Returns
///
/// * `anyhow::Result<(Address, bool)>` - On success, returns a tuple where:
///    - The first element is the Uniswap V2 pair address.
///    - The second element is a boolean indicating if the provided token is token_0
///      (`true` if it is, `false` otherwise).
///
/// # Details
///
/// This function retrieves necessary addresses from the `CONTRACT` configuration, connects to
/// the Uniswap V2 factory, retrieves the pair address, and then confirms the token's position
/// by comparing with the weth address.
///
/// # Example
///
/// ```ignore
/// let (pair_address, is_token_0) = get_token_uniswap_v2_pair_address(token_address, client).await?;
/// println!("Pair address: {:?}, token is token_0: {}", pair_address, is_token_0);
/// ```
pub async fn get_token_uniswap_v2_pair_address(
    token_address: Address,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<(Address, bool)> {
    // Retrieve configuration addresses from contracts.
    let uniswap_v2_factory_address: Address =
        CHAIN_DATA.get_address().uniswap_v2_factory.parse()?;
    let weth_address: Address = CHAIN_DATA.get_address().weth.parse()?;

    // Initialize the Uniswap V2 factory contract to query for pair data.
    let uniswap_factory = UNISWAP_V2_FACTORY::new(uniswap_v2_factory_address, client.clone());
    let pair_address = uniswap_factory
        .get_pair(token_address, weth_address)
        .await?;
    // Initialize the Uniswap pair contract.
    let pair_contract = UNISWAP_PAIR::new(pair_address, client.clone());

    // Retrieve token_0 from the pair contract.
    let token_0 = pair_contract.token_0().call().await?;

    // Determine if the provided token is token_0 by checking if token_0 is different from weth.
    let is_token_0 = token_0 != weth_address;

    Ok((pair_address, is_token_0))
}
