use chainshield_backend::{
    dex::thegraph::{
        shared::SubGraph, uniswap_v2::get_top_uniswap_v2_pair_for_token,
        uniswap_v3::get_top_uniswap_v3_pool_for_token,
    },
    utils::logging::setup_logger,
};
use dotenv::dotenv;
use ethers::abi::Address;

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
