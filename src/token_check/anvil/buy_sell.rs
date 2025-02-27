//! This module contains functionality for simulating token transactions on an Anvil test
//! environment using the Uniswap V2 router. It provides methods to simulate adding liquidity,
//! buying tokens for WETH, and selling tokens for WETH.

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_router_v2::UNISWAP_V2_ROUTER;
use crate::data::chain_data::CHAIN_DATA;
use crate::data::token_data::ERC20Token;
use crate::token_check::anvil::tx_trait::Txs;
use crate::utils::tx::{get_amount_out_uniswap_v2, test_amount_of_token_to_purchase, TxSlippage};
use ethers::types::Address;
use ethers::types::U256;
use ethers::utils::format_units;
use log::error;
use std::error::Error;

use super::simlator::AnvilTestSimulator;

impl AnvilTestSimulator {
    /// Simulates buying a token for WETH using the Uniswap V2 router.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves required addresses (router and WETH).
    /// 2. Checks wallet ETH balance.
    /// 3. Determines the amount of token to purchase.
    /// 4. Calculates the minimum amount of tokens expected from the swap.
    /// 5. Sends the swap transaction and awaits its confirmation.
    /// 6. Traces the transaction and updates token balance.
    ///
    /// # Arguments
    ///
    /// * `token` - A reference to the [`ERC20Token`] that is being purchased.
    ///
    /// # Returns
    ///
    /// * [`anyhow::Result<U256>`] - Returns the updated token balance after the purchase.
    pub async fn simulate_buying_token_for_weth(&self, token: &ERC20Token) -> anyhow::Result<U256> {
        let router_address: Address = CHAIN_DATA
            .get_address(&token.chain)
            .uniswap_v2_router
            .parse()?;
        let weth_address: Address = CHAIN_DATA.get_address(&token.chain).weth.parse()?;

        let mut new_token_balance = U256::from(0);
        let router = UNISWAP_V2_ROUTER::new(router_address, self.signed_client.clone());

        // Ensure the wallet has sufficient ETH
        self.get_wallet_eth_balance().await?;
        let amount_in = test_amount_of_token_to_purchase()?;

        // Calculate the minimum amount of token expected from the swap based on slippage settings.
        let amount_out_min = get_amount_out_uniswap_v2(
            weth_address,
            token.address,
            amount_in,
            TxSlippage::FivePercent,
            &token.chain,
            &self.client,
        )
        .await?;

        let amount_out_min_readable = format_units(amount_out_min, 18u32)?;
        println!("calculated amount out min {}", amount_out_min_readable);
        println!("........................................................");

        // Set a deadline timestamp 5 minutes in the future.
        let deadline = self.get_current_timestamp().await? + 300;

        // Prepare the swap transaction:
        // The swap function will exchange ETH for the specified token.
        let tx = router
            .swap_exact_eth_for_tokens(
                amount_out_min,
                vec![weth_address, token.address],
                self.sender,
                U256::from(deadline),
            )
            .value(amount_in);

        // Estimate the gas for the transaction.
        let gas_estimate = tx.estimate_gas().await?;
        println!("gas est => {}", gas_estimate);

        println!("sending tx");
        let pending_tx_result = tx.send().await;

        match pending_tx_result {
            Ok(pending_tx) => {
                // Transaction sent successfully.
                println!("Transaction sent, awaiting receipt");
                let tx_hash = pending_tx.tx_hash();
                println!("tx_hash => {:?}", tx_hash);

                println!("awaiting tx receipt");
                // Await the transaction receipt and handle the absence of a receipt.
                let receipt = pending_tx
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Transaction receipt not found"))?;

                let tx_hash = receipt.transaction_hash;

                // Trace the transaction details.
                self.trace_transaction(tx_hash).await?;

                println!("........................................................");
                println!("balance after buying {}...", token.name);
                new_token_balance = self
                    .get_wallet_token_balance_by_address(token.address)
                    .await?;
                self.get_wallet_eth_balance().await?;
            }
            Err(tx_err) => {
                // Transaction sending failed; log the error details.
                error!("Failed to send transaction: {:?}", tx_err);

                // Attempt to extract and log a revert reason from the error.
                if let Some(revert_reason) = extract_revert_reason(&tx_err) {
                    error!("Revert reason: {}", revert_reason);
                } else {
                    error!("Failed to extract revert reason");
                }
            }
        }

        println!("token balance after purchase => {}", new_token_balance);
        Ok(new_token_balance)
    }

