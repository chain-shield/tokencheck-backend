use anyhow::Result;
use chainshield_backend::{
    abi::erc20::ERC20,
    app_config::AI_MODEL,
    data::{
        contracts::CONTRACT,
        token_data::{get_token_uniswap_v2_pair_address, ERC20Token},
    },
    token_check::{
        token_checklist::generate_token_checklist,
        token_score::{get_token_score_with_ai, get_token_score_with_rules_based_approch},
    },
    utils::logging::setup_logger,
};
use dotenv::dotenv;
use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use std::sync::Arc;

// mainnet
pub const WHITELIST_TOKENS_MAINNET: [&str; 3] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    // "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

pub struct SetupData {
    client: Arc<Provider<Ws>>,
    pub token: ERC20Token,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    // THIS IS FOR TESTING PURPOSES - WILL BE REPLACE BY SERVER CODE
    for token in WHITELIST_TOKENS_MAINNET {
        let data = setup(token).await?;

        let token_checklist = generate_token_checklist(data.token, &data.client).await?;

        println!("token checklist => {:#?}", token_checklist);

        let token_score = get_token_score_with_rules_based_approch(token_checklist.clone());

        println!("token score (rule based) => {:#?}", token_score);

        let token_score_ai = get_token_score_with_ai(token_checklist, &AI_MODEL).await?;
        println!("token score (ai) => {:#?}", token_score_ai);
    }

    Ok(())
}

/// get ERC20Token - struct that contains all data we need - from token address
pub async fn setup(token_address: &str) -> Result<SetupData> {
    dotenv().ok();
    let ws_url = CONTRACT.get_address().ws_url.clone();
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
        pair_address,
        is_token_0,
        ..Default::default()
    };

    Ok(SetupData { client, token })
}
