use super::anvil::validation::{TokenLiquid, TokenStatus};
use super::external_api::moralis;
use super::token_holder_check::get_token_holder_check;
use super::token_liquidity_check::get_percentage_liquidity_locked_or_burned;
use crate::app_config::AI_MODEL;
use crate::data::token_data::ERC20Token;
use crate::token_check::ai::ai_submission::check_code_with_ai;
use crate::token_check::external_api::etherscan_api::{get_source_code, TokenWebData};
use crate::token_check::token_holder_check::TokenHolderCheck;
use crate::utils::type_conversion::address_to_string;
use ethers::providers::{Provider, Ws};
use std::sync::Arc;

// This is the check list of 11 creteria to evaluate when checking
// if a toke is legitimate or not
// based on this check list a score will determined i.e. TokenScore
#[derive(Debug, Default, Clone)]
pub struct TokenCheckList {
    pub token_name: String,
    pub token_address: String,
    pub token_symbol: String,

    // below 4 fields are determined by submitting `CODE_CHECK_PROMPT` to
    // LLM (gpt-4o , deepseek-reasoner, etc)

    // if token is scam based on code review
    pub possible_scam: bool,
    // 2 to 3 sentences as to why (or why not) its a scam
    pub reason_possible_scam: String,
    // could suspicious code be justified as legitimate to counter bots and snippers?
    pub could_legitimately_justify_suspicious_code: bool,
    // 2 to 3 sentences as to why (or why not) suspicious could be justified
    pub reason_could_or_couldnt_justify_suspicious_code: String,

    // get below 3 fields from etherscan / moralis api and calculate

    // percentage of minted tokens owned by creator (deployer) of token contract
    // pub creator_percentage_tokens_held: f64,
    // what percentage of tokens does top token holder own?
    pub top_holder_percentage_tokens_held: f64,

    // percentage of total tokens minted that are locked or burned (ie not avaliable for circulation)
    pub percentage_of_tokens_locked_or_burned: f64,

    // what percentage of LP (liquidity tokens) is locked (in 3rd party locker) or burned (pointing to zero/dead address)
    pub percentage_liquidity_locked_or_burned: Option<f64>,

    // get data from etherscan / moralis api and calculate
    // how much liquidity (in wei) does token have on major exchange (uniswap, etc)
    pub liquidity_in_wei: u128,

    // get data from etherscan or moralis api
    // does token have a website?
    pub has_website: bool,
    // does token have a twitter profile or discord channel
    pub has_twitter_or_discord: bool,

    // run anvil simulation
    // Is token sellable or transferable when simulating it
    pub is_token_sellable: Option<bool>,
}

pub async fn generate_token_checklist(
    token: ERC20Token,
    client: &Arc<Provider<Ws>>,
) -> anyhow::Result<TokenCheckList> {
    let token_address = address_to_string(token.address);

    println!("1. grabbing source code..");
    let token_code = get_source_code(&token_address).await?;

    // get if token is `possible_scam` and `could_legitimately_justify_suspicious_code`
    println!("2. checking source code..");
    let token_code_check = check_code_with_ai(token_code, &AI_MODEL).await?.unwrap();

    // let token_contract_creator = get_contract_owner(&token_address).await?.unwrap();

    println!("3. token holder check...");
    let token_holder_check = match get_token_holder_check(&token, client).await? {
        Some(check) => check,
        None => TokenHolderCheck::default(),
    };

    println!("4. getting liquidity...");
    let liquidity_in_eth = token.get_liquidity(client).await?;

    println!("5. getting online presence...");
    let token_online_presense = match moralis::get_token_info(&token_address).await? {
        Some(online_presense) => online_presense,
        None => TokenWebData::default(),
    };

    println!("6. getting % liquidity burned or locked...");
    let percentage_liquidity_locked_or_burned =
        get_percentage_liquidity_locked_or_burned(&token, client).await?;

    println!("7. running buy / sell simulation with anvil...");
    let token_status_from_simulated_buy_sell = token
        .validate_with_simulated_buy_sell(TokenLiquid::HasEnough)
        .await?;

    let is_token_sellable = match token_status_from_simulated_buy_sell {
        TokenStatus::CannotSell => Some(false),
        TokenStatus::Legit => Some(true),
        TokenStatus::CannotBuy => None,
    };

    let token_holder_check = TokenCheckList {
        token_name: token.name,
        token_address,
        token_symbol: token.symbol,
        possible_scam: token_code_check.possible_scam,
        reason_possible_scam: token_code_check.reason,
        could_legitimately_justify_suspicious_code: token_code_check
            .could_legitimately_justify_suspicious_code,
        reason_could_or_couldnt_justify_suspicious_code: token_code_check
            .reason_could_be_legitimate_or_not,
        // creator_percentage_tokens_held: token_holder_check.creator_holder_percentage,
        top_holder_percentage_tokens_held: token_holder_check.top_holder_percentage,
        percentage_of_tokens_locked_or_burned: token_holder_check
            .percentage_tokens_burned_or_locked,
        percentage_liquidity_locked_or_burned,
        liquidity_in_wei: liquidity_in_eth,
        has_website: !token_online_presense.website.is_empty(),
        has_twitter_or_discord: !token_online_presense.twitter.is_empty()
            || !token_online_presense.discord.is_empty(),
        is_token_sellable,
    };

    Ok(token_holder_check)
}
