//! This module provides functions for interacting with ERC20 tokens and
//! liquidity pools on decentralized exchanges (e.g., Uniswap v2).
//!
//! It includes methods to query the total liquidity pool token supply,
//! the total token supply, check if the token has sufficient liquidity,
//! and verify if there is enough liquidity to execute a trade operation.
//! These functions interact with on-chain contracts via an Ethereum provider
//! connected using WebSocket (Ws).

use ethers::providers::{Provider, Ws};
use ethers::types::U256;
use std::sync::Arc;

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_pair::UNISWAP_PAIR;
use crate::app_config::{MIN_LIQUIDITY, MIN_RESERVE_ETH_FACTOR, MIN_TRADE_FACTOR};
use crate::data::token_data::ERC20Token;
use crate::utils::tx::amount_of_token_to_purchase;

impl ERC20Token {
    /// Returns the total number of liquidity pool (LP) tokens supplied on Uniswap v2.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference-counted provider connected via WebSockets to an Ethereum node.
    ///
    /// # Returns
    ///
    /// * `Ok(U256)` containing the total supply of LP tokens if successful.
    /// * `Err(anyhow::Error)` if an error occurs during the contract call.
    pub async fn get_total_liquidity_token_supply(
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<U256> {
        // Connect to the Uniswap pair contract at the stored pair address.
        let pool = UNISWAP_PAIR::new(self.token_dex.pair_or_pool_address, client.clone());

        // Query the total supply of liquidity tokens.
        let supply = pool.total_supply().call().await?;

        Ok(supply)
    }

    /// Returns the total number of tokens minted for this ERC20 contract.
    ///
    /// This method calls the token contract to retrieve its total supply.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference-counted provider connected via WebSockets to an Ethereum node.
    ///
    /// # Returns
    ///
    /// * `Ok(U256)` representing the total token supply.
    /// * `Err(anyhow::Error)` if an error occurs during the contract call.
    pub async fn get_total_token_supply(&self, client: &Arc<Provider<Ws>>) -> anyhow::Result<U256> {
        // Connect to the ERC20 token contract using the stored token address.
        let token_contract = ERC20::new(self.address, client.clone());

        // Retrieve the total token supply from the contract.
        let supply = token_contract.total_supply().call().await?;

        Ok(supply)
    }

    /// Checks if the token has enough liquidity based on a preset liquidity threshold.
    ///
    /// This function compares the available liquidity (specifically, the reserve of
    /// the paired ETH supply) with a minimum liquidity requirement defined in the configuration.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference-counted provider connected via WebSockets to an Ethereum node.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the liquidity is above the minimum threshold.
    /// * `Ok(false)` if the liquidity is below the required threshold.
    /// * `Err(anyhow::Error)` in case of an error during the contract call.
    pub async fn has_enough_liquidity(&self, client: &Arc<Provider<Ws>>) -> anyhow::Result<bool> {
        // Connect to the Uniswap pair contract.
        let pool = UNISWAP_PAIR::new(self.token_dex.pair_or_pool_address, client.clone());

        // Retrieve the current reserves from the liquidity pool.
        let (reserve0, reserve1, _) = pool.get_reserves().call().await?;

        // Determine which reserve corresponds to ETH based on token position.
        let eth_supply = if self.token_dex.is_token_0 {
            reserve1
        } else {
            reserve0
        };

        Ok(eth_supply >= MIN_LIQUIDITY)
    }

    /// Retrieves the liquidity value from the pool.
    ///
    /// This function returns the current amount of ETH supply available in the liquidity pool.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference-counted provider connected via WebSockets to an Ethereum node.
    ///
    /// # Returns
    ///
    /// * `Ok(u128)` containing the liquidity value (ETH supply).
    /// * `Err(anyhow::Error)` if an error occurs during the contract call.
    pub async fn get_liquidity(&self, client: &Arc<Provider<Ws>>) -> anyhow::Result<u128> {
        // Connect to the Uniswap pair contract.
        let pool = UNISWAP_PAIR::new(self.token_dex.pair_or_pool_address, client.clone());

        // Retrieve the reserves data.
        let (reserve0, reserve1, _) = pool.get_reserves().call().await?;

        // Determine the ETH-based liquidity depending on token arrangement.
        let eth_supply = if self.token_dex.is_token_0 {
            reserve1
        } else {
            reserve0
        };

        Ok(eth_supply)
    }

    /// Checks if there is sufficient liquidity for a trade.
    ///
    /// This function evaluates whether the liquidity pool can support a trade operation.
    /// It verifies both the token supply and the ETH reserve against minimum trade and reserve factors.
    ///
    /// # Arguments
    ///
    /// * `tokens_to_sell` - The amount of tokens intended to be sold.
    /// * `client` - A reference-counted provider connected via WebSockets to an Ethereum node.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the liquidity is adequate for the trade.
    /// * `Ok(false)` if the liquidity conditions are not met.
    /// * `Err(anyhow::Error)` if there is an error during interaction with the contract.
    pub async fn has_enough_liquidity_for_trade(
        &self,
        tokens_to_sell: U256,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<bool> {
        // Connect to the Uniswap pair contract.
        let pool = UNISWAP_PAIR::new(self.token_dex.pair_or_pool_address, client.clone());

        // Calculate the amount of tokens required to execute the trade.
        let eth_amount_used_for_purchase = amount_of_token_to_purchase()?;

        // Retrieve the reserves available in the liquidity pool.
        let (reserve0, reserve1, _) = pool.get_reserves().call().await?;

        // Based on the token's position in the pair, assign the proper reserves for ETH and token supplies.
        let (eth_supply, token_supply) = if self.token_dex.is_token_0 {
            (reserve1, reserve0)
        } else {
            (reserve0, reserve1)
        };

        // Verify that both token supply and ETH reserve meet the minimum requirements for a safe trade.
        let enough_liquidity = tokens_to_sell * MIN_TRADE_FACTOR < U256::from(token_supply)
            && eth_amount_used_for_purchase * MIN_RESERVE_ETH_FACTOR < U256::from(eth_supply);

        Ok(enough_liquidity)
    }
}
