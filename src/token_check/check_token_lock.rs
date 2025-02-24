//! This module provides functions to check whether the liquidity tokens of an ERC20 token
//! are locked above a given threshold. It interfaces with external APIs (Etherscan or The Graph)
//! depending on the chain and aggregates data from known token locker addresses.

use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;

use crate::{
    app_config::{CHAIN, TOKEN_LOCKERS_BASE, TOKEN_LOCKERS_MAINNET},
    token_check::external_api::{
        etherscan_api::get_token_holder_list, thegraph_api::fetch_uniswap_lp_holders,
    },
};

use crate::data::token_data::ERC20Token;

/// Represents a token holder with its address and the token balance held.
///
/// This struct is used to store information about liquidity token holders fetched from external APIs.
#[derive(Debug, Default)]
pub struct TokenHolders {
    /// The address of the token holder.
    pub holder: String,
    /// The token balance of the holder.
    pub quantity: U256,
}

/// Checks if the liquidity of the given token is locked above a specified threshold.
///
/// This asynchronous function retrieves the total liquidity token supply and the list of top holders
/// (using either Etherscan or The Graph based on the chain). It then determines:
///  1. The top holder by liquidity.
///  2. The total locked balance by summing the balances from addresses that are on the designated
///     list of token locker addresses.
///  3. Whether this locked balance meets or exceeds a given percentage (threshold) of the total supply.
///
/// # Arguments
///
/// * `token` - A reference to the ERC20 token for which the liquidity lock status is being checked.
/// * `threshold_percent` - The percentage threshold (as a float) to determine if the liquidity is sufficiently locked.
/// * `client` - An asynchronous client (wrapped in an Arc) that is used to interact with the Ethereum node.
///
/// # Returns
///
/// * `Result<Option<bool>>` - Returns:
///     - `Ok(Some(true))` if the locked balance is greater than or equal to the threshold,
///     - `Ok(Some(false))` if it is below the threshold,
///     - `Ok(None)` if data is insufficient (e.g., when no valid top holder is found),
///     - or an error if any of the external API calls or data processing fails.
///
/// # Examples
///
/// ```no_run
/// # use ethers::prelude::*;
/// # use std::sync::Arc;
/// # async fn example(token: ERC20Token, client: Arc<Provider<Ws>>) -> anyhow::Result<()> {
/// let is_locked = is_liquidity_locked(&token, 50.0, &client).await?;
/// match is_locked {
///     Some(true) => println!("Liquidity is locked above threshold."),
///     Some(false) => println!("Liquidity lock below threshold."),
///     None => println!("Insufficient data to determine liquidity lock status."),
/// }
/// # Ok(())
/// # }
/// ```
pub async fn is_liquidity_locked(
    token: &ERC20Token,
    threshold_percent: f64,
    client: &Arc<Provider<Ws>>,
) -> Result<Option<bool>> {
    // Retrieve the total liquidity token supply from the token contract.
    let total_supply = token.get_total_liquidity_token_supply(client).await?;

    // Retrieve the list of top liquidity providers from an external API.
    // Uses different implementations based on the chain configuration.
    let top_holders: Vec<TokenHolders> = if CHAIN == Chain::Base {
        // For Base chain, use the Etherscan API.
        get_token_holder_list(token.token_dex.pair_or_pool_address).await?
    } else {
        // For other chains (e.g., Mainnet), use the The Graph API.
        fetch_uniswap_lp_holders(token.token_dex.pair_or_pool_address).await?
    };

    // Initialize the sum of locked token balances to zero.
    let mut locked_balance = U256::zero();

    // Variable to track the top holder (the one with the highest token quantity).
    let mut top_holder = TokenHolders::default();

    // Iterate over each token holder entry.
    for info in top_holders.iter() {
        // Update top_holder if the current holder has a larger quantity.
        if top_holder.quantity < info.quantity {
            top_holder = TokenHolders {
                holder: info.holder.clone(),
                quantity: info.quantity,
            };
        }

        // Depending on the chain, sum up balances from all known locker addresses.
        if CHAIN == Chain::Mainnet {
            if TOKEN_LOCKERS_MAINNET.contains(&info.holder.as_str()) {
                locked_balance = locked_balance + info.quantity;
            }
        } else {
            if TOKEN_LOCKERS_BASE.contains(&info.holder.as_str()) {
                locked_balance = locked_balance + info.quantity;
            }
        }
    }

    // If no valid top holder is found (i.e., top_holder quantity is zero), return None.
    if top_holder.quantity == U256::zero() {
        return Ok(None);
    }

    // Log the top holder's details for debugging or information purposes.
    println!(
        "top holder for {} LP is {} with {}",
        token.name, top_holder.holder, top_holder.quantity
    );

    // Calculate the required locked balance based on the provided threshold percentage.
    let required_locked = total_supply * U256::from(threshold_percent as u64) / U256::from(100_u64);
    // Check if the aggregated locked balance meets or exceeds this calculated threshold.
    let locked_enough = locked_balance >= required_locked;

    Ok(Some(locked_enough))
}
