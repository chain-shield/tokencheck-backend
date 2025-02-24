//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and performs token checking for tokens in the mainnet whitelist.
//! It demonstrates a test run where token data is fetched, token checklists are generated, and token scores are calculated using both rule-based and AI-based methods.

use anyhow::Result;
use chainshield_backend::{
    abi::erc20::ERC20,
    app_config::{AI_MODEL, CHAIN},
    data::{
        chain_data::CHAIN_DATA,
        token_data::{get_token_uniswap_v2_pair_address, ERC20Token, TokenDex},
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
use log::info;
use std::sync::Arc;

/// Whitelist tokens for mainnet testing.
pub const WHITELIST_TOKENS_MAINNET: [&str; 3] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    // "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

/// Structure to hold setup data including the blockchain provider client and token information.
pub struct SetupData {
    client: Arc<Provider<Ws>>,
    pub token: ERC20Token,
}

/// Main function that iterates over tokens, performs token checks, and prints token scores using both rule-based and AI-based approaches.
///
/// # Errors
/// Returns an error if any step in token processing or blockchain communication fails.
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    // THIS IS FOR TESTING PURPOSES - WILL BE REPLACED BY SERVER CODE
    for token in WHITELIST_TOKENS_MAINNET {
        // Set up blockchain client and token data for the given token address
        let data = setup(token).await?;

        // Generate token checklist using the fetched token data and client
        let token_checklist = generate_token_checklist(data.token, &data.client).await?;

        info!("token checklist => {:#?}", token_checklist);

        // Calculate token score based on predefined rules
        let token_score = get_token_score_with_rules_based_approch(token_checklist.clone());

        info!("token score (rule based) => {:#?}", token_score);

        // Calculate token score using AI model
        let token_score_ai = get_token_score_with_ai(token_checklist, &AI_MODEL).await?;
        info!("token score (ai) => {:#?}", token_score_ai);
    }

    Ok(())
}

/// Sets up the environment for an ERC20 token given its address.
///
/// # Parameters
/// - `token_address`: A string slice representing the token's address.
///
/// # Returns
/// - A `SetupData` struct containing the initialized provider and token data.
///
/// # Errors
/// Returns an error if any step in the initialization (e.g., connecting to the provider, parsing addresses, or fetching data) fails.
pub async fn setup(token_address: &str) -> Result<SetupData> {
    dotenv().ok();
    let ws_url = CHAIN_DATA.get_address(CHAIN).ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    // Parse the token address from string to Address type
    let token_address_h160: Address = token_address.parse()?;
    let token_contract = ERC20::new(token_address_h160, client.clone());

    // Get basic token data (symbol, decimals, name) from the ERC20 contract
    let symbol = token_contract.symbol().call().await?;
    let decimals = token_contract.decimals().call().await?;
    let name = token_contract.name().call().await?;

    // Retrieve the pair address and determine if the token is token_0 in the pair
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
