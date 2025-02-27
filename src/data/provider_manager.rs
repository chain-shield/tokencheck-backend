use anyhow::anyhow;
use ethers::providers::{Provider, Ws};
use ethers::types::Chain;
use log::error;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use tokio::runtime::Runtime;

use crate::app_config::CHAINS;

use super::chain_data::CHAIN_DATA;

pub static PROVIDERS_HASH: Lazy<Arc<HashMap<Chain, Provider<Ws>>>> = Lazy::new(|| {
    let runtime = Runtime::new().unwrap();

    runtime.block_on(async {
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
        Arc::new(provider_hash)
    })
});

pub async fn get_chain_provider(chain: &Chain) -> anyhow::Result<Arc<Provider<Ws>>> {
    match PROVIDERS_HASH.get(chain) {
        Some(provider) => Ok(Arc::new(provider.clone())),
        None => Err(anyhow!("could not retreive provider")),
    }
}
