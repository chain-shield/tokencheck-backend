//! This module defines the main application configuration parameters such as the blockchain chain,
//! application mode, AI model selection, and various thresholds and limits used throughout the app.
//!
//! Modify these constants to adjust the application's behavior for production, simulation, or testing.

use crate::{data::dex::Dex, token_check::ai::ai_submission::AIModel};
use ethers::types::Chain;

/// Represents the operational mode of the application.
#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    /// Production mode with live settings.
    Production,
    /// Simulation mode for testing and development.
    Simulation,
}

// ==========================================================================
// CHANGE THESE VALUES TO SET CHAIN AND MODE FOR THE APPLICATION
// ==========================================================================

/// Chains this app supports
pub const CHAINS: [Chain; 2] = [Chain::Mainnet, Chain::Base];

/// dexes supported by app
pub const DEXES: [Dex; 2] = [Dex::UniswapV2, Dex::UniswapV3];

/// Specifies the blockchain chain where the application operates.
/// Adjust this value to target a different chain (e.g., Mainnet, Testnet).
pub const CHAIN: Chain = Chain::Mainnet;

/// Defines the mode in which the application runs.
/// Use `AppMode::Production` for live environments or `AppMode::Simulation` for testing.
pub const APP_MODE: AppMode = AppMode::Production;

/// Specifies the AI model used for token analysis.
pub const AI_MODEL: AIModel = AIModel::OpenAi;

/// This value is set to the equivalent of 10 ether.
pub const MIN_LIQUIDITY: u128 = 10_000_000_000_000_000_000; // 10 ether

/// Threshold for very low liquidity (in wei); tokens below this may be high risk.
///
/// Although the comment indicates "3 ether", the value here corresponds to 1 ether.
pub const VERY_LOW_LIQUIDITY_THRESHOLD: u128 = 1_000_000_000_000_000_000; // 3 ether

/// Minimum trade factor multiplier used in transactional and liquidity assessments.
pub const MIN_TRADE_FACTOR: u64 = 10;

/// Minimum reserve ETH factor multiplier used in liquidity calculations.
pub const MIN_RESERVE_ETH_FACTOR: u64 = 10;

/// Expected percentage of liquidity that should be locked (expressed as a percentage).
pub const LIQUIDITY_PERCENTAGE_LOCKED: f64 = 90.0;

/// Threshold percentage for token holders, used to detect concentrated ownership.
pub const TOKEN_HOLDER_THRESHOLD_PERCENTAGE: f64 = 10.0;

/// List of addresses considered as token lockers on the mainnet.
///
/// These addresses often represent team finances, burn addresses, or mechanisms to lock tokens.
pub const TOKEN_LOCKERS_MAINNET: [&str; 4] = [
    "0xe2fe530c047f2d85298b07d9333c05737f1435fb", // team finance (lowercased)
    "0x663a5c229c09b049e36dcc11a9b0d4a8eb9db214", // UNCX (lowercased)
    "0x000000000000000000000000000000000000dead", // token burn (lowercased)
    "0x0000000000000000000000000000000000000000", // token burn
];

/// List of addresses considered as base token lockers.
///
/// These addresses are typically used for non-trading purposes such as token burns or long-term holds.
pub const TOKEN_LOCKERS_BASE: [&str; 3] = [
    "0xc4e637d37113192f4f1f060daebd7758de7f4131", // UNCX (lowercased)
    "0x000000000000000000000000000000000000dead", // token burn (lowercased)
    "0x0000000000000000000000000000000000000000", // token burn
];

/// Maximum allowed size (in bytes) for a token contract's source code.
pub const CONTRACT_TOKEN_SIZE_LIMIT: u32 = 15_000;

/// Maximum character length allowed for a token project's website content.
pub const WEBSITE_MAX_CHARACTER_LENGTH: u32 = 40_000;

/// Maximum number of allowed purchase attempts.
pub const PURCHASE_ATTEMPT_LIMIT: u8 = 5;

/// Maximum number of allowed sell attempts.
pub const SELL_ATTEMPT_LIMIT: u8 = 10;

/// Maximum number of API checks that can be performed.
pub const API_CHECK_LIMIT: u8 = 10;

/// List of blacklisted tokens or identifiers.
pub const BLACKLIST: [&str; 1] = ["CHILLI"];

/// Prompt for final token determination (updated version) for JSON assessment.
///
/// This prompt instructs a crypto investigator on how to analyze a token's source code
/// and on-chain data, returning a reputation score and a detailed justification in valid JSON.
pub const FINAL_DETERMINATION_PROMPT_UPDATED: &str = r#"You are a senior crypto investigator. I will provide the following JSON assessment:

The result of analyzing an ERC-20 contract's source code for potential scams or malicious features, including stats on tokens holders, liquidity, and online presence.

