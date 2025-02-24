//! ChainShield TokenCheck Backend Library.
//! This library provides modules for ABI interactions, data management, token checking, logging, and various utilities.

pub mod abi {
    pub mod erc20;
    pub mod uniswap_factory_v2;
    pub mod uniswap_pair;
    pub mod uniswap_pool;
    pub mod uniswap_quoter;
    pub mod uniswap_router_v2;
    pub mod uniswap_v3_factory;
    pub mod uniswap_v3_router;
}

pub mod data {
    pub mod chain_data;
    pub mod token_data;
}

pub mod token_check {
    pub mod ai {
        pub mod ai_structs;
        pub mod ai_submission;
    }
    pub mod openai {
        pub mod structs;
    }
    pub mod deepseek {
        pub mod structs;
    }
    pub mod check_token_lock;
    pub mod token_checklist;
    pub mod token_holder_check;
    pub mod token_liquidity_check;
    pub mod token_methods;
    pub mod token_score;
    pub mod external_api {
        pub mod etherscan_api;
        pub mod moralis;
        pub mod thegraph_api;
    }
    pub mod anvil {
        pub mod buy_sell;
        pub mod simlator;
        pub mod supporting_methods;
        pub mod tx_trait;
        pub mod validation;
    }
}
pub mod utils {
    pub mod logging;
    pub mod tx;
    pub mod type_conversion;
    pub mod web_scrapper;
}

pub mod app_config;
