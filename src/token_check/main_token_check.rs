use anyhow::anyhow;
use log::info;

use crate::{
    app_config::AI_MODEL,
    data::{
        provider_manager::get_chain_provider,
        token_checklist_cache::get_token_checklist_from_cache,
        token_data::get_core_token_data_by_address,
    },
    token_check::{
        token_checklist::generate_token_checklist, token_score::get_token_score_with_ai,
    },
};

use super::{token_checklist::TokenCheckList, token_score::TokenScoreAssessment};

/// Processes a token audit request and generates a reputation score
///
/// This function serves as the main entry point for token auditing:
/// 1. Attempts to retrieve an existing token checklist from cache
/// 2. If not found, fetches token data and generates a new checklist
/// 3. Uses AI to analyze the checklist and generate a reputation score
///
/// # Arguments
/// * `token_address` - The blockchain address of the token to audit
///
/// # Returns
/// * A tuple containing the token checklist and optional score assessment
/// * Returns an error if token data cannot be retrieved or processing fails
pub async fn get_token_audit_and_reputation_score(
    token_address: &str,
) -> anyhow::Result<(TokenCheckList, Option<TokenScoreAssessment>)> {
    // First try to get checklist from cache
    let token_checklist = match get_token_checklist_from_cache(token_address).await {
        Some(checklist) => checklist,
        None => {
            // If not in cache, fetch token data and generate a new checklist
            if let Some(token_data) = get_core_token_data_by_address(token_address).await? {
                let client = get_chain_provider(&token_data.chain).await?;
                let checklist = generate_token_checklist(&token_data, &client).await?;
                checklist
            } else {
                return Err(anyhow!(
                    "could not get token data, address may not be valid"
                ));
            }
        }
    };

    info!("token checklist => {:#?}", token_checklist);

    // Note: The checklist should be saved to cache here to avoid regenerating it in future requests
    token_checklist.save_to_cache().await;

    // Calculate token score using AI model
    let token_score_ai = get_token_score_with_ai(&token_checklist, &AI_MODEL).await?;
    info!("token score (ai) => {:#?}", token_score_ai);

    //save token score to cache, if avaliable
    if let Some(token_score) = token_score_ai.clone() {
        token_score.save_to_cache(token_address).await;
    }

    Ok((token_checklist, token_score_ai))
}
