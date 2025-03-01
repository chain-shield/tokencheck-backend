//! This module contains functionality for simulating token transactions on an Anvil test
//! environment using the Uniswap V2 router. It provides methods to simulate adding liquidity,
//! buying tokens for WETH, and selling tokens for WETH.

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_v3_router::{ExactInputSingleParams, UNISWAP_V3_ROUTER};
use crate::data::chain_data::CHAIN_DATA;
use crate::data::token_data::ERC20Token;
use crate::token_check::anvil::buy_sell_uniswap_v2::extract_revert_reason;
use crate::token_check::anvil::tx_trait::Txs;
use crate::utils::tx::{get_amount_out_uniswap_v3, test_amount_of_token_to_purchase, TxSlippage};
use ethers::types::Address;
use ethers::types::U256;
use ethers::utils::format_units;
use log::error;

use super::simlator::AnvilTestSimulator;

impl AnvilTestSimulator {
    /// Simulates buying a token for WETH using the Uniswap V3 router.
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
    pub async fn simulate_buying_token_on_uniswap_v3_for_weth(
        &self,
        token: &ERC20Token,
    ) -> anyhow::Result<U256> {
        let router_address: Address = CHAIN_DATA
            .get_address(&token.chain)
            .uniswap_v3_router
            .parse()?;
        let weth_address: Address = CHAIN_DATA.get_address(&token.chain).weth.parse()?;

        let mut new_token_balance = U256::from(0);
        let router = UNISWAP_V3_ROUTER::new(router_address, self.signed_client.clone());

        // Ensure the wallet has sufficient ETH
        self.get_wallet_eth_balance().await?;
        let amount_in = test_amount_of_token_to_purchase()?;

        // Calculate the minimum amount of token expected from the swap based on slippage settings.
        // Use fixed fee tier of 3000 (0.3%) for V3
        let amount_out_min = get_amount_out_uniswap_v3(
            weth_address,
            token.address,
            10000, // 1%     fee tier
            amount_in,
            TxSlippage::FivePercent,
            &token.chain,
            &self.client,
        )
        .await?;

        let amount_out_min_readable = format_units(amount_out_min, 18u32)?;
        println!("calculated amount out min {}", amount_out_min_readable);
        println!("........................................................");

        // Prepare the swap transaction:
        // The swap function will exchange ETH for the specified token using V3 exactInputSingle
        let params = ExactInputSingleParams {
            token_in: weth_address,
            token_out: token.address,
            fee: 3000, // 0.3% fee tier
            recipient: self.sender,
            amount_in,
            amount_out_minimum: amount_out_min,
            sqrt_price_limit_x96: U256::zero(), // No price limit
        };

        let tx = router.exact_input_single(params).value(amount_in);

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

    /// Simulates selling a token for WETH using the Uniswap V3 router.
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
    pub async fn simulate_selling_token_on_uniswap_v3_for_weth(
        &self,
        token: &ERC20Token,
    ) -> anyhow::Result<U256> {
        let router_address: Address = CHAIN_DATA
            .get_address(&token.chain)
            .uniswap_v3_router
            .parse()?;
        let weth_address: Address = CHAIN_DATA.get_address(&token.chain).weth.parse()?;
        let token_contract = ERC20::new(token.address, self.signed_client.clone());

        let mut new_token_balance = U256::from(0);
        let router = UNISWAP_V3_ROUTER::new(router_address, self.signed_client.clone());

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
        let amount_out_min = get_amount_out_uniswap_v3(
            token.address,
            weth_address,
            10000, // 1% fee tier
            amount_to_sell,
            TxSlippage::FivePercent,
            &token.chain,
            &self.client,
        )
        .await?;

        println!("........................................................");

        // Prepare the swap transaction to convert tokens to ETH using V3's exactInputSingle
        let params = ExactInputSingleParams {
            token_in: token.address,
            token_out: weth_address,
            fee: 10000, // 1%
            recipient: self.sender,
            amount_in: amount_to_sell,
            amount_out_minimum: amount_out_min,
            sqrt_price_limit_x96: U256::zero(), // No price limit
        };

        let tx = router.exact_input_single(params);

        println!("sending swap transaction");
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
