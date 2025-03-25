use anyhow::anyhow;
use dotenv::dotenv;
use log::info;
use tokencheck_backend::app_config::AI_MODEL;
use tokencheck_backend::data::provider_manager::get_chain_provider;
use tokencheck_backend::data::token_checklist_cache::get_token_checklist_from_cache;
use tokencheck_backend::data::token_data::get_core_token_data_by_address;
use tokencheck_backend::data::token_score_cache::get_token_token_score_from_cache;
use tokencheck_backend::token_check::token_checklist::generate_token_checklist;
use tokencheck_backend::token_check::token_score::get_token_score_with_ai;
use tokencheck_backend::utils::logging::setup_logger;

/// Whitelist tokens for mainnet testing.
pub const WHITELIST_TOKENS_MAINNET: [&str; 4] = [
    "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
    "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
    "0x1151CB3d861920e07a38e03eEAd12C32178567F6",
    "0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E",
];

#[tokio::test]
#[ignore]
async fn test_checklist_is_generated_with_link() -> anyhow::Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    let link = "0x514910771af9ca656af840dff83e8264ecf986ca";

    let token_checklist = match get_token_checklist_from_cache(link).await {
        Some(checklist) => checklist,
        None => {
            if let Some(token_data) = get_core_token_data_by_address(link).await? {
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

    Ok(())
}

#[tokio::test]
async fn test_checklist_and_token_score_generation_with_link() -> anyhow::Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    let link = "0x514910771af9ca656af840dff83e8264ecf986ca";

    let token_checklist = match get_token_checklist_from_cache(link).await {
        Some(checklist) => checklist,
        None => {
            if let Some(token_data) = get_core_token_data_by_address(&link).await? {
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

    // save to cache
    token_checklist.save_to_cache().await;

    // Calculate token score using AI model
    let token_score_ai = get_token_score_with_ai(&token_checklist, &AI_MODEL).await?;
    info!("token score (ai) => {:#?}", token_score_ai);

    let mut token_score_str = String::new();
    if let Some(token_score) = token_score_ai.clone() {
        token_score.save_to_cache(link).await;
        token_score_str = format!("{:?}", token_score);
    }

    // get save token scores
    let saved_token_checklist = get_token_checklist_from_cache(link).await.unwrap();
    let saved_token_score = get_token_token_score_from_cache(link).await.unwrap();

    let saved_token_checklist_str = format!("{:?}", saved_token_checklist);
    let saved_token_score_str = format!("{:?}", saved_token_score);
    let token_checklist_str = format!("{:?}", token_checklist);

    assert_eq!(saved_token_score_str, token_score_str);
    assert_eq!(saved_token_checklist_str, token_checklist_str);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_checklist_and_token_score_generation() -> anyhow::Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    // THIS IS FOR TESTING PURPOSES - WILL BE REPLACED BY SERVER CODE
    for token in WHITELIST_TOKENS_MAINNET {
        let token_checklist = match get_token_checklist_from_cache(token).await {
            Some(checklist) => checklist,
            None => {
                if let Some(token_data) = get_core_token_data_by_address(&token).await? {
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

        // Calculate token score using AI model
        let token_score_ai = get_token_score_with_ai(&token_checklist, &AI_MODEL).await?;
        info!("token score (ai) => {:#?}", token_score_ai);
    }

    Ok(())
}
