use dotenv::dotenv;
use ethers::types::Address;
use ethers::{
    providers::{Provider, Ws},
    types::Chain,
};
use std::{sync::Arc, time::Duration};
use tokencheck_backend::{
    abi::erc20::ERC20,
    app_config::AI_MODEL,
    data::chain_data::CHAIN_DATA,
    token_check::{
        ai::ai_submission::check_code_with_ai,
        external_api::etherscan_api::{get_source_code, get_token_info},
    },
    utils::web_scrapper::scrape_site_and_get_text,
};
use tokio::time::sleep;

pub const WHITELIST_TOKENS: [&str; 4] = [
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

// TEST ON BASE
#[tokio::test]
#[ignore]
async fn test_audit_token_contract() -> anyhow::Result<()> {
    dotenv().ok();
    const VIRTUALS: &str = "0x0000000000000000000000000000000000000000";
    let source_code = get_source_code(VIRTUALS, &Chain::Base).await?;

    let audit = check_code_with_ai(source_code, &AI_MODEL).await?.unwrap();
    println!("{:#?}", audit);

    // assert!(!source_code.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_scrapper() -> anyhow::Result<()> {
    dotenv().ok();
    let ws_url = CHAIN_DATA.get_address(&Chain::Mainnet).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    for token in WHITELIST_TOKENS {
        let token_address: Address = token.parse()?;
        let contract = ERC20::new(token_address, client.clone());
        let name = contract.name().call().await?;

        sleep(Duration::from_secs(1)).await;
        let token_info = get_token_info(token, &Chain::Mainnet).await?.unwrap();
        println!("website content for {} ....", name);
        let website_text = scrape_site_and_get_text(&token_info.website).await?;
        assert!(!website_text.is_empty());
    }

    Ok(())
}
