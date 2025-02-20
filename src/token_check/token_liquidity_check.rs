//! This module provides functionality to check the percentage of an ERC20 token's liquidity
//! that is either locked or burned. It does this by fetching the total supply from the token,
//! retrieving token holders from external APIs (either Moralis or Uniswap), and aggregating the
//! locked balances based on a set of predefined "locker" addresses.
//!
//! The main function exposed by this module is `get_percentage_liquidity_locked_or_burned`.

use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;

use crate::{
    app_config::{CHAIN, TOKEN_LOCKERS_BASE, TOKEN_LOCKERS_MAINNET},
    token_check::{
        check_token_lock::TokenHolders,
        external_api::thegraph_api::fetch_uniswap_lp_holders,
        {external_api::moralis, token_holder_check::u256_div_u256_to_f64},
    },
    utils::type_conversion::address_to_string,
};

use crate::data::token_data::ERC20Token;

/// Calculates the percentage of an ERC20 token's liquidity that is either locked or burned.
///
/// This function performs the following steps:
///
/// 1. Retrieves the total supply of the liquidity token by querying the blockchain.
/// 2. Fetches the list of token holders:
///    - For `Chain::Base`, the list is fetched via the Moralis API using the pair address.
///    - Otherwise, the list is fetched from Uniswap's Graph API.
/// 3. Iterates through each token holder to:
///    - Identify the top token holder based on balance (for informational purposes).
///    - Sum the locked balances from known locker addresses (predefined in
///      `TOKEN_LOCKERS_MAINNET` or `TOKEN_LOCKERS_BASE`).
/// 4. Computes the percentage of liquidity locked relative to total supply.
///
/// # Parameters
///
/// - `token`: A reference to an `ERC20Token` holding the token's properties and metadata.
/// - `client`: An `Arc`-wrapped `Provider<Ws>` used for blockchain interaction.
///
/// # Returns
///
/// - `Ok(Some(percentage))` if the percentage of locked liquidity can be computed.
/// - `Ok(None)` if no token holders are found or if the top holder's balance is zero.
/// - An error of type `anyhow::Error` if any underlying call fails (e.g., network or API errors).
///
/// # Example
///
/// ```no_run
/// # async fn example_usage() -> anyhow::Result<()> {
/// use ethers::prelude::*;
/// use std::sync::Arc;
///
/// // Assume you have initialized an ERC20Token and a WebSocket provider.
/// let token: ERC20Token = /* initialize token */ ;
/// let ws_provider: Provider<Ws> = /* initialize provider */ ;
/// let client = Arc::new(ws_provider);
///
/// let locked_percentage = get_percentage_liquidity_locked_or_burned(&token, &client).await?;
/// println!("Locked liquidity percentage: {:?}", locked_percentage);
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// - The function adjusts its behavior depending on the network configuration:
///   for `Chain::Base`, it uses Moralis API; for other chains, it uses Uniswap's data.
/// - Locked addresses are determined using the constants `TOKEN_LOCKERS_MAINNET` or `TOKEN_LOCKERS_BASE`,
///   which include addresses associated with token locking or burning (for example, burn addresses).
///
pub async fn get_percentage_liquidity_locked_or_burned(
    token: &ERC20Token,
    client: &Arc<Provider<Ws>>,
) -> Result<Option<f64>> {
    // Retrieve total supply for the liquidity token.
    let total_supply = token.get_total_liquidity_token_supply(client).await?;

    // Retrieve the list of token holders based on the chain configuration.
    let top_holders: Vec<TokenHolders> = if CHAIN == Chain::Base {
        // For Chain::Base, convert the pair address and use the Moralis API.
        let pair_address = address_to_string(token.pair_address);
        moralis::get_token_holder_list(&pair_address).await?
    } else {
        // For other chains, fetch token holders using Uniswap's Graph API.
        fetch_uniswap_lp_holders(token.pair_address).await?
    };

    // Return None if no token holders are found.
    if top_holders.is_empty() {
        return Ok(None);
    }

    // Initialize locked balance counter and prepare to find the top holder.
    let mut locked_balance = U256::zero();
    let mut top_holder = TokenHolders::default();

    // Process each token holder:
    // - Update the top holder if a higher balance is found.
    // - Sum the locked balances based on predefined locker addresses.
    for info in top_holders.iter() {
        // If the current holder's quantity is greater than the stored top holder's, update it.
        if top_holder.quantity < info.quantity {
            top_holder = TokenHolders {
                holder: info.holder.clone(),
                quantity: info.quantity,
            };
        }

        // Depending on the chain, add to the locked balance if the holder is in the known list.
        if CHAIN == Chain::Mainnet {
            if TOKEN_LOCKERS_MAINNET.contains(&info.holder.as_str()) {
                locked_balance += info.quantity;
            }
        } else {
            if TOKEN_LOCKERS_BASE.contains(&info.holder.as_str()) {
                locked_balance += info.quantity;
            }
        }
    }

    // If the top holder's balance is zero, meaningful computation cannot continue.
    if top_holder.quantity == U256::zero() {
        return Ok(None);
    }

    // Log the top holder's details for debugging or audit purposes.
    println!(
        "top holder for {} LP is {} with {}",
        token.name, top_holder.holder, top_holder.quantity
    );

    // Calculate the percentage of liquidity locked relative to total supply.
    let percentage_of_liquidity_locked =
        100_f64 * u256_div_u256_to_f64(locked_balance, total_supply)?;

    Ok(Some(percentage_of_liquidity_locked))
}
