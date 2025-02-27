//! This module provides the `AnvilTestSimulator`, a test harness for interacting with a forked Anvil
//! blockchain. It sets up an Anvil instance with forking based on a provided Geth RPC URL, creates wallets,
//! and initializes signed clients that can interact with the Anvil provider.
//!
//! The simulator is useful for testing transaction flows and blockchain interactions in an isolated
//! and controllable environment.

use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Ws},
    signers::{Signer, Wallet},
    types::{Address, Chain},
    utils::{Anvil, AnvilInstance},
};
use std::sync::Arc;

use crate::utils::tx::{get_test_wallet, get_wallet};

/// The starting balance for the simulated account.
pub const STARTING_BALANCE: f64 = 1000.0;

/// Simulator for testing against an Anvil forked blockchain.
///
/// This simulator initializes an Anvil instance with a fork of the specified RPC URL, sets up wallets,
/// and connects them to the blockchain provider. It provides two signed clients for sending signed transactions,
/// along with an unsigned client for general provider operations.
pub struct AnvilTestSimulator {
    /// The primary client used for sending transactions, wrapped in a signer middleware.
    pub signed_client: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    /// The secondary client used for testing transactions, wrapped in a signer middleware.
    pub second_signed_client: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    /// The raw provider connected to the Anvil instance.
    pub client: Arc<Provider<Ws>>,
    /// The running Anvil instance.
    pub anvil: AnvilInstance,
    /// The address of the primary sender.
    pub sender: Address,
}

impl AnvilTestSimulator {
    /// Creates a new instance of `AnvilTestSimulator` with a forked blockchain using Anvil.
    ///
    /// The simulator connects to an existing Geth node via the provided RPC URL in order to create a forked
    /// blockchain environment. It then sets up the Anvil instance, creates two wallets, and initializes corresponding
    /// signed clients.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - A string slice that holds the URL of the Geth node to fork from.
    ///
    /// # Returns
    ///
    /// * `Ok(AnvilTestSimulator)` if the simulator was successfully created, otherwise an error is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use token_check::anvil::simlator::AnvilTestSimulator;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rpc_url = "https://your-geth-node-url";
    ///     let simulator = AnvilTestSimulator::new(rpc_url).await.unwrap();
    ///     // Use simulator for sending transactions or other operations.
    /// }
    /// ```
    pub async fn new(rpc_url: &str, chain: &Chain) -> Result<Self> {
        // Configure and spawn a new Anvil instance, forking from the provided Geth node URL.
        println!("creating forked anvil note");
        let anvil = Anvil::new()
            // Optionally add extra arguments or configuration options as needed.
            // .args(["--no-storage-caching", "--code-size-limit", "2048"])
            .fork(rpc_url) // Fork from the Geth node.
            .chain_id(*chain as u64) // Set the chain ID for the forked network.
            .args(["--no-storage-caching"]) // Additional arguments to Anvil.
            .spawn();

        // Optionally, setup a mock sender if required (this section is commented out as an example).
        // let from_address: Address = anvil.addresses()[0];
        // let private_keys = anvil.keys();
        // let from_private_key = private_keys[0].clone();

        // Connect to the Anvil WebSocket endpoint.
        println!("connecting anvil");
        let anvil_ws_url = anvil.ws_endpoint();
        let provider = Provider::<Ws>::connect(anvil_ws_url).await?;
        let client = Arc::new(provider.clone());

        // Retrieve primary and test wallets.
        println!("getting wallets setup");
        let wallet = get_wallet(chain)?; // Primary wallet.
        let second_wallet = get_test_wallet(chain)?; // Secondary/test wallet.
        let from_address = wallet.address();

        // Create signer middleware clients for both wallets.
        println!("signing clients");
        let signed_client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
        let second_signed_client = Arc::new(SignerMiddleware::new(provider, second_wallet));

        // Set the balance for the primary wallet using the Anvil RPC method.
        // The balance is hardcoded to "0x3635c9adc5dea00000", which corresponds to 100 ETH.
        signed_client
            .provider()
            .request::<_, ()>(
                "anvil_setBalance",
                [
                    format!("{:#x}", from_address),
                    "0x3635c9adc5dea00000".to_string(), // 100 ETH
                ],
            )
            .await?;

        // Construct and return the simulator instance.
        let simulator = Self {
            signed_client,
            second_signed_client,
            client,
            anvil,
            sender: from_address,
        };

        Ok(simulator)
    }
}
