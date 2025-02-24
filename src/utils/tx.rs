//! Utility functions for handling transaction amounts, wallet management, block information,
//! and fee calculations.

use crate::abi::uniswap_router_v2::UNISWAP_V2_ROUTER;
use crate::app_config::CHAIN;
use crate::data::chain_data::CHAIN_DATA;
use anyhow::{anyhow, Context, Result};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::types::{Address, Block, BlockNumber, H256, U256, U64};
use ethers::{
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer, Wallet},
    types::Eip1559TransactionRequest,
};
use rand::Rng;
use std::cmp::min;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

/// Enum representing the possible slippage tolerances for a transaction.
///
/// # Variants
/// - `OnePercent`: 1% slippage tolerance
/// - `TwoPercent`: 2% slippage tolerance
/// - `ThreePercent`: 3% slippage tolerance
/// - `FivePercent`: 5% slippage tolerance
/// - `TenPercent`: 10% slippage tolerance
/// - `None`: No slippage adjustment
#[derive(PartialEq, Eq)]
pub enum TxSlippage {
    OnePercent,
    TwoPercent,
    ThreePercent,
    FivePercent,
    TenPercent,
    None,
}

/// Calculates the output amount for a given input token amount using the Uniswap V2 router,
/// applying a specified slippage tolerance.
///
/// # Parameters
/// - `token_in`: The input token address.
/// - `token_out`: The output token address.
/// - `amount_in`: The amount of input tokens.
/// - `slippage_tolerance`: The allowed slippage tolerance as defined in `TxSlippage`.
/// - `client`: An Arc-wrapped WebSocket provider for blockchain interactions.
///
/// # Returns
/// Returns the calculated output amount as a `U256`.
pub async fn get_amount_out_uniswap_v2(
    token_in: Address,
    token_out: Address,
    amount_in: U256,
    slippage_tolerance: TxSlippage,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<U256> {
    // Parse the Uniswap V2 router address from the contract configuration.
    let uniswap_v2_router_address: Address = CHAIN_DATA
        .get_address(CHAIN)
        .uniswap_v2_router
        .parse()
        .context("Failed to parse Uniswap V2 router address")?;
    let router = UNISWAP_V2_ROUTER::new(uniswap_v2_router_address, client.clone());

    // Call the Uniswap V2 router to get output amounts for the given input.
    let amounts = router
        .get_amounts_out(amount_in, vec![token_in, token_out])
        .call()
        .await?;
    let base_amount_out = amounts[amounts.len() - 1];

    // Adjust the base output amount based on the provided slippage tolerance.
    let amount_out = match slippage_tolerance {
        TxSlippage::TenPercent => base_amount_out * U256::from(90) / U256::from(100),
        TxSlippage::FivePercent => base_amount_out * U256::from(95) / U256::from(100),
        TxSlippage::TwoPercent => base_amount_out * U256::from(98) / U256::from(100),
        TxSlippage::ThreePercent => base_amount_out * U256::from(97) / U256::from(100),
        TxSlippage::OnePercent => base_amount_out * U256::from(99) / U256::from(100),
        TxSlippage::None => base_amount_out,
    };

    Ok(amount_out)
}

/// Returns a wallet instance using the private key specified in the environment variable
/// `PRIVATE_KEY`.
///
/// # Errors
/// Fails if the environment variable `PRIVATE_KEY` is not set or if the key parsing fails.
pub fn get_wallet() -> anyhow::Result<Wallet<SigningKey>> {
    // Retrieve PRIVATE_KEY from environment, providing context on failure.
    let private_key = env::var("PRIVATE_KEY").context("PRIVATE_KEY not found in .env file")?;
    let wallet = LocalWallet::from_str(&private_key)?.with_chain_id(CHAIN);
    Ok(wallet)
}

/// Returns a test wallet instance using the private key specified in `PRIVATE_KEY_2`.
///
/// # Errors
/// Fails if the environment variable `PRIVATE_KEY_2` is not set or if the key parsing fails.
pub fn get_test_wallet() -> anyhow::Result<Wallet<SigningKey>> {
    // Retrieve PRIVATE_KEY_2 from environment, providing context on failure.
    let private_key = env::var("PRIVATE_KEY_2").context("PRIVATE_KEY_2 not found in .env file")?;
    let wallet = LocalWallet::from_str(&private_key)?.with_chain_id(CHAIN);
    Ok(wallet)
}

/// Retrieves the latest block from the given provider along with its block number.
///
/// # Parameters
/// - `client`: An Arc-wrapped WebSocket provider for fetching blockchain data.
///
/// # Returns
/// A tuple containing the latest `Block<H256>` and its block number (`U64`).
///
/// # Errors
/// Fails if the latest block or its number cannot be retrieved.
pub async fn get_current_block(client: &Arc<Provider<Ws>>) -> anyhow::Result<(Block<H256>, U64)> {
    // Fetch the latest block.
    let block = client
        .get_block(BlockNumber::Latest)
        .await?
        .ok_or_else(|| {
            anyhow!("Could not retrieve the latest block for next_base_fee calculation")
        })?;

    // Retrieve the block number, erroring if missing.
    let block_number = block
        .number
        .ok_or_else(|| anyhow!("missing block number"))?;

    Ok((block, block_number))
}

/// Calculates the next block's base fee with a small random variation based on the current block's data.
///
/// # Parameters
/// - `block`: A reference to the current block.
///
/// # Returns
/// A `U256` representing the new base fee.
///
/// # Errors
/// Fails if the current block is missing the base fee per gas.
pub fn calculate_next_block_base_fee(block: &Block<H256>) -> Result<U256> {
    // Ensure the block has a base fee per gas.
    let base_fee = block
        .base_fee_per_gas
        .ok_or_else(|| anyhow!("Block missing base fee per gas"))?;

    let gas_used = block.gas_used;
    let mut target_gas_used = block.gas_limit / 2;
    // Ensure target_gas_used is at least one to avoid division by zero.
    if target_gas_used.is_zero() {
        target_gas_used = U256::one();
    }

    // Adjust base fee based on gas usage relative to target.
    let new_base_fee = if gas_used > target_gas_used {
        base_fee + ((base_fee * (gas_used - target_gas_used)) / target_gas_used) / U256::from(8u64)
    } else {
        base_fee - ((base_fee * (target_gas_used - gas_used)) / target_gas_used) / U256::from(8u64)
    };

    // Add a small random seed (0 to 8 inclusive) for minor variability.
    let seed = rand::thread_rng().gen_range(0..9);
    Ok(new_base_fee + seed)
}

/// Calculates the total transaction cost in Ether based on a list of EIP-1559 transactions,
/// the provided gas cost, and the next block base fee.
///
/// # Parameters
/// - `txs`: A slice of EIP-1559 transaction requests.
/// - `gas_cost`: The total gas cost.
/// - `next_base_fee`: The computed next base fee from the current block.
///
/// # Returns
/// The transaction cost as a `U256`.
///
/// # Errors
/// Fails if there is an overflow during multiplication.
pub fn get_transaction_cost_in_eth(
    txs: &[Eip1559TransactionRequest],
    gas_cost: U256,
    next_base_fee: U256,
) -> Result<U256> {
    // Iterate over transactions and sum the minimum fee per gas (between max_fee_per_gas or next_base_fee).
    let total_gas_price = txs
        .iter()
        .map(|tx| min(tx.max_fee_per_gas.unwrap_or_default(), next_base_fee))
        .fold(U256::zero(), |acc, x| acc.saturating_add(x));

    // Calculate the total transaction cost.
    let transaction_cost = gas_cost.checked_mul(total_gas_price).ok_or_else(|| {
        anyhow!("overflow when computing transaction cost (gas_cost * gas_price)")
    })?;

    Ok(transaction_cost)
}

/// Retrieves the token purchase amount for testing from the environment variable `TEST_BUY_AMOUNT`.
///
/// # Returns
/// The purchase amount as a `U256` value.
///
/// # Errors
/// Fails if the environment variable is not set or if the value cannot be parsed.
pub fn amount_of_token_to_purchase() -> anyhow::Result<U256> {
    // Retrieve the amount string from the environment with context on failure.
    let amount_to_buy =
        env::var("TEST_BUY_AMOUNT").context("TEST_BUY_AMOUNT is not set in .env file")?;
    let amount_in = ethers::utils::parse_ether(amount_to_buy)?;
    Ok(amount_in)
}

/// Retrieves the token purchase amount for testing purposes from `TOKEN_TO_BUY_FOR_ANVIL_TEST`.
///
/// # Returns
/// The test purchase amount as a `U256` value in wei.
///
/// # Errors
/// Fails if the environment variable is not set or if the value cannot be parsed.
pub fn test_amount_of_token_to_purchase() -> anyhow::Result<U256> {
    // Retrieve the amount string from the environment with inline error context.
    let amount_to_buy = env::var("TOKEN_TO_BUY_FOR_ANVIL_TEST")
        .context("TOKEN_TO_BUY_FOR_ANVIL_TEST is not set in .env")?;
    let amount_in = ethers::utils::parse_ether(amount_to_buy)?;
    // The following commented out code can be used for additional logging if necessary.
    // let purchase_amount = format_units(amount_in, "ether")?;
    // println!("buying {} of token", purchase_amount);
    Ok(amount_in)
}