This provided assessment will have the following field: 

    - token_name
    - token_address
    - token_symbol

    - possible_scam (boolean)

    // 2 to 3 sentences as to why (or why not) its a scam
    - reason_possible_scam 

    // could suspicious code be legitimately used to defend against bots and snippers?
    - could_legitimately_justify_suspicious_code (boolean)

    // 2 to 3 sentences as to why (or why not) suspicious could be justified
    - reason_could_or_couldnt_justify_suspicious_code

    // what percentage of tokens does top token holder own?
    - top_holder_percentage_tokens_held (0.0 to 100.0)

    // percentage of total tokens minted that are locked or burned (ie not available for circulation)
    - percentage_of_tokens_locked_or_burned (0.0 to 100.0)

    // what percentage of LP (liquidity tokens) is locked (in 3rd party locker) or burned (pointing to zero/dead address)
    - percentage_liquidity_locked_or_burned Some(0.0 to 100.0) // wrapped in some because its Option<f64> (rust), if value is None then could not determine value

    // the amount of liquidity (in wei) the token has on a major exchange (uniswap, etc)
    - liquidity_in_wei

    // does token have a website?
    - has_website (boolean)

    // does token have a twitter profile or discord channel
    - has_twitter_or_discord (boolean)

    // Is token sellable or transferable when simulating swap with foundry anvil?
    - is_token_sellable: Some(true or false), if None then could not run simulation, and result is indetermined

Based on these inputs, please make a holistic determination on the legitimacy of the token and return one of the following scores:

4 - Legit,
3 - Likely Legit,
2 - Iffy,
1 - Likely Scam,
0 - Scam,

*Note: if token is well known token with an established high reputation and history, then score the token as "4 - Legit"

Your **output must be strictly valid JSON** (no extra text or code fencing), in this format:

{
  "token_score": "4 - Legit" | "3 - Likely Legit" | "2 - Iffy" | "1 - Likely Scam" | "0 - Scam",
  "reason": "<5_to_7_sentences_explaining_in_detail_why_token_received_specified_score>",
}

Where:
- `token_score` is a token reputation score that must be one of the following values: "4 - Legit", "3 - Likely Legit", "2 - Iffy", "1 - Likely Scam", or "0 - Scam",
- `reason` is a 5 to 7 sentence justification of token score.

Return **only** valid JSON. Do NOT include triple backticks or any other formatting around the JSON.
"#;

/// Prompt for final token determination combining website and code analysis.
///
/// This prompt instructs an investigator to combine findings from both the website analysis and the code review,
/// returning a final scam assessment in valid JSON.
pub const FINAL_DETERMINATION_PROMPT: &str = r#"You are a senior crypto investigator. I will provide two JSON assessments:

1. The result of analyzing an ERC-20 contract's source code for potential scams or malicious features.
2. The result of analyzing the project's website for credibility.

Each assessment will have a `"possible_scam"` boolean and a `"reason"` field, plus additional summary/context if available. Using these two inputs, please:

1. Combine the findings: If the website is credible but the code is suspicious (or vice versa), decide whether the suspicious code might be legitimately used to defend against malicious trading bots, front-running, sniping, etc.
2. Make a final determination: Is this token possibly a scam or not?

Your **output must be strictly valid JSON** (no extra text or code fencing), in this format:

{
  "final_scam_assessment": <true_or_false>,
  "reason": "<2-3_sentences_explaining_why_you_concluded_scam_or_not>",
  "could_legitimately_justify_suspicious_code": <true_or_false>
}

Where:
- `final_scam_assessment` is a Boolean indicating if you think the token is likely a scam overall.
- `reason` is a concise (2–3 sentences) explanation.
- `could_legitimately_justify_suspicious_code` is a Boolean indicating whether the suspicious code could be explained by legitimate anti-bot or anti-exploit measures.
"#;

/// Prompt provided to a Solidity security reviewer to assess potential scam signals in an ERC-20 contract.
///
/// This prompt focuses on identifying signs of rug pulls, honeypots, and other malicious features in the source code.
/// The expected output is strictly valid JSON.
pub const CODE_CHECK_PROMPT: &str = r#"You are an expert Solidity security reviewer. I will provide you with an ERC-20 contract source code. You need to check whether this contract has any signs of being a rug pull, honeypot, or other scam.

Pay special attention to:
1. The transfer function or `_transfer` logic (any hidden conditions or blacklists).
2. Ownership methods (`Ownable`, `renounceOwnership`, etc.) and whether ownership is *actually* renounced—or if there is a hidden or alternate owner variable.
3. Any ability for the owner or privileged account to mint additional tokens.
4. Any external calls or "rescue tokens," "withdraw," or "removeLiquidity" methods that could drain user funds or liquidity.
5. Unusually high or dynamically modifiable fees that could be set to extreme values.
6. Proxy or upgradeable patterns that could hide malicious updates later.
7. Any hidden or custom logic that prevents selling or imposes heavy taxes on sellers.
8. Disregard any trust signals such as "renounced ownership" or "burned liquidity" unless it is clear there is *no* backdoor enabling the developer to regain control or drain liquidity.

