use crate::app_config::{AppMode, APP_MODE};
use crate::data::chain_data::CHAIN_DATA;
use crate::data::dex::Dex;
use crate::data::token_data::ERC20Token;
use crate::token_check::anvil::simlator::AnvilTestSimulator;
use crate::token_check::anvil::tx_trait::Txs;
use ethers::providers::Middleware;
use ethers::types::U256;
use log::error;

/// Describes the validation status of a token after running simulated buy and sell operations.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenStatus {
    /// The token is deemed valid.
    Legit,
    /// The token cannot be sold after simulation.
    CannotSell,
    /// The token cannot be bought in simulation, or buying encountered issues.
    CannotBuy,
}

impl ERC20Token {
    /// Takes a snapshot of the current blockchain state using anvil by simulating buy and sell operations.
    ///
    /// This function runs through several steps to check token behavior:
    /// 1. Optionally simulate adding liquidity based on the provided `TokenLiquid` variant.
    /// 2. Attempt to buy the token. If the purchase fails or no tokens are received, the token is marked as unable to be bought.
    /// 3. Simulate block mining (time elapse) to mimic a real blockchain environment.
    /// 4. Check that the token balance remains stable after simulated time elapse.
    /// 5. Execute a dummy transfer to further validate token stability.
    /// 6. Lastly, attempt to sell the token. If the balance after sale is not zero, the token is marked as unsellable.
    ///
    /// # Arguments
    ///
    /// * `liquidity_status` - The current liquidity condition of the token. It can be either a need to add liquidity
    ///   using a provided transaction or an indication that there is enough liquidity.
    ///
    /// # Returns
    ///
    /// Returns a `TokenStatus` indicating the result of the validation simulation wrapped in an `anyhow::Result`.
    ///
    /// # Errors
    ///
    /// If any of the simulation steps fail (for example, during the sell simulation), the error will be propagated.
    pub async fn validate_with_simulated_buy_sell(&self) -> anyhow::Result<TokenStatus> {
        // Launch a new anvil node for validation using the websocket URL.
        let ws_url = if APP_MODE == AppMode::Production {
            CHAIN_DATA.get_address(&self.chain).alchemy_url.clone()
        } else {
            CHAIN_DATA.get_address(&self.chain).ws_url.clone()
        };
        let anvil = AnvilTestSimulator::new(&ws_url, &self.chain).await?;

        let top_dex_data = self
            .clone()
            .token_dex
            .expect("is_liquidity_locked: no token dex found");

        println!("validating token...");

        // Attempt to buy the token using the anvil simulator.
        let buy_result = match top_dex_data.dex {
            Dex::UniswapV2 => {
                anvil
                    .simulate_buying_token_on_uniswap_v2_for_weth(&top_dex_data, self)
                    .await
            }
            Dex::UniswapV3 => {
                anvil
                    .simulate_buying_token_on_uniswap_v3_for_weth(&top_dex_data, self)
                    .await
            }
            _ => return Ok(TokenStatus::CannotBuy),
        };

        if let Err(err) = buy_result {
            error!("Buy transaction failed with error: {:?}", err);
            // If buying fails, the token is considered not purchasable.
            return Ok(TokenStatus::CannotBuy);
        }

        // Unwrap the token balance received after a successful simulated purchase.
        let token_balance = buy_result?;

        println!("check token balance after purchase");
        if token_balance == U256::from(0) {
            println!("No tokens received after buy, reverting...");
            // If no tokens are received, mark as unable to buy.
            return Ok(TokenStatus::CannotBuy);
        }

        // Simulate block creation to mimic a passing of time.
        println!("simulating creating blocks");
        let _ = anvil
            .signed_client
            .provider()
            .request::<_, String>("evm_mine", None::<()>)
            .await?;

        println!("check token balance after five minutes");
        // Check that token balance after time elapse is not degraded.
        let balance_after_buy = anvil
            .get_wallet_token_balance_by_address(self.address)
            .await?;
        if balance_after_buy < token_balance {
            println!("Tokens are dropping or going to zero after 5 mins...");
            // If the token balance drops, mark as non-purchasable.
            return Ok(TokenStatus::CannotBuy);
        }

        // Simulate a dummy transfer to further verify token stability.
        println!("do dummy transfer");
        anvil
            .do_dummy_transfer(self.address, balance_after_buy)
            .await?;

        println!("check token balance after transfers");
        // Verify that the token balance remains stable after the transfer.
        let balance_after_transfer = anvil
            .get_wallet_token_balance_by_address(self.address)
            .await?;
        if balance_after_transfer < token_balance {
            println!("Tokens are dropping or going to zero after transfer...");
            // If token balance drops post-transfer, mark as non-purchasable.
            return Ok(TokenStatus::CannotBuy);
        }

        // Attempt to sell the token.
        println!("simulate selling token for validation");
        // Attempt to buy the token using the anvil simulator.
        let sell_result = match top_dex_data.dex {
            Dex::UniswapV2 => {
                anvil
                    .simulate_selling_token_on_uniswap_v2_for_weth(&top_dex_data, self)
                    .await
            }
            Dex::UniswapV3 => {
                anvil
                    .simulate_selling_token_on_uniswap_v3_for_weth(&top_dex_data, self)
                    .await
            }
            _ => return Ok(TokenStatus::CannotBuy),
        };

        match sell_result {
            Ok(_) => {
                // After a successful sell, ensure that the token balance becomes zero.
                let balance_after_sell = anvil
                    .get_wallet_token_balance_by_address(self.address)
                    .await?;
                if balance_after_sell != U256::from(0) {
                    println!("Cannot sell {}, scam alert", self.name);
                    // If the balance is not zero after sale, the token is flagged as unsellable.
                    return Ok(TokenStatus::CannotSell);
                }

                println!("{} is legit", self.name);
                Ok(TokenStatus::Legit)
            }
            Err(err) => {
                println!("Sell transaction failed: {:?}", err);
                Err(err)
            }
        }
    }
}
