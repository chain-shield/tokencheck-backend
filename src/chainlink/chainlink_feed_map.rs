use crate::app_config::CHAINS;

use super::chainlink_data::{get_chainlink_price_feeds_by_chain, ChainlinkPriceFeed};
use anyhow::anyhow;
use ethers::types::Chain;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static CHAINLINK_FEED_MAP: Lazy<HashMap<Chain, HashMap<String, ChainlinkPriceFeed>>> =
    Lazy::new(|| {
        let mut price_feed_hash = HashMap::new();

        let chainlink_price_feed_hashmap = get_chainlink_price_feeds_by_chain();

        for chain in CHAINS {
            let mut price_feed_for_this_chain = HashMap::<String, ChainlinkPriceFeed>::new();

            let chainlink_price_feeds = chainlink_price_feed_hashmap.get(&chain).unwrap();
            // create hashmap with token symbol as index
            for feed in chainlink_price_feeds.iter() {
                if feed.base_currency == "USD" {
                    price_feed_for_this_chain
                        .insert(feed.token_symbol.to_lowercase() + "/USD", *feed);
                }
            }

            price_feed_hash.insert(chain, price_feed_for_this_chain);
        }
        price_feed_hash
    });

pub async fn get_chainlink_price_feed_for_token_(
    token_symbol: &str,
    chain: &Chain,
) -> anyhow::Result<String> {
    let usd_feed_symbol = token_symbol.to_lowercase() + "/USD";

    let feed_map = CHAINLINK_FEED_MAP
        .get(chain)
        .expect("chainlink feed for this chain not available");

    if feed_map.contains_key(&usd_feed_symbol) {
        match feed_map.get(&usd_feed_symbol) {
            Some(price_feed) => Ok(price_feed.address.to_string()),
            None => Err(anyhow!(
                "chainlink price feed for {} not found",
                usd_feed_symbol
            )),
        }
    } else {
        Err(anyhow!(
            "chainlink price feed for {} not found",
            usd_feed_symbol
        ))
    }
}
