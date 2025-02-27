use ethers::abi::Address;
use ethers::providers::{Provider, Ws};
use ethers::types::U256;
use std::sync::Arc;

use crate::abi::erc20::ERC20;
use crate::abi::uniswap_pair::UNISWAP_PAIR;
use crate::abi::uniswap_pool::UNISWAP_V3_POOL;
use crate::data::chain_data::CHAIN_DATA;
use crate::data::dex::Dex;
use crate::data::token_data::ERC20Token;

impl ERC20Token {
    pub async fn get_total_liquidity_token_supply(
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<U256> {
        match self.token_dex.dex {
            Dex::UniswapV2 => {
                self.get_total_liquidity_token_supply_uniswap_v2(client)
                    .await
            }
            Dex::UniswapV3 => {
                self.get_total_liquidity_token_supply_uniswap_v3(client)
                    .await
            }

            _ => Ok(U256::zero()),
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
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<U256> {
        // Connect to the Uniswap pair contract at the stored pair address.
        let pool = UNISWAP_PAIR::new(self.token_dex.pair_or_pool_address, client.clone());

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
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<U256> {
        // Connect to the Uniswap pair contract at the stored pair address.
        let pool = UNISWAP_V3_POOL::new(self.token_dex.pair_or_pool_address, client.clone());

        // TODO - EDGE CASE - liquidity() is not same as number of liquidity tokens! resolve this
        // issue.
        // Query the supply of liquidity
        let supply = pool.liquidity().call().await?;

        Ok(U256::from(supply))
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

    pub async fn get_liquidity(&self, client: &Arc<Provider<Ws>>) -> anyhow::Result<u128> {
        match self.token_dex.dex {
            Dex::UniswapV3 => self.get_liquidity_uniswap_v3(client).await,
            Dex::UniswapV2 => self.get_liquidity_uniswap_v2(client).await,
            _ => Ok(0),
        }
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
    pub async fn get_liquidity_uniswap_v2(
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<u128> {
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

    /// Retrieves the liquidity value from the pool (Uniswap v3)
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
    pub async fn get_liquidity_uniswap_v3(
        &self,
        client: &Arc<Provider<Ws>>,
    ) -> anyhow::Result<u128> {
        // Connect to the Uniswap pair contract at the stored pair address.
        let weth_address: Address = CHAIN_DATA.get_address(&self.chain).weth.parse()?;
        let weth_contract = ERC20::new(weth_address, client.clone());

        let eth_liquidity = weth_contract
            .balance_of(self.token_dex.pair_or_pool_address)
            .call()
            .await?;

        Ok(eth_liquidity.as_u128())
    }
}
