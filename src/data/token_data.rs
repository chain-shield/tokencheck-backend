//! This module provides functionality to retrieve ERC20 token information
//! along with its associated Uniswap V2 pair data.

use super::provider_manager::get_chain_provider;
use crate::abi::erc20::ERC20;
use crate::app_config::CHAINS;
use crate::dex::dex_data::{find_top_dex_for_token, TokenDexData};
use anyhow::{anyhow, Result};
use ethers::contract::ContractError;
use ethers::providers::{Provider, ProviderError, Ws};
use ethers::types::{Address, Chain};
use log::{debug, error, info};

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

    pub token_dex: Option<TokenDexData>,
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
    info!("checking token address is vaild....");
    let token_address_h160: Address = match token_address.parse() {
        Ok(address) => address,
        Err(_) => {
            error!("token supplied is not valid address");
            return Ok(None);
        }
    };

    info!("finding which chain token is from...");
    let token_chain = match find_chain_token_is_from(token_address_h160).await? {
        Some(chain) => chain,
        None => {
            error!("token could not be found on any chain");
            return Ok(None);
        }
    };
    info!("token chain is {}", token_chain);

    let provider = get_chain_provider(&token_chain).await?;
    let token_contract = ERC20::new(token_address_h160, provider.clone());

    info!("find which dexes token is listed on that has highest liquidity...");
    let token_dex = find_top_dex_for_token(token_address_h160, &token_chain).await?;

    debug!("token_dex => {:#?}", token_dex);

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
    };

    Ok(Some(token))
}

pub async fn find_chain_token_is_from(token_address: Address) -> anyhow::Result<Option<Chain>> {
    // loop through L1,L2 clients to find which chain token is from
    for chain in CHAINS {
        info!("connecting to chain {}", chain);
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
