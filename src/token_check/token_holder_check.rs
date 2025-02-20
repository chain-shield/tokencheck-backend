//! This module implements the token holder check logic.
//!
//! It retrieves information about token holders (using an external API),
//! calculates metrics such as the percentage of tokens held by the top holder,
//! the percentage of tokens burned or locked, and determines whether the top holder
//! holds more than a defined threshold percentage of the total token supply.
//!
//! The functions here use asynchronous calls to fetch data and perform necessary computations.

use anyhow::{anyhow, Result};
use ethers::prelude::*;
use std::sync::Arc;

use crate::{
    app_config::{
        CHAIN, TOKEN_HOLDER_THRESHOLD_PERCENTAGE, TOKEN_LOCKERS_BASE, TOKEN_LOCKERS_MAINNET,
    },
    utils::type_conversion::{address_to_string, u256_to_f64},
};

use crate::data::token_data::ERC20Token;
use crate::token_check::{check_token_lock::TokenHolders, external_api::moralis};

/// Holds the results for the token holder check.
///
/// # Fields
///
/// * `top_holder_percentage` - The percentage of the total token supply owned by the largest (non-locked and non-burned) holder.
/// * `percentage_tokens_burned_or_locked` - The percentage of the total token supply that is either burned or held in locked addresses.
/// * `top_holder_more_than_10_percent_of_tokens` - A boolean indicator whether the top holder owns more than the threshold of tokens.
#[derive(Debug, Default)]
pub struct TokenHolderCheck {
    // pub creator_holder_percentage: f64, // Uncomment if implementation for creator holdings is added
    pub top_holder_percentage: f64,
    pub percentage_tokens_burned_or_locked: f64,
    // pub creator_owns_more_than_10_percent_of_tokens: bool, // Uncomment if implementation for creator holdings is added
    pub top_holder_more_than_10_percent_of_tokens: bool,
}

/// Retrieves and computes token holder metrics for a given token.
///
/// This asynchronous function performs the following steps:
///
/// 1. Retrieves the total token supply using the ERC20 token interface.
/// 2. Fetches token holder data using the external Moralis API.
/// 3. Sums up balances for tokens in "locked" addresses (depending on the current chain).
/// 4. Identifies the top token holder among non-locked tokens.
/// 5. Computes the percentage of tokens held by the top holder and the percentage that is burned or locked.
/// 6. Determines if the top holder's balance exceeds a defined threshold.
///
/// # Arguments
///
/// * `token` - A reference to the ERC20 token data.
/// * `client` - A shared reference (Arc) to the WebSocket provider used for asynchronous operations.
///
/// # Returns
///
/// * `Ok(Some(TokenHolderCheck))` if successful and there is valid token holder information.
/// * `Ok(None)` if no token holder information could be found or the top holder balance is zero.
/// * `Err` if any of the asynchronous calls or conversions fail.
pub async fn get_token_holder_check(
    token: &ERC20Token,
    // creator_address: &str,
    client: &Arc<Provider<Ws>>,
) -> Result<Option<TokenHolderCheck>> {
    // Step 1: Get the total token supply.
    let total_supply = token.get_total_token_supply(client).await?;

    // Retrieve the token holders list using Moralis API.
    let token_address = address_to_string(token.address);
    let top_holders: Vec<TokenHolders> = moralis::get_token_holder_list(&token_address).await?;

    if top_holders.is_empty() {
        // No token holders found yet.
        return Ok(None);
    }

    let mut top_holder = TokenHolders::default();
    // let mut creator_holdings = TokenHolders::default(); // For future use if tracking creator holdings

    // Sum up balances for locked or burned tokens.
    let mut burnt_or_locked_balance = U256::zero();

    for info in top_holders.iter() {
        // Determine if the address is a known locked address.
        let mut is_burned_or_locked = false;

        if CHAIN == Chain::Mainnet {
            // On Mainnet, check against the main network's locked addresses.
            if TOKEN_LOCKERS_MAINNET.contains(&info.holder.as_str()) {
                burnt_or_locked_balance += info.quantity;
                is_burned_or_locked = true;
            }
        } else {
            // For other chains, check against the base network's locked addresses.
            if TOKEN_LOCKERS_BASE.contains(&info.holder.as_str()) {
                burnt_or_locked_balance += info.quantity;
                is_burned_or_locked = true;
            }
        }

        // Only consider non-locked addresses for the top holder.
        if !is_burned_or_locked {
            if top_holder.quantity < info.quantity {
                top_holder = TokenHolders {
                    holder: info.holder.clone(), // Using clone here since String implements Clone.
                    quantity: info.quantity,
                };
            }
        }
    }

    // If no valid top holder is found (i.e., non-zero quantity), return None.
    if top_holder.quantity == U256::zero() {
        return Ok(None);
    }

    println!(
        "top holder for {} token is {} with {}",
        token.name, top_holder.holder, top_holder.quantity
    );

    // Calculate maximum threshold allowed for a token holder.
    let max_token_threshold =
        total_supply * U256::from(TOKEN_HOLDER_THRESHOLD_PERCENTAGE as u64) / U256::from(100_u64);

    // Compute the check data using precise conversion from U256 divisions.
    let token_holder_check = TokenHolderCheck {
        // Uncomment and adjust if using creator holdings in the future:
        // creator_holder_percentage: 100_f64 * u256_div_u256_to_f64(creator_holdings.quantity, total_supply)?,
        top_holder_percentage: 100_f64 * u256_div_u256_to_f64(top_holder.quantity, total_supply)?,
        percentage_tokens_burned_or_locked: 100_f64
            * u256_div_u256_to_f64(burnt_or_locked_balance, total_supply)?,
        top_holder_more_than_10_percent_of_tokens: top_holder.quantity > max_token_threshold,
    };

    println!(
        "top holder more than 10% ? => {}",
        token_holder_check.top_holder_more_than_10_percent_of_tokens
    );

    Ok(Some(token_holder_check))
}

/// Safely divides two U256 numbers and converts the result to an f64.
///
/// The division is performed by scaling the numerator to preserve precision (up to 6 decimal places)
/// and then converting it to f64. If the conversion fails, an error is returned.
///
/// # Arguments
///
/// * `numerator` - The U256 numerator.
/// * `denominator` - The U256 denominator.
///
/// # Returns
///
/// * `Ok(f64)` representing the precise division result.
/// * An `Err` if the conversion from U256 to f64 fails.
///
/// # Safety
///
/// Ensure that the numerator is less than the denominator when higher precision is required.
/// This function uses a scale factor of 10^6 to retain fractional precision.
pub fn u256_div_u256_to_f64(numerator: U256, denominator: U256) -> Result<f64> {
    // Scale factor to retain precision up to 6 decimal places.
    let scale = U256::exp10(6);

    // Scale up the numerator before division.
    let scaled_value = numerator * scale / denominator;

    // Convert the scaled U256 value to f64.
    let value = u256_to_f64(scaled_value)
        .ok_or_else(|| anyhow!("Conversion from U256 to f64 failed"))?
        / 1e6_f64;

    Ok(value)
}
