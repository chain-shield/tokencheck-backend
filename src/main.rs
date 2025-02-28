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

/// Main function that iterates over tokens, performs token checks, and prints token scores using both rule-based and AI-based approaches.
///
/// # Errors
/// Returns an error if any step in token processing or blockchain communication fails.
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    Ok(())
}