    /// Simulates selling a token for WETH using the Uniswap V2 router.
    ///
    /// The function performs these steps:
    /// 1. Retrieves router and WETH addresses.
    /// 2. Approves the swap router to trade the token.
    /// 3. Calculates the minimum acceptable amount (considering slippage).
    /// 4. Sends the swap transaction and awaits its confirmation.
    /// 5. Traces the transaction and updates token balance.
    ///
    /// # Arguments
    ///
    /// * `token` - A reference to the [`ERC20Token`] to be sold.
    ///
    /// # Returns
    ///
    /// * [`anyhow::Result<U256>`] - Returns the updated token balance after selling the token.
    pub async fn simulate_selling_token_for_weth(
        &self,
        token: &ERC20Token,
    ) -> anyhow::Result<U256> {
        let router_address: Address = CHAIN_DATA
            .get_address(&token.chain)
            .uniswap_v2_router
            .parse()?;
        let weth_address: Address = CHAIN_DATA.get_address(&token.chain).weth.parse()?;
        let token_contract = ERC20::new(token.address, self.signed_client.clone());

        let mut new_token_balance = U256::from(0);
        let router = UNISWAP_V2_ROUTER::new(router_address, self.signed_client.clone());

        println!("........................................................");
        self.get_wallet_eth_balance().await?;
        let amount_to_sell = self
            .get_wallet_token_balance_by_address(token.address)
            .await?;

        // Approve the router to spend the token on behalf of the sender.
        token_contract
            .approve(router_address, amount_to_sell)
            .send()
            .await?;

        // Calculate the minimum amount of WETH that should be received.
        let amount_out_min = get_amount_out_uniswap_v2(
            token.address,
            weth_address,
            amount_to_sell,
            TxSlippage::FivePercent,
            &token.chain,
            &self.client,
        )
        .await?;

        println!("........................................................");

        // Set a deadline timestamp 5 minutes in the future.
        let deadline = self.get_current_timestamp().await? + 300;

        // Prepare the swap transaction to convert tokens to ETH.
        let tx = router.swap_exact_tokens_for_eth(
            amount_to_sell,
            amount_out_min,
            vec![token.address, weth_address],
            self.sender,
            U256::from(deadline),
        );

        println!("sending swap transcation");
        let pending_tx_result = tx.send().await;

        match pending_tx_result {
            Ok(pending_tx) => {
                // Transaction was sent successfully.
                println!("Transaction sent, awaiting receipt");
                let tx_hash = pending_tx.tx_hash();
                println!("tx_hash => {:?}", tx_hash);

                println!("awaiting transaction receipt");
                // Await receipt; handle error if receipt is missing.
                let receipt = pending_tx
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Transaction receipt not found"))?;

                let tx_hash = receipt.transaction_hash;
                self.trace_transaction(tx_hash).await?;

                new_token_balance = self
                    .get_wallet_token_balance_by_address(token.address)
                    .await?;
                self.get_wallet_eth_balance().await?;
            }
            Err(tx_err) => {
                // Log failure details for debugging.
                error!("Failed to send transaction: {:?}", tx_err);

                // Attempt to extract a revert reason from the error.
                if let Some(revert_reason) = extract_revert_reason(&tx_err) {
                    error!("Revert reason: {}", revert_reason);
                } else {
                    error!("Failed to extract revert reason");
                }
            }
        }

        Ok(new_token_balance)
    }
}

/// Attempts to extract a revert reason from an error by scanning its string representation
/// as well as all underlying source errors.
///
/// # Arguments
///
/// * `err` - A reference to an error that implements the [`Error`] trait.
///
/// # Returns
///
/// * `Option<String>` - The detected revert reason if present, otherwise `None`.
pub fn extract_revert_reason<E>(err: &E) -> Option<String>
where
    E: Error,
{
    // Check the top-level error for a revert reason.
    if let Some(reason) = parse_revert(&format!("{}", err)) {
        return Some(reason);
    }

    // Recursively check all underlying errors.
    let mut source = err.source();
    while let Some(src) = source {
        if let Some(reason) = parse_revert(&format!("{}", src)) {
            return Some(reason);
        }
        source = src.source();
    }

    None
}

/// Parses an error string and attempts to extract a revert reason from common patterns.
///
/// This function looks for patterns like "execution reverted:" or
/// "reverted with reason string '" in the provided error string.
///
/// # Arguments
///
/// * `error_str` - The error string to be parsed.
///
/// # Returns
///
/// * `Option<String>` - The extracted revert reason if found, otherwise `None`.
fn parse_revert(error_str: &str) -> Option<String> {
    // Common pattern 1:
    if let Some(start_idx) = error_str.find("execution reverted:") {
        let offset = start_idx + "execution reverted:".len();
        let reason = error_str[offset..].trim();
        if !reason.is_empty() {
            return Some(reason.to_string());
        }
    }

    // Common pattern 2:
    // Example: "... reverted with reason string 'BadStuff' ..."
    if let Some(start_idx) = error_str.find("reverted with reason string '") {
        let offset = start_idx + "reverted with reason string '".len();
        if let Some(end_idx) = error_str[offset..].find('\'') {
            let reason = &error_str[offset..offset + end_idx];
            if !reason.is_empty() {
                return Some(reason.to_string());
            }
        }
    }

    None
}
