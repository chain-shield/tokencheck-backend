use anyhow::Result;
use chainshield_backend::abi::erc20::ERC20;
use chainshield_backend::app_config::AI_MODEL;
use chainshield_backend::data::chain_data::CHAIN_DATA;
use chainshield_backend::data::dex::TokenDex;
use chainshield_backend::data::token_data::{get_token_uniswap_v2_pair_address, ERC20Token};
use chainshield_backend::token_check::token_checklist::generate_token_checklist;
use chainshield_backend::token_check::token_score::{
    get_token_score_with_ai, get_token_score_with_rules_based_approch,
};
use dotenv::dotenv;
use ethers::providers::{Provider, Ws};
use ethers::types::{Address, Chain};
use std::sync::Arc;

// mainnet
pub const WHITELIST_TOKENS_MAINNET: [&str; 3] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    // "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

// base ?
pub const SCAMLIST_TOKENS_BASE: [&str; 4] = [
    "0xaff019720963fb45e13b745abfa10b946de8f4c9",
    "0x9a301ad1ae2ba1ecf8693a60de92e834f4429e8c",
    "0x7ea18f3dff39b4cede0d8b16fe05852e85024146",
    "0x8f806505a0677da5f9c4e8aff5bc9237b6cd154f",
];

pub struct SetupData {
    client: Arc<Provider<Ws>>,
    pub token: ERC20Token,
}

#[tokio::test]
#[ignore]
async fn test_generate_checklist_base() -> anyhow::Result<()> {
    const SCAM: &str = "0x9a301ad1ae2ba1ecf8693a60de92e834f4429e8c";
    const VIRTUALS: &str = "0x0b3e328455c4059EEb9e3f84b5543F74E24e7E1b";
    let data = setup(SCAM, &Chain::Base).await?;

    let token_checklist = generate_token_checklist(&data.token, &data.client).await?;

    println!("token checklist => {:#?}", token_checklist);

    let token_score = get_token_score_with_rules_based_approch(token_checklist);

    println!("token score => {:#?}", token_score);

    Ok(())
}

#[tokio::test]
async fn test_generate_checklist_mainnet() -> anyhow::Result<()> {
    for token in WHITELIST_TOKENS_MAINNET {
        let data = setup(token, &Chain::Mainnet).await?;

        let token_checklist = generate_token_checklist(&data.token, &data.client).await?;

        println!("token checklist => {:#?}", token_checklist);

        let token_score = get_token_score_with_rules_based_approch(token_checklist.clone());

        println!("token score (rule based) => {:#?}", token_score);

        let token_score_ai = get_token_score_with_ai(token_checklist, &AI_MODEL).await?;
        println!("token score (ai) => {:#?}", token_score_ai);
    }

    Ok(())
}

/// get ERC20Token - struct that contains all data we need - from token address
pub async fn setup(token_address: &str, chain: &Chain) -> Result<SetupData> {
    dotenv().ok();
    let ws_url = CHAIN_DATA.get_address(&Chain::Mainnet).ws_url.clone();
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
        get_token_uniswap_v2_pair_address(token_address_h160, chain, &client).await?;

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
