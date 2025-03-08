use anyhow::anyhow;
use ethers::providers::{Provider, Ws};
use ethers::types::Chain;
use log::error;
use std::{collections::HashMap, sync::Arc};

use crate::app_config::CHAINS;

use super::chain_data::CHAIN_DATA;

use tokio::sync::OnceCell; // Use OnceCell from tokio::sync
                           //
static PROVIDERS_HASH: OnceCell<Arc<HashMap<Chain, Provider<Ws>>>> = OnceCell::const_new();
pub async fn get_chain_provider(chain: &Chain) -> anyhow::Result<Arc<Provider<Ws>>> {
    // Check if providers are already initialized
    if PROVIDERS_HASH.get().is_none() {
        // Initialize if not already done
        let mut provider_hash = HashMap::<Chain, Provider<Ws>>::new();
        for chain in CHAINS {
            let ws_url = CHAIN_DATA.get_address(&chain).ws_url.clone();
            match Provider::<Ws>::connect(ws_url).await {
                Ok(provider) => {
                    provider_hash.insert(chain, provider);
                }
                Err(e) => {
                    error!("Failed to connect to chain {:?}: {}", chain, e);
                }
            }
        }

        // Try to set, ignoring if someone else set it first
        let _ = PROVIDERS_HASH.set(Arc::new(provider_hash));
    }

    // Now get the provider for the requested chain
    match PROVIDERS_HASH.get().unwrap().get(chain) {
        Some(provider) => Ok(Arc::new(provider.clone())),
        None => Err(anyhow!("could not retrieve provider")),
    }
}
