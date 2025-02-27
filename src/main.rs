//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and performs token checking for tokens in the mainnet whitelist.
//! It demonstrates a test run where token data is fetched, token checklists are generated, and token scores are calculated using both rule-based and AI-based methods.

use anyhow::{anyhow, Result};
use chainshield_backend::{
    app_config::AI_MODEL,
    data::{
        provider_manager::get_chain_provider, token_checklist_cache::get_token_checklist,
        token_data::get_core_token_data_by_address,
    },
    token_check::{
        token_checklist::generate_token_checklist,
        token_score::{get_token_score_with_ai, get_token_score_with_rules_based_approch},
    },
    utils::logging::setup_logger,
};
use dotenv::dotenv;
use ethers::types::Address;
use log::info;

/// Whitelist tokens for mainnet testing.
pub const WHITELIST_TOKENS_MAINNET: [&str; 3] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    // "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

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
        let token_address: Address = token.parse()?;

        let token_checklist = match get_token_checklist(token_address).await {
            Some(checklist) => checklist,
            None => {
                if let Some(token_data) = get_core_token_data_by_address(&token).await? {
                    let client = get_chain_provider(&token_data.chain).await?;
                    let checklist = generate_token_checklist(token_data, &client).await?;
                    checklist
                } else {
                    return Err(anyhow!(
                        "could not get token data, address may not be valid"
                    ));
                }
            }
        };

        info!("token checklist => {:#?}", token_checklist);

        // TODO - SAVE TOKEN CHECKLIST TO CACHE

        // Calculate token score based on predefined rules
        let token_score = get_token_score_with_rules_based_approch(token_checklist.clone());

        // TODO - CREATE CACHE FOR TOKEN SCORE AND SAVE SCORE TO CACHE

        info!("token score (rule based) => {:#?}", token_score);

        // Calculate token score using AI model
        let token_score_ai = get_token_score_with_ai(token_checklist, &AI_MODEL).await?;
        info!("token score (ai) => {:#?}", token_score_ai);
    }

    Ok(())
}
