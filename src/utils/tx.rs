use crate::abi::erc20::ERC20;
use crate::abi::uniswap_router_v2::UNISWAP_V2_ROUTER;
use crate::app_config::CHAIN;
use crate::data::contracts::CONTRACT;
use anyhow::{anyhow, Result};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::types::{Address, Block, BlockNumber, Bytes, H256, U256, U64};
use ethers::utils::format_units;
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

#[derive(PartialEq, Eq)]
pub enum TxSlippage {
    OnePercent,
    TwoPercent,
    ThreePercent,
    FivePercent,
    TenPercent,
    None,
}

pub async fn get_amount_out_uniswap_v2(
    token_in: Address,
    token_out: Address,
    amount_in: U256,
    slippage_tolerance: TxSlippage,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<U256> {
    let uniswap_v2_router_address: Address = CONTRACT.get_address().uniswap_v2_router.parse()?;
    let router = UNISWAP_V2_ROUTER::new(uniswap_v2_router_address, client.clone());

    let amounts = router
        .get_amounts_out(amount_in, vec![token_in, token_out])
        .call()
        .await?;
    let base_amount_out = amounts[amounts.len() - 1];

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

// ************************** WALLET ***************************************************
pub fn get_wallet() -> anyhow::Result<Wallet<SigningKey>> {
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not found in .env file");

    let wallet = LocalWallet::from_str(&private_key)?.with_chain_id(CHAIN);
    Ok(wallet)
}

pub fn get_test_wallet() -> anyhow::Result<Wallet<SigningKey>> {
    let private_key = env::var("PRIVATE_KEY_2").expect("PRIVATE_KEY_2 not found in .env file");

    let wallet = LocalWallet::from_str(&private_key)?.with_chain_id(CHAIN);
    Ok(wallet)
}
// ************************** BLOCK ***************************************************
pub async fn get_current_block(client: &Arc<Provider<Ws>>) -> anyhow::Result<(Block<H256>, U64)> {
    // get the latest block
    let block = client
        .get_block(BlockNumber::Latest)
        .await?
        .ok_or_else(|| {
            anyhow!("Could not retrieve the latest block for next_base_fee calculation")
        })?;

    // block number
    let block_number = block
        .number
        .ok_or_else(|| anyhow!("missing block number"))?;

    Ok((block, block_number))
}

/// Calculate the next block base fee with minor randomness
pub fn calculate_next_block_base_fee(block: &Block<H256>) -> Result<U256> {
    let base_fee = block
        .base_fee_per_gas
        .ok_or_else(|| anyhow!("Block missing base fee per gas"))?;

    let gas_used = block.gas_used;
    let mut target_gas_used = block.gas_limit / 2;
    if target_gas_used.is_zero() {
        target_gas_used = U256::one();
    }

    let new_base_fee = if gas_used > target_gas_used {
        base_fee + ((base_fee * (gas_used - target_gas_used)) / target_gas_used) / U256::from(8u64)
    } else {
        base_fee - ((base_fee * (target_gas_used - gas_used)) / target_gas_used) / U256::from(8u64)
    };

    let seed = rand::thread_rng().gen_range(0..9);
    Ok(new_base_fee + seed)
}

pub fn get_transaction_cost_in_eth(
    txs: &[Eip1559TransactionRequest],
    gas_cost: U256,
    next_base_fee: U256,
) -> Result<U256> {
    let total_gas_price = txs
        .iter()
        .map(|tx| min(tx.max_fee_per_gas.unwrap_or_default(), next_base_fee))
        .fold(U256::zero(), |acc, x| acc.saturating_add(x));

    let transaction_cost = gas_cost.checked_mul(total_gas_price).ok_or_else(|| {
        anyhow!("overflow when computing transaction cost (gas_cost * gas_price)")
    })?;

    Ok(transaction_cost)
}

pub fn amount_of_token_to_purchase() -> anyhow::Result<U256> {
    let amount_to_buy =
        std::env::var("TEST_BUY_AMOUNT").expect("TEST_BUY_AMOUNT is not set in .env");
    let amount_in = ethers::utils::parse_ether(amount_to_buy)?;
    Ok(amount_in)
}

pub fn test_amount_of_token_to_purchase() -> anyhow::Result<U256> {
    let amount_to_buy = std::env::var("TOKEN_TO_BUY_FOR_ANVIL_TEST")
        .expect("TOKEN_TO_BUY_FOR_ANVIL_TEST is not set in .env");
    let amount_in = ethers::utils::parse_ether(amount_to_buy)?;
    // let purchase_amount = format_units(amount_in, "ether")?;
    // println!("buying {} of token", purchase_amount);
    Ok(amount_in)
}
