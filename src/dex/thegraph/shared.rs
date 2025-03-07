use crate::app_config::{
    UNISWAP_V2_BASE_SUBGRAPH_ID, UNISWAP_V2_MAINNET_SUBGRAPH_ID, UNISWAP_V3_BASE_SUBGRAPH_ID,
    UNISWAP_V3_MAINNET_SUBGRAPH_ID,
};

// Token position in a pool
#[derive(Debug, Clone, Copy)]
pub enum TokenPosition {
    Token0,
    Token1,
}

impl TokenPosition {
    pub fn field_name(&self) -> &'static str {
        match self {
            TokenPosition::Token0 => "token0_",
            TokenPosition::Token1 => "token1_",
        }
    }
}

// SUBGRAPH ENUM
#[derive(Debug, Clone, Copy)]
pub enum SubGraph {
    UniswapV3Mainnet,
    UniswapV3Base,
    UniswapV2Mainnet,
    UniswapV2Base,
}

impl SubGraph {
    pub fn value(&self) -> &'static str {
        match self {
            SubGraph::UniswapV3Mainnet => UNISWAP_V3_MAINNET_SUBGRAPH_ID,
            SubGraph::UniswapV2Mainnet => UNISWAP_V2_MAINNET_SUBGRAPH_ID,
            SubGraph::UniswapV3Base => UNISWAP_V3_BASE_SUBGRAPH_ID,
            SubGraph::UniswapV2Base => UNISWAP_V2_BASE_SUBGRAPH_ID,
        }
    }
}
