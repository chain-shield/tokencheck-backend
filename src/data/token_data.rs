//! This module provides functionality to retrieve ERC20 token information
//! along with its associated Uniswap V2 pair data.

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_factory_v2::UNISWAP_V2_FACTORY;
use crate::abi::uniswap_pair::UNISWAP_PAIR;
use crate::app_config::CHAINS;
use crate::data::dex::find_top_dex_pair_address_and_is_token_0;
use anyhow::{anyhow, Result};
use ethers::contract::ContractError;
use ethers::providers::{Provider, ProviderError, Ws};
use ethers::types::{Address, Chain};
use log::{error, info};
use std::sync::Arc;

use super::chain_data::CHAIN_DATA;
use super::dex::TokenDex;
use super::provider_manager::get_chain_provider;
/// Represents an ERC20 token along with its associated Uniswap pair data.
#[derive(Clone, Default, Debug)]
pub struct ERC20Token {
    /// chain id
    pub chain: Chain,
    /// The token's full name.
    pub name: String,
    /// The token's symbol.
    pub symbol: String,
    /// Number of decimals the token uses.
    pub decimals: u8,
    /// The token's contract address.
    pub address: Address,

    pub token_dex: TokenDex,

    pub is_listed_on_dex: bool,
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
pub async fn get_core_token_data_by_address(token_address: &str) -> Result<Option<ERC20Token>> {
    info!("Setting up token contract...");

    // Parse the token address from a string to an Address.
    let token_address_h160: Address = match token_address.parse() {
        Ok(address) => address,
        Err(_) => {
            error!("token supplied is not valid address");
            return Ok(None);
        }
    };

    let token_chain = match find_chain_token_is_from(token_address_h160).await? {
        Some(chain) => chain,
        None => {
            error!("token could not be found on any chain");
            return Ok(None);
        }
    };
    let provider = get_chain_provider(&token_chain).await?;
    let token_contract = ERC20::new(token_address_h160, provider.clone());

    let token_dex =
        match find_top_dex_pair_address_and_is_token_0(token_address_h160, &provider, &token_chain)
            .await?
        {
            Some((dex, pair_address, is_token_0)) => TokenDex {
                dex,
                pair_or_pool_address: pair_address,
                is_token_0,
            },
            None => TokenDex::default(), // pair address will be Address(0)
        };

    let is_listed_on_dex = token_dex.pair_or_pool_address == Address::zero();

    // Fetch the basic token data (name, symbol, decimals) from the ERC20 contract.
    info!("Getting basic token info...");
    let symbol = token_contract.symbol().call().await?;
    let decimals = token_contract.decimals().call().await?;
    let name = token_contract.name().call().await?;

    let token = ERC20Token {
        chain: token_chain,
        name,
        symbol,
        decimals,
        address: token_address_h160,
        token_dex,
        is_listed_on_dex,
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
    chain: &Chain,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<(Address, bool)> {
    // Retrieve configuration addresses from contracts.
    let uniswap_v2_factory_address: Address =
        CHAIN_DATA.get_address(chain).uniswap_v2_factory.parse()?;
    let weth_address: Address = CHAIN_DATA.get_address(chain).weth.parse()?;

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

pub async fn find_chain_token_is_from(token_address: Address) -> anyhow::Result<Option<Chain>> {
    // loop through L1,L2 clients to find which chain token is from
    for chain in CHAINS {
        let chain_provider = get_chain_provider(&chain).await?;
        let token_contract = ERC20::new(token_address, chain_provider);
        // Check multiple ERC20 methods to confirm the token's presence
        let name_result = token_contract.name().call().await;
        if let Err(e) = &name_result {
            if is_network_error(e) {
                return Err(anyhow!("Network error on chain {:?}: {:?}", chain, e));
            }
            // If name() fails (e.g., contract doesn’t exist), skip to next chain
            continue;
        }

        let symbol_result = token_contract.symbol().call().await;
        if let Err(e) = &symbol_result {
            if is_network_error(e) {
                return Err(anyhow!("Network error on chain {:?}: {:?}", chain, e));
            }
            continue;
        }

        let supply_result = token_contract.total_supply().await;
        if let Err(e) = &supply_result {
            if is_network_error(e) {
                return Err(anyhow!("Network error on chain {:?}: {:?}", chain, e));
            }
            continue;
        }

        // If all calls succeed, we’ve found the chain
        if name_result.is_ok() && symbol_result.is_ok() && supply_result.is_ok() {
            if let Ok(name) = name_result {
                println!("Token found with name: {}", name);
            }
            return Ok(Some(chain));
        }
    }

    Ok(None)
}

pub fn is_network_error(error: &ContractError<Provider<Ws>>) -> bool {
    match error {
        ContractError::ProviderError { e } => {
            matches!(e, ProviderError::JsonRpcClientError(_))
        }
        _ => false,
    }
}
