use anyhow::anyhow;
use log::info;

use crate::{
    app_config::AI_MODEL,
    data::{
        provider_manager::get_chain_provider, token_checklist_cache::get_token_checklist,
        token_data::get_core_token_data_by_address,
    },
    token_check::{
        token_checklist::generate_token_checklist, token_score::get_token_score_with_ai,
    },
};

use super::{token_checklist::TokenCheckList, token_score::TokenScoreAssessment};

/// GET REQUEST WILL PROCESS TOKEN THROUGH THIS SERVICE
pub async fn get_token_audit_and_reputation_score(
    token_address: &str,
) -> anyhow::Result<(TokenCheckList, Option<TokenScoreAssessment>)> {
    let token_checklist = match get_token_checklist(token_address).await {
        Some(checklist) => checklist,
        None => {
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

    // TODO - SAVE TOKEN CHECKLIST TO CACHE

    // Calculate token score using AI model
    let token_score_ai = get_token_score_with_ai(&token_checklist, &AI_MODEL).await?;
    info!("token score (ai) => {:#?}", token_score_ai);

    Ok((token_checklist, token_score_ai))
}