After analyzing these points, respond **strictly** in the following JSON format (no additional text). The `reason` should not exceed 2 to 3 sentences:

{ 
  "possible_scam": <true_or_false>, 
  "reason": "<2_or_3_sentences_describing_rationale>" 
  "could_legitimately_justify_suspicious_code": <true_or_false>
  "reason_could_be_legitimate_or_not": <2_or_3_sentences_describing_rationale>
}

*Note*: 

- `could_legitimately_justify_suspicious_code` is a Boolean indicating whether the suspicious code could reasonably be explained by legitimate anti-bot, anti-snipper, or anti-exploit measures,
 that could be implemented by legitimate smart contracts. If there is no suspicious code just set to true.
`reason_could_be_legitimate_or_not` - If you set `could_legitimately_justify_suspicious_code` to true, THEN explain in a couple sentences WHY any suspicious code in contract could be legitimate anti-bot/anti-snipper/anti-exploit measures, 
 however if you set `could_legitimately_justify_suspicious_code` to false explain WHY. If there is no suspicious code just state "code is legitimate".

Please only produce valid JSON—no code fencing or extra explanation. Provide a Boolean for `possible_scam`.

FOLLOWED BY the solidity source code which will be in a String called "source_code".

Return **only** valid JSON. Do NOT include triple backticks or any other formatting around the JSON.
"#;

/// Prompt provided to a web credibility expert to assess a token project's website.
///
/// This prompt instructs the reviewer to look for scam signals, inconsistencies,
/// and missing critical information. The output must be strictly valid JSON.
pub const WEBSITE_CHECK_PROMPT: &str = r#"You are an expert crypto investigator specializing in website credibility. I will provide the text scraped from a token project's website. Please:

1. Determine if the website seems credible or if it exhibits potential scam signals.  
2. Focus on any obvious contradictions, fake partnerships, poorly written or copy-pasted content, unrealistic claims, or missing critical information (e.g., team info, roadmap, contact details).  
3. Summarize the site's content (in your own words) and highlight any suspicious indicators.

**Output must be strictly valid JSON** with this structure (no extra text or code fencing):

{
  "possible_scam": <true_or_false>,
  "reason": "<2_or_3_sentences_here>",
  "summary": "<overview_of_website_and_why_it's_credible_or_not>"
}

Where:
- `possible_scam` is a Boolean that indicates your judgment on whether the website is likely credible or possibly a scam.
- `reason` is a concise (2–3 sentence) explanation of why you concluded so.
- `summary` is a short overview (under 2000 tokens for the entire JSON response) describing key points of the website and its credibility.

Return only valid JSON. Do NOT include triple backticks or any other formatting around the JSON."#;

/// Prompt provided to a Twitter analyst for evaluating a crypto project's social media profile.
///
/// This prompt instructs the analyst to review the account's stats, engagement patterns, and recent posts,
/// returning a credibility assessment in strictly valid JSON.
pub const TWITTER_PROMPT: &str = r#"You are an expert crypto investigator specializing in social media analysis. I will provide the Twitter profile's stats of a crypto project (e.g., follower count, following count, account age) and its 50 most recent posts. Please:

1. Determine if the account seems credible or if it exhibits potential scam signals.
2. Look for inconsistencies, copy-pasted or repetitive "shilling," unrealistic claims, suspicious engagement patterns, or evidence of bot activity. Also note any red flags from extremely low engagement relative to follower count, newly created accounts with huge follower spikes, or significant mismatches between followers and actual interactions.
3. Summarize the account's overall behavior (in your own words) and highlight any red flags that might indicate a scam. This summary should be no longer than 500 tokens.
**Output must be strictly valid JSON** with this structure (no extra text or code fencing):

{
  "possible_scam": <true_or_false>,
  "reason": "<2_or_3_sentences_here>",
  "summary": "<brief_overview_of_the_tweets_account_stats_and_why_it's_credible_or_not>"
}

Where:
- `possible_scam` is a Boolean that indicates your judgment on whether the Twitter account is likely credible or possibly a scam.
- `reason` is a concise (2–3 sentence) explanation of why you concluded so.
- `summary` is a short overview describing key points of the account's content, engagement levels, follower stats, post history, etc. (limit the entire JSON response to under 2000 tokens).
"#;

//*****************************************
//*****************************************
//*****************************************
//*****************************************
//*****************************************
