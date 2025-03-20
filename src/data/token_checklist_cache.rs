use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::token_check::token_checklist::TokenCheckList;
use crate::utils::type_conversion::address_to_string;

/// Global cache for token checklists, keyed by lowercase token address strings.
///
/// This static variable provides thread-safe access to a shared cache of `TokenCheckList`
/// instances across the application.
pub static TOKEN_CHECKLIST_HASH: Lazy<Arc<Mutex<HashMap<String, TokenCheckList>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::<String, TokenCheckList>::new())));

/// Retrieves all token checklists from the cache.
///
/// Returns a copy of the entire token checklist cache as a HashMap.
pub async fn get_token_checklists_from_cache() -> HashMap<String, TokenCheckList> {
    let token_checklist_hash = Arc::clone(&TOKEN_CHECKLIST_HASH);
    let tokens = token_checklist_hash.lock().await;

    tokens.clone()
}

/// Retrieves a specific token checklist from the cache by its address.
///
/// # Arguments
/// * `token_address` - The token address as a string
///
/// # Returns
/// * `Some(TokenCheckList)` if the token is found in the cache
/// * `None` if the token is not found
pub async fn get_token_checklist_from_cache(token_address: &str) -> Option<TokenCheckList> {
    let token_checklist_hash = Arc::clone(&TOKEN_CHECKLIST_HASH);
    let token_checklists = token_checklist_hash.lock().await;

    token_checklists.get(&token_address.to_lowercase()).cloned()
}

impl TokenCheckList {
    /// Updates the token checklist in the global cache.
    ///
    /// This method inserts or updates the current TokenCheckList instance
    /// in the global cache, using the lowercase token address as the key.
    pub async fn save_to_cache(&self) {
        let token_checklist_hash = Arc::clone(&TOKEN_CHECKLIST_HASH);
        let mut tokens = token_checklist_hash.lock().await;
        let token_address = address_to_string(self.token.address).to_lowercase();
        tokens.insert(token_address, self.clone());
    }
}
