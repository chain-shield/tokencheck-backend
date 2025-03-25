use anyhow::anyhow;
use ethers::providers::{Provider, Ws};
use ethers::types::Chain;
use log::error;
use std::{collections::HashMap, sync::Arc};

use crate::app_config::{AppMode, APP_MODE, CHAINS};

use super::chain_data::CHAIN_DATA;

use tokio::sync::OnceCell;

/// A global cache of WebSocket providers for different blockchain networks.
/// This is lazily initialized the first time a provider is requested.
static PROVIDERS_HASH: OnceCell<Arc<HashMap<Chain, Provider<Ws>>>> = OnceCell::const_new();

/// Retrieves a WebSocket provider for the specified blockchain network.
///
/// This function ensures that providers are initialized only once and then cached.
/// Subsequent calls will return the cached provider for the requested chain.
///
/// # Arguments
///
/// * `chain` - The blockchain network to get a provider for
///
/// # Returns
///
/// * `anyhow::Result<Arc<Provider<Ws>>>` - A thread-safe reference to the provider or an error
///
/// # Errors
///
/// Returns an error if:
/// - The provider for the requested chain couldn't be initialized
/// - The requested chain is not supported
pub async fn get_chain_provider(chain: &Chain) -> anyhow::Result<Arc<Provider<Ws>>> {
    // Check if providers are already initialized
    if PROVIDERS_HASH.get().is_none() {
        // Initialize if not already done
        let mut provider_hash = HashMap::<Chain, Provider<Ws>>::new();
        for chain in CHAINS {
            let ws_url = if APP_MODE == AppMode::Production {
                CHAIN_DATA.get_address(&chain).alchemy_url.clone()
            } else {
                CHAIN_DATA.get_address(&chain).ws_url.clone()
            };

            match Provider::<Ws>::connect(ws_url).await {
                Ok(provider) => {
                    provider_hash.insert(chain, provider);
                }
                Err(e) => {
                    error!("Failed to connect to chain {:?}: {}", chain, e);
                    // Note: We continue even if a provider fails to connect
                    // This allows the application to work with other chains
                }
            }
        }

        // Try to set, ignoring if someone else set it first
        let _ = PROVIDERS_HASH.set(Arc::new(provider_hash));
    }

    // Now get the provider for the requested chain
    match PROVIDERS_HASH
        .get()
        .ok_or_else(|| anyhow!("Provider initialization failed"))?
        .get(chain)
    {
        Some(provider) => Ok(Arc::new(provider.clone())),
        None => Err(anyhow!("Provider not available for the requested chain")),
    }
}
