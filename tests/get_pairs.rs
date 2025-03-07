use chainshield_backend::{
    dex::{
        dex_data::find_top_dex_for_token,
        thegraph::{
            shared::SubGraph, uniswap_v2::get_top_uniswap_v2_pair_for_token,
            uniswap_v3::get_top_uniswap_v3_pool_for_token,
        },
    },
    utils::logging::setup_logger,
};
use dotenv::dotenv;
use ethers::{abi::Address, types::Chain};

/// Whitelist tokens for mainnet testing.
pub const WHITELIST_TOKENS_MAINNET: [&str; 4] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

#[tokio::test]
async fn test_top_pool() -> anyhow::Result<()> {
    dotenv().ok();
    // if let Err(_) = setup_logger() {
    //     println!("logger already initialize!");
    // };

    for token in WHITELIST_TOKENS_MAINNET {
        let token_address: Address = token.parse()?;
        let pool = find_top_dex_for_token(token_address, &Chain::Mainnet).await?;
        let pool = pool.unwrap();
        println!(
            "{:?} Pool: {} <> {} (Fee: {})",
            pool.dex,
            format!("{:?}", pool.token_0),
            format!("{:?}", pool.token_1),
            pool.fee
        );
        println!("  pool address: {}", format!("{:?}", pool.pair_address));
        println!("  TVL: ${}", pool.liquidity_in_usd);
        println!("  Created at block: {}", pool.create_at_block_number);
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_link_uniswap_v3_top_pool() -> anyhow::Result<()> {
    dotenv().ok();
    if let Err(_) = setup_logger() {
        println!("logger already initialize!");
    };

    // Replace with your token address
    let token_address = "0x514910771af9ca656af840dff83e8264ecf986ca"; // LINK token
    let token_address: Address = token_address.parse()?;

    // Option 1: Get pools where the token is either token0 or token1 (two separate queries)
    let pool = get_top_uniswap_v3_pool_for_token(token_address, SubGraph::UniswapV3Mainnet)
        .await?
        .unwrap();

    // Option 2: Get pools where the token is either token0 or token1 (single query with aliases)
    // let pools = get_all_uniswap_v3_pools_single_query(token_address, 10).await?;

    println!(
        "UNISWAP V3 Pool: {} <> {} (Fee: {})",
        format!("{:?}", pool.token_0),
        format!("{:?}", pool.token_1),
        pool.fee
    );
    println!("  pool address: {}", format!("{:?}", pool.pair_address));
    println!("  TVL: ${}", pool.liquidity_in_usd);
    println!("  Created at block: {}", pool.create_at_block_number);

    // Replace with your token address
    let token_base = "0x0b3e328455c4059eeb9e3f84b5543f74e24e7e1b";
    let token_base: Address = token_base.parse()?;

    // Option 1: Get pools where the token is either token0 or token1 (two separate queries)
    let pool = get_top_uniswap_v3_pool_for_token(token_base, SubGraph::UniswapV3Base)
        .await?
        .unwrap();

    println!(
        "UNISWAP V3 Pool (BASE): {} <> {} (Fee: {})",
        format!("{:?}", pool.token_0),
        format!("{:?}", pool.token_1),
        pool.fee
    );
    println!("  pool address: {}", format!("{:?}", pool.pair_address));
    println!("  TVL: ${}", pool.liquidity_in_usd);
    println!("  Created at block: {}", pool.create_at_block_number);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_link_uniswap_v2_top_pool() -> anyhow::Result<()> {
    dotenv().ok();
    if let Err(_) = setup_logger() {
        println!("logger already initialize!");
    };

    // Replace with your token address
    let token_mainnet = "0x514910771af9ca656af840dff83e8264ecf986ca"; // LINK token
    let token_mainnet: Address = token_mainnet.parse()?;

    // Option 1: Get pools where the token is either token0 or token1 (two separate queries)
    let pool = get_top_uniswap_v2_pair_for_token(token_mainnet, SubGraph::UniswapV2Mainnet)
        .await?
        .unwrap();

    // Option 2: Get pools where the token is either token0 or token1 (single query with aliases)
    // let pools = get_all_uniswap_v3_pools_single_query(token_address, 10).await?;

    println!(
        "UNISWAP V2 Pool: {} <> {} (Fee: {})",
        format!("{:?}", pool.token_0),
        format!("{:?}", pool.token_1),
        pool.fee
    );
    println!("  pool address: {}", format!("{:?}", pool.pair_address));
    println!("  TVL: ${}", pool.liquidity_in_usd);
    println!("  Created at block: {}", pool.create_at_block_number);

    // Replace with your token address
    let token_base = "0x0b3e328455c4059eeb9e3f84b5543f74e24e7e1b";
    let token_base: Address = token_base.parse()?;

    // Option 1: Get pools where the token is either token0 or token1 (two separate queries)
    let pool = get_top_uniswap_v2_pair_for_token(token_base, SubGraph::UniswapV2Base)
        .await?
        .unwrap();

    println!(
        "UNISWAP V2 Pool (BASE): {} <> {} (Fee: {})",
        format!("{:?}", pool.token_0),
        format!("{:?}", pool.token_1),
        pool.fee
    );
    println!("  pool address: {}", format!("{:?}", pool.pair_address));
    println!("  TVL: ${}", pool.liquidity_in_usd);
    println!("  Created at block: {}", pool.create_at_block_number);

    Ok(())
}
