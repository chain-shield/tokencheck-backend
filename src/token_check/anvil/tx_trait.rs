use crate::abi::erc20::ERC20;
use crate::data::token_data::ERC20Token;
use crate::token_check::anvil::simlator::AnvilTestSimulator;
use crate::utils::tx::{calculate_next_block_base_fee, get_current_block};
use async_trait::async_trait;
use ethers::types::{Address, U256};
use ethers::utils::format_units;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Ws},
    signers::Wallet,
};
use std::sync::Arc;

/// The `Txs` trait defines an interface for interacting with Ethereum-based
/// transactions. It provides methods to obtain wallet clients, retrieve token
/// and Ether balances, as well as calculate gas fees and timestamps.
///
/// # Examples
///
/// ```rust
/// // Implement the Txs trait for your custom transaction simulator.
/// struct MySimulator { /* fields omitted */ }
///
/// #[async_trait]
/// impl Txs for MySimulator {
///     // Provide concrete implementations for the required methods.
/// }
///
/// // Retrieve current timestamp:
/// async fn check_timestamp(tx: &impl Txs) -> anyhow::Result<u64> {
///     tx.get_current_timestamp().await
/// }
/// ```
#[async_trait]
pub trait Txs {
    /// Returns a clone of the signer middleware client.
    fn signed_client(&self) -> Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>;

    /// Returns a clone of the provider client.
    fn client(&self) -> Arc<Provider<Ws>>;

    /// Returns the wallet's sender address.
    fn sender(&self) -> Address;

    /// Asynchronously retrieves the current blockchain timestamp.
    ///
    /// This method gets the current block number and then fetches the block
    /// details to extract the timestamp. The timestamp may be used as a deadline
    /// for certain transactions.
    ///
    /// # Errors
    ///
    /// Returns an error if the block number or block details cannot be retrieved.
    ///
    /// # Returns
    ///
    /// A result containing the current timestamp as a `u64` or an error.
    async fn get_current_timestamp(&self) -> anyhow::Result<u64> {
        // Get the current block number from the signer client.
        let current_block = self.signed_client().get_block_number().await?;
        // Get detailed block information to retrieve the timestamp.
        let current_block_details = self.signed_client().get_block(current_block).await?;
        let current_timestamp = current_block_details
            .ok_or_else(|| anyhow::anyhow!("No current block details"))?
            .timestamp
            .as_u64();

        Ok(current_timestamp)
    }

    /// Asynchronously retrieves and prints the wallet's token balance for the specified ERC20 token.
    ///
    /// This method first obtains the token balance by calling `get_wallet_token_balance_by_address`
    /// and then formats the balance for human-readable output.
    ///
    /// # Arguments
    ///
    /// * `token` - The ERC20 token data, including address, name, symbol, and decimals.
    ///
    /// # Returns
    ///
    /// A result containing the token balance as a `U256` or an error.
    async fn get_wallet_token_balance(&self, token: &ERC20Token) -> anyhow::Result<U256> {
        let new_token_balance_u256 = self
            .get_wallet_token_balance_by_address(token.address)
            .await?;
        let token_balance = format_units(new_token_balance_u256, u32::from(token.decimals))?;
        println!(
            "YOU HAVE {} of {}, ({})",
            token_balance, token.name, token.symbol
        );
        Ok(new_token_balance_u256)
    }

    /// Asynchronously retrieves the wallet's token balance using the token's address.
    ///
    /// This method interacts with the ERC20 contract to query the balance for the provided address.
    ///
    /// # Arguments
    ///
    /// * `token_address` - The Ethereum address of the token contract.
    ///
    /// # Returns
    ///
    /// A result containing the token balance as a `U256` or an error.
    async fn get_wallet_token_balance_by_address(
        &self,
        token_address: Address,
    ) -> anyhow::Result<U256> {
        // Log the start of the token balance retrieval.
        println!("getting token balance");
        let token_contract = ERC20::new(token_address, self.signed_client());

        // Call the ERC20 contract's balance_of method for the sender.
        let new_token_balance_u256 = token_contract.balance_of(self.sender()).call().await?;
        println!("token balance = {}", new_token_balance_u256);

        Ok(new_token_balance_u256)
    }

    /// Asynchronously retrieves the wallet's Ether balance.
    ///
    /// This method fetches the current Ether balance from the provider client.
    ///
    /// # Returns
    ///
    /// A result containing the Ether balance as a `U256` or an error.
    async fn get_wallet_eth_balance(&self) -> anyhow::Result<U256> {
        // Retrieve the ETH balance for the sender's address.
        let new_eth_balance_u256 = self
            .signed_client()
            .get_balance(self.sender(), None)
            .await?;
        // The following line (commented out) can be used to format the balance for display.
        // let eth_balance = format_units(new_eth_balance_u256, 18u32)?;
        // println!("YOU HAVE {} of ETH", eth_balance);
        Ok(new_eth_balance_u256)
    }

    /// Asynchronously calculates and returns the adjusted gas fee and the recommended priority fee.
    ///
    /// This function fetches the current block, calculates the next block's base fee,
    /// applies a 5% buffer, and then derives a 10% suggested priority fee.
    ///
    /// # Returns
    ///
    /// A result containing a tuple where the first element is the adjusted max fee and the second is the priority fee, both as `U256`, or an error.
    async fn get_gas_and_priority_fee(&self) -> anyhow::Result<(U256, U256)> {
        // Retrieve the current block necessary for fee calculation.
        let (block, _) = get_current_block(&self.client()).await?;

        // Compute the next block's base fee.
        let next_base_fee = calculate_next_block_base_fee(&block)?;

        // Calculate a 5% fee buffer.
        let buffer = next_base_fee / 20; // 5% buffer
        let adjusted_max_fee = next_base_fee + buffer;
        // Determine a 10% suggested priority fee based on the adjusted fee.
        let prority_max_fee = adjusted_max_fee / 10; // 10% suggested priority fee
        Ok((adjusted_max_fee, prority_max_fee))
    }
}

/// Implements the `Txs` trait for the `AnvilTestSimulator`.
///
/// This allows the simulator to interact with Ethereum transactions by exposing
/// methods to retrieve clients, balances, and gas fee information.
impl Txs for AnvilTestSimulator {
    /// Returns a clone of the simulator's signer middleware client.
    fn signed_client(&self) -> Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>> {
        self.signed_client.clone()
    }

    /// Returns a clone of the simulator's provider client.
    fn client(&self) -> Arc<Provider<Ws>> {
        self.client.clone()
    }

    /// Returns the sender's address associated with the simulator.
    fn sender(&self) -> Address {
        self.sender
    }
}
