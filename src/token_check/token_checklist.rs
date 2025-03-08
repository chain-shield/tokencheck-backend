use super::anvil::validation::TokenStatus;
use super::external_api::moralis;
use super::token_holder_check::get_token_holder_check;
use super::token_liquidity_check::get_percentage_liquidity_locked_or_burned;
use crate::app_config::AI_MODEL;
use crate::data::token_data::ERC20Token;
use crate::token_check::ai::ai_submission::check_code_with_ai;
use crate::token_check::external_api::etherscan_api::{TokenWebData, get_source_code};
use crate::token_check::token_holder_check::TokenHolderCheck;
use crate::utils::type_conversion::address_to_string;
use ethers::providers::{Provider, Ws};
use std::sync::Arc;

///! This module implements token checking functionality to evaluate the
///! legitimacy of a token. It performs validations including source code analysis via an
///! AI model, token holder checks, liquidity assessments, online presence validations,
///! and simulated buy/sell transaction tests.

/// Holds the information used to check a token's legitimacy.
///
/// This structure aggregates data from multiple validations such as code analysis, liquidity
/// metrics, and token holder information to compute an overall legitimacy score.
#[derive(Debug, Default, Clone)]
pub struct TokenCheckList {
    /// The token data including name, address, symbol, chain, pair/pool address etc.
    pub token: ERC20Token,

    // Fields derived from AI-based code analysis
    /// Flag indicating if the token's source code is possibly associated with a scam.
    pub possible_scam: bool,
    /// Explanation as to why the token may or may not be a scam.
    pub reason_possible_scam: String,
    /// Flag to denote if suspicious code could be justified as legitimate.
    pub could_legitimately_justify_suspicious_code: bool,
    /// Explanation as to why suspicious code might be legitimate or not.
    pub reason_could_or_couldnt_justify_suspicious_code: String,

    // Fields derived from token holder and liquidity analysis
    /// The percentage of total tokens held by the top token holder.
    pub top_holder_percentage_tokens_held: f64,
    /// The percentage of tokens that are locked or burned.
    pub percentage_of_tokens_locked_or_burned: f64,
    /// The percentage of liquidity tokens that are locked or burned.
    pub percentage_liquidity_locked_or_burned: Option<f64>,
    /// The liquidity in USD for the token.
    pub liquidity_in_usd: f64,

    // Fields derived from online presence checks
    /// Indicates whether the token has a website.
    pub has_website: bool,
    /// Indicates whether the token is active on Twitter or Discord.
    pub has_twitter_or_discord: bool,

    /// Represents whether the token is sellable based on simulation.
    /// `Some(true)` if sellable, `Some(false)` if not sellable, `None` if buy simulation failed.
    pub is_token_sellable: Option<bool>,
}

/// Generates the token checklist by performing a sequence of asynchronous validations.
///
/// This function performs the following steps:
/// 1. Retrieves the token's source code.
/// 2. Analyzes the source code using an AI model to determine if there is any potentially scammy behavior.
/// 3. Obtains token holder details.
/// 4. Retrieves liquidity information (if token is on a DEX).
/// 5. Checks the percentage of liquidity locked or burned (if token is on a DEX).
/// 6. Simulates buy/sell transactions to verify token sellability (if token is on a DEX).
/// 7. Checks the token's online presence (e.g., website, Twitter, Discord).
///
/// # Arguments
///
/// * `token` - An instance of `ERC20Token` representing the token to be checked.
/// * `client` - An `Arc`-wrapped Ethereum provider for interacting with the blockchain.
///
/// # Returns
///
/// Returns a `TokenCheckList` wrapped in an `anyhow::Result` containing the aggregated data from all checks.
///
/// # Errors
///
/// Returns an error if any of the asynchronous API calls or validations fail.
pub async fn generate_token_checklist(
    token: &ERC20Token,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<TokenCheckList> {
    // Convert the token address to a string format for API calls.
    let token_address = address_to_string(token.address);

    // Step 1: Retrieve token source code.
    println!("1. grabbing source code..");
    let token_code = get_source_code(&token_address, &token.chain).await?;

    // Step 2: Analyze the token's source code using an AI model.
    println!("2. checking source code..");
    let token_code_check = check_code_with_ai(token_code, &AI_MODEL)
        .await?
        .ok_or_else(|| anyhow::anyhow!("AI code check did not return a result"))?;

    // Step 3: Perform token holder check. If no check is available, default values are used.
    println!("3. token holder check...");
    let token_holder_check = match get_token_holder_check(&token, client).await? {
        Some(check) => check,
        None => TokenHolderCheck::default(),
    };

    // Default values if Token is NOT on a DEX
    let mut liquidity_in_usd = 0.0;
    let mut percentage_liquidity_locked_or_burned: Option<f64> = None;
    let mut is_token_sellable: Option<bool> = None;

    // The following steps only execute if the token is on a DEX
    match token.clone().token_dex {
        Some(token_dex) => {
            // Step 4: Retrieve liquidity information.
            println!("4. getting liquidity...");
            liquidity_in_usd = token_dex.liquidity_in_usd;

            // Step 5: Retrieve the percentage of liquidity that is locked or burned.
            println!("5. getting % liquidity burned or locked...");
            percentage_liquidity_locked_or_burned =
                get_percentage_liquidity_locked_or_burned(&token, client).await?;

            // Step 6: Simulate a buy/sell to check token sellability.
            println!("6. running buy / sell simulation with anvil...");
            let token_status_from_simulated_buy_sell =
                token.validate_with_simulated_buy_sell().await?;

            is_token_sellable = match token_status_from_simulated_buy_sell {
                TokenStatus::CannotSell => Some(false),
                TokenStatus::Legit => Some(true),
                TokenStatus::CannotBuy => None, // Cannot determine sellability if buying fails
            };
        }
        None => {
            // No DEX information available for this token
        }
    }

    // Step 7: Check for online presence details of the token.
    println!("7. getting online presence...");
    let token_online_presence = match moralis::get_token_info(&token_address, &token.chain).await? {
        Some(online_presence) => online_presence,
        None => TokenWebData::default(),
    };

    // Construct the final TokenCheckList with data from all the above validations.
    let token_checklist = TokenCheckList {
        token: token.clone(),
        possible_scam: token_code_check.possible_scam,
        reason_possible_scam: token_code_check.reason,
        could_legitimately_justify_suspicious_code: token_code_check
            .could_legitimately_justify_suspicious_code,
        reason_could_or_couldnt_justify_suspicious_code: token_code_check
            .reason_could_be_legitimate_or_not,
        top_holder_percentage_tokens_held: token_holder_check.top_holder_percentage,
        percentage_of_tokens_locked_or_burned: token_holder_check
            .percentage_tokens_burned_or_locked,
        percentage_liquidity_locked_or_burned,
        liquidity_in_usd,
        has_website: !token_online_presence.website.is_empty(),
        has_twitter_or_discord: !token_online_presence.twitter.is_empty()
            || !token_online_presence.discord.is_empty(),
        is_token_sellable,
    };

    Ok(token_checklist)
}
