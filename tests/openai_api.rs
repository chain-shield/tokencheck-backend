use anyhow::Result;
use chainshield_backend::app_config::CHAIN;
use chainshield_backend::data::chain_data::CHAIN_DATA;
use chainshield_backend::data::token_data::{
    get_token_uniswap_v2_pair_address, ERC20Token, TokenDex,
};
use chainshield_backend::token_check::external_api::moralis;
use chainshield_backend::token_check::token_holder_check::get_token_holder_check;
use chainshield_backend::utils::logging::setup_logger;
use chainshield_backend::{
    abi::erc20::ERC20,
    app_config::AI_MODEL,
    token_check::{
        ai::ai_submission::check_code_with_ai,
        external_api::etherscan_api::{get_contract_owner, get_source_code, get_token_info},
    },
};
use dotenv::dotenv;
use ethers::providers::{Provider, Ws};
use ethers::types::Address;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub const WHITELIST_TOKENS: [&str; 4] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

pub const SCAMLIST_TOKENS: [&str; 4] = [
    "0xaff019720963fb45e13b745abfa10b946de8f4c9",
    "0x9a301ad1ae2ba1ecf8693a60de92e834f4429e8c",
    "0x7ea18f3dff39b4cede0d8b16fe05852e85024146",
    "0x8f806505a0677da5f9c4e8aff5bc9237b6cd154f",
];

pub struct SetupData {
    client: Arc<Provider<Ws>>,
    token: ERC20Token,
}

// TEST ON BASE
#[tokio::test]
#[ignore]
async fn test_audit_token_contract() -> anyhow::Result<()> {
    dotenv().ok();
    const VIRTUALS: &str = "0x0b3e328455c4059EEb9e3f84b5543F74E24e7E1b";
    let source_code = get_source_code(VIRTUALS).await?;

    let audit = check_code_with_ai(source_code, &AI_MODEL).await?.unwrap();
    println!("{:#?}", audit);

    // assert!(!source_code.is_empty());

    Ok(())
}

// TEST ON BASE
#[tokio::test]
#[ignore]
async fn test_contract_creation() -> anyhow::Result<()> {
    dotenv().ok();
    const VIRTUALS: &str = "0x0b3e328455c4059EEb9e3f84b5543F74E24e7E1b";
    let token_owner = get_contract_owner(VIRTUALS).await?;

    match token_owner {
        Some(data) => println!("{:#?}", data),
        None => println!("Opps..could not unwrap!"),
    }

    // assert!(!source_code.is_empty());

    Ok(())
}

// TEST ON BASE
#[tokio::test]
async fn get_holder_analysis() -> anyhow::Result<()> {
    dotenv().ok();

    const VIRTUALS: &str = "0x0b3e328455c4059EEb9e3f84b5543F74E24e7E1b";

    let info = setup(VIRTUALS).await?;

    // let token_owner = get_contract_owner(VIRTUALS).await?;
    //
    // let owner = match token_owner {
    //     Some(data) => data,
    //     None => panic!("Opps..could not unwrap!"),
    // };

    let token_holder_analysis = get_token_holder_check(&info.token, &info.client)
        .await?
        .unwrap();

    println!("analysis => {:#?}", token_holder_analysis);

    Ok(())
}

// TEST ON BASE
#[tokio::test]
#[ignore]
async fn test_audit_token_contract_2() -> anyhow::Result<()> {
    dotenv().ok();
    const SCAM_TOKEN: &str = "0x1f035d740FD128E3818a08D613bC4C2D8f8Fccee";
    let source_code = get_source_code(SCAM_TOKEN).await?;

    let audit = check_code_with_ai(source_code, &AI_MODEL).await?.unwrap();

    println!("AUDIT => {:#?}", audit);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_whitelist_contracts() -> anyhow::Result<()> {
    dotenv().ok();

    let ws_url = CHAIN_DATA.get_address(CHAIN).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    for token in WHITELIST_TOKENS {
        let token_address: Address = token.parse()?;
        let contract = ERC20::new(token_address, client.clone());
        let name = contract.name().call().await?;
        let source_code = get_source_code(token).await?;

        match check_code_with_ai(source_code, &AI_MODEL).await? {
            Some(audit) => println!("{} AUDIT => {:#?}", name, audit),
            None => println!("Opps..something went wrong!"),
        };
    }
    // assert!(!source_code.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_scamlist_contracts() -> anyhow::Result<()> {
    dotenv().ok();

    let ws_url = CHAIN_DATA.get_address(CHAIN).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    for token in SCAMLIST_TOKENS {
        let token_address: Address = token.parse()?;
        let contract = ERC20::new(token_address, client.clone());
        let name = contract.name().call().await?;
        let source_code = get_source_code(token).await?;

        match check_code_with_ai(source_code, &AI_MODEL).await? {
            Some(audit) => println!("{} AUDIT => {:#?}", name, audit),
            None => println!("Opps..something went wrong!"),
        };
    }
    // assert!(!source_code.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_whitelist_get_info() -> anyhow::Result<()> {
    dotenv().ok();
    let ws_url = CHAIN_DATA.get_address(CHAIN).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    for token in WHITELIST_TOKENS {
        let token_address: Address = token.parse()?;
        let contract = ERC20::new(token_address, client.clone());
        let name = contract.name().call().await?;

        sleep(Duration::from_secs(1)).await;
        let token_info = get_token_info(token).await?;

        match token_info {
            Some(info) => {
                println!("INFO FOR {}, => {:#?}", name, info);
                assert!(!info.website.is_empty() || !info.twitter.is_empty());
            }
            None => println!("no token info avaliable!"),
        }
    }
    // assert!(!source_code.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_whitelist_get_info_using_moralis() -> anyhow::Result<()> {
    dotenv().ok();
    let ws_url = CHAIN_DATA.get_address(CHAIN).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    for token in WHITELIST_TOKENS {
        let token_address: Address = token.parse()?;
        let contract = ERC20::new(token_address, client.clone());
        let name = contract.name().call().await?;

        let token_info = moralis::get_token_info(token).await?;

        match token_info {
            Some(info) => {
                println!("INFO FOR {}, => {:#?}", name, info);
                assert!(!info.website.is_empty() || !info.twitter.is_empty());
            }
            None => println!("no token info avaliable!"),
        }
    }
    // assert!(!source_code.is_empty());

    Ok(())
}

/// get ERC20Token - struct that contains all data we need - from token address
pub async fn setup(token_address: &str) -> Result<SetupData> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");
    let ws_url = CHAIN_DATA.get_address(CHAIN).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    let token_address_h160: Address = token_address.parse()?;
    let token_contract = ERC20::new(token_address_h160, client.clone());

    // get basic toke data
    let symbol = token_contract.symbol().call().await?;
    let decimals = token_contract.decimals().call().await?;
    let name = token_contract.name().call().await?;

    // get pair address of token, and is_token_0 , true if token is token_0, otherwise its token_1
    println!("get pair address..");
    let (pair_address, is_token_0) =
        get_token_uniswap_v2_pair_address(token_address_h160, &client).await?;

    let token = ERC20Token {
        name,
        symbol,
        decimals,
        address: token_address_h160,
        token_dex: TokenDex {
            pair_or_pool_address: pair_address,
            is_token_0,
            ..Default::default()
        },
        ..Default::default()
    };

    Ok(SetupData { client, token })
}
