use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Ws},
    signers::{Signer, Wallet},
    types::Address,
    utils::{Anvil, AnvilInstance},
};
use std::sync::Arc;

use crate::{
    app_config::CHAIN,
    utils::tx::{get_test_wallet, get_wallet},
};

pub const STARTING_BALANCE: f64 = 1000.0;

pub struct AnvilTestSimulator {
    pub signed_client: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    pub second_signed_client: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    pub client: Arc<Provider<Ws>>,
    pub anvil: AnvilInstance,
    pub sender: Address,
}

impl AnvilTestSimulator {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        // Main network provider   // Configure Anvil with forking
        println!("creating forked anvil note");
        let anvil = Anvil::new()
            // .args(["--no-storage-caching", "--code-size-limit", "2048"])
            .fork(rpc_url) // URL of your Geth node
            .chain_id(CHAIN)
            .args(["--no-storage-caching"])
            .spawn();

        // setup mock sender
        // let from_address: Address = anvil.addresses()[0];
        // let private_keys = anvil.keys();
        // let from_private_key = private_keys[0].clone();

        // Connect to Anvil
        println!("connecting anvil");
        let anvil_ws_url = anvil.ws_endpoint();
        let provider = Provider::<Ws>::connect(anvil_ws_url).await?;
        let client = Arc::new(provider.clone());

        // Create a wallet with the private key
        println!("getting wallets setup");
        let wallet = get_wallet()?;
        let second_wallet = get_test_wallet()?;
        let from_address = wallet.address();

        // Create the SignerMiddleware
        println!("signing clients");
        let signed_client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
        let second_signed_client = Arc::new(SignerMiddleware::new(provider, second_wallet));

        signed_client
            .provider()
            .request::<_, ()>(
                "anvil_setBalance",
                [
                    format!("{:#x}", from_address),
                    "0x3635c9adc5dea00000".to_string(), //100 ETH
                ],
            )
            .await?;

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
