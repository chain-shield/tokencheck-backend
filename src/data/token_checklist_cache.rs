use ethers::types::Address;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::token_check::token_checklist::TokenCheckList;
use crate::utils::type_conversion::address_to_string;

pub static TOKEN_CHECKLIST_HASH: Lazy<Arc<Mutex<HashMap<String, TokenCheckList>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::<String, TokenCheckList>::new())));

pub async fn get_token_checklists() -> HashMap<String, TokenCheckList> {
    let token_checklist_hash = Arc::clone(&TOKEN_CHECKLIST_HASH);
    let tokens = token_checklist_hash.lock().await;

    tokens.clone()
}

pub async fn get_token_checklist(token_address: &str) -> Option<TokenCheckList> {
    let token_checklist_hash = Arc::clone(&TOKEN_CHECKLIST_HASH);
    let token_checklists = token_checklist_hash.lock().await;

    if let Some(token) = token_checklists.get(&token_address.to_lowercase()) {
        Some(token.clone())
    } else {
        None
    }
}

impl TokenCheckList {
    pub async fn update_state(&self) {
        let token_checklist_hash = Arc::clone(&TOKEN_CHECKLIST_HASH);
        let mut tokens = token_checklist_hash.lock().await;
        let token_address = address_to_string(self.token.address).to_lowercase();
        tokens.insert(token_address, self.clone());
    }
}
