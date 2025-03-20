use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::token_check::token_score::TokenScoreAssessment;

/// Global cache for token token_scores, keyed by lowercase token address strings.
///
/// This static variable provides thread-safe access to a shared cache of `TokenScoreAssessment`
/// instances across the application.
pub static TOKEN_SCORE_HASH: Lazy<Arc<Mutex<HashMap<String, TokenScoreAssessment>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::<String, TokenScoreAssessment>::new())));

/// Retrieves all token token_scores from the cache.
///
/// Returns a copy of the entire token token_score cache as a HashMap.
pub async fn get_token_token_scores_from_cache() -> HashMap<String, TokenScoreAssessment> {
    let token_score_hash = Arc::clone(&TOKEN_SCORE_HASH);
    let tokens = token_score_hash.lock().await;

    tokens.clone()
}

/// Retrieves a specific token token_score from the cache by its address.
///
/// # Arguments
/// * `token_address` - The token address as a string
///
/// # Returns
/// * `Some(TokenScoreAssessment)` if the token is found in the cache
/// * `None` if the token is not found
pub async fn get_token_token_score_from_cache(token_address: &str) -> Option<TokenScoreAssessment> {
    let token_score_hash = Arc::clone(&TOKEN_SCORE_HASH);
    let token_scores = token_score_hash.lock().await;

    token_scores.get(&token_address.to_lowercase()).cloned()
}

impl TokenScoreAssessment {
    /// Updates the token token_score in the global cache.
    ///
    /// This method inserts or updates the current TokenScoreAssessment instance
    /// in the global cache, using the lowercase token address as the key.
    pub async fn save_to_cache(&self, token_address: &str) {
        let token_score_hash = Arc::clone(&TOKEN_SCORE_HASH);
        let mut tokens = token_score_hash.lock().await;
        let token_address = token_address.to_lowercase();
        tokens.insert(token_address, self.clone());
    }
}
