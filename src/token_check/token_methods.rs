use ethers::providers::{Provider, Ws};
use ethers::types::U256;
use std::sync::Arc;

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_pair::UNISWAP_PAIR;
use crate::abi::uniswap_pool::UNISWAP_V3_POOL;
use crate::data::dex::Dex;
use crate::data::token_data::ERC20Token;
use crate::dex::dex_data::TokenDexData;

impl ERC20Token {
    pub async fn get_total_liquidity_token_supply(
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<U256> {
        let token_dex = self
            .clone()
            .token_dex
            .expect("is_liquidity_locked: no token dex found");
        match token_dex.dex {
            Dex::UniswapV2 => get_total_liquidity_token_supply_uniswap_v2(&token_dex, client).await,
            Dex::UniswapV3 => get_total_liquidity_token_supply_uniswap_v3(&token_dex, client).await,

            _ => Ok(U256::zero()),
        }
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
}

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
pub async fn get_total_liquidity_token_supply_uniswap_v2(
    token_dex: &TokenDexData,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<U256> {
    // Connect to the Uniswap pair contract at the stored pair address.
    let pool = UNISWAP_PAIR::new(token_dex.pair_address, client.clone());

    // Query the total supply of liquidity tokens.
    let supply = pool.total_supply().call().await?;

    Ok(supply)
}

/// Returns the total number of liquidity pool (LP) tokens supplied on Uniswap v3.
///
/// # Arguments
///
/// * `client` - A reference-counted provider connected via WebSockets to an Ethereum node.
///
/// # Returns
///
/// * `Ok(U256)` containing the total supply of LP tokens if successful.
/// * `Err(anyhow::Error)` if an error occurs during the contract call.
pub async fn get_total_liquidity_token_supply_uniswap_v3(
    token_dex: &TokenDexData,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<U256> {
    // Connect to the Uniswap pair contract at the stored pair address.
    let pool = UNISWAP_V3_POOL::new(token_dex.pair_address, client.clone());

    // TODO - EDGE CASE - liquidity() is not same as number of liquidity tokens! resolve this
    // issue.
    // Query the supply of liquidity
    let supply = pool.liquidity().call().await?;

    Ok(U256::from(supply))
}
