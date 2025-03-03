use ethers::providers::{Provider, Ws};

use std::sync::Arc;

use crate::abi::{erc20::ERC20, uniswap_pair::UNISWAP_PAIR};

use super::shared::TokenDexData;

pub async fn get_liquidity_uniswap_v3(
    dex_data: &TokenDexData,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<u128> {
    // Connect to the Uniswap pair contract at the stored pair address.
    let base_token_address = if dex_data.is_token_0 {
        dex_data.token_1
    } else {
        dex_data.token_0
    };

    let base_token_contract = ERC20::new(base_token_address, client.clone());

    let liquidity = base_token_contract
        .balance_of(dex_data.pair_address)
        .call()
        .await?;

    Ok(liquidity.as_u128())
}

pub async fn get_liquidity_uniswap_v2(
    dex_data: &TokenDexData,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<u128> {
    // Connect to the Uniswap pair contract.
    let pool = UNISWAP_PAIR::new(dex_data.pair_address, client.clone());

    // Retrieve the reserves data.
    let (reserve0, reserve1, _) = pool.get_reserves().call().await?;

    // Determine the ETH-based liquidity depending on token arrangement.
    let liquidity = if dex_data.is_token_0 {
        reserve1
    } else {
        reserve0
    };

    Ok(liquidity)
}
