//! This module provides functionality to retrieve ERC20 token information
//! along with its associated DEX pair data with the highest liquidity.

use super::provider_manager::get_chain_provider;
use crate::abi::erc20::ERC20;
use crate::app_config::CHAINS;
use crate::dex::dex_data::{find_top_dex_for_token, TokenDexData};
use anyhow::{anyhow, Result};
use ethers::contract::ContractError;
use ethers::providers::{Provider, ProviderError, Ws};
use ethers::types::{Address, Chain};
use log::{debug, error, info};

/// Represents an ERC20 token along with its associated DEX pair data.
#[derive(Clone, Default, Debug)]
pub struct ERC20Token {
    /// The blockchain network ID where this token exists
    pub chain: Chain,
    /// The token's full name.
    pub name: String,
    /// The token's symbol.
    pub symbol: String,
    /// Number of decimals the token uses.
    pub decimals: u8,
    /// The token's contract address.
    pub address: Address,
    /// Optional DEX data for the token, containing information about the most liquid pair
    pub token_dex: Option<TokenDexData>,
}

/// Fetches and constructs an `ERC20Token` from a given token address.
///
/// This function attempts to:
/// 1. Validate the token address
/// 2. Determine which blockchain the token exists on
/// 3. Fetch basic token information (name, symbol, decimals)
/// 4. Find the DEX with the highest liquidity for this token
///
/// # Arguments
///
/// * `token_address` - A string slice representing the token's address.
///
/// # Returns
///
/// * `Result<Option<ERC20Token>>` - On success, returns `Some(ERC20Token)` with the token info.
///   Returns `None` if the token address is invalid or the token cannot be found on any chain.
///   Returns an error if there's a network issue or other problem during data retrieval.
///
/// # Example
///
/// ```ignore
/// let token = get_core_token_data_by_address("0x...").await?;
/// if let Some(token_info) = token {
///     println!("Token symbol: {}", token_info.symbol);
/// }
/// ```
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

/// Determines which blockchain network a token exists on by checking multiple chains.
///
/// This function iterates through the configured chains and attempts to call
/// standard ERC20 methods on each chain to verify the token's existence.
///
/// # Arguments
///
/// * `token_address` - The Ethereum address of the token to locate
///
/// # Returns
///
/// * `Result<Option<Chain>>` - On success, returns `Some(Chain)` with the identified chain.
///   Returns `None` if the token cannot be found on any chain.
///   Returns an error if there's a network issue during the search.
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
            // If name() fails (e.g., contract doesn't exist), skip to next chain
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

        // If all calls succeed, we've found the chain
        if name_result.is_ok() && symbol_result.is_ok() && supply_result.is_ok() {
            return Ok(Some(chain));
        }
    }

    Ok(None)
}

/// Determines if a contract error is related to network connectivity issues.
///
/// This helper function distinguishes between errors that indicate a contract
/// doesn't exist (which should be handled by trying another chain) versus
/// actual network connectivity problems that should halt execution.
///
/// # Arguments
///
/// * `error` - The contract error to check
///
/// # Returns
///
/// * `bool` - `true` if the error is a network-related error, `false` otherwise
pub fn is_network_error(error: &ContractError<Provider<Ws>>) -> bool {
    match error {
        ContractError::ProviderError { e } => {
            matches!(e, ProviderError::JsonRpcClientError(_))
        }
        _ => false,
    }
}
