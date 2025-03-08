use ethers::types::Address;

/// Default fee for Uniswap V2 pools (0.3%)
pub const UNISWAP_V2_FEE: u32 = 3000;

/// Represents different decentralized exchanges (DEXs) supported by the system
#[derive(Clone, Default, Debug)]
pub enum Dex {
    /// Uniswap V2 - Constant product AMM
    #[default]
    UniswapV2,
    /// Uniswap V3 - Concentrated liquidity AMM
    UniswapV3,
    /// Uniswap V4 - Hooks-enabled AMM
    UniswapV4,
    /// Aerodrome - Base chain AMM
    Aerodrome,
    /// Sushiswap - Fork of Uniswap with additional features
    Sushiswap,
    /// Balancer - Multi-token pools with customizable weights
    Balancer,
    /// Curve - Specialized for stablecoin and similar asset swaps
    Curve,
    /// DackieSwap - A decentralized exchange
    DackieSwap,
    /// BasedSwap - A decentralized exchange
    BasedSwap,
    /// AlienBase - A decentralized exchange
    AlienBase,
    /// OasisSwap - A decentralized exchange
    OasisSwap,
    /// LFGSwap - A decentralized exchange
    LFGSwap,
    /// IcecreamSwap - A decentralized exchange
    IcecreamSwap,
    /// Glacier - A decentralized exchange
    Glacier,
    /// CrescentSwap - A decentralized exchange
    CrescentSwap,
    /// Throne - A decentralized exchange
    Throne,
    /// EtherVista - A decentralized exchange
    EtherVista,
    /// KokonutSwap - A decentralized exchange
    KokonutSwap,
    /// BakerySwap - A decentralized exchange
    BakerySwap,
    /// CbsSwap - A decentralized exchange
    CbsSwap,
    /// MoonBase - A decentralized exchange
    MoonBase,
    /// DegenBrains - A decentralized exchange
    DegenBrains,
    /// Fwx - A decentralized exchange
    Fwx,
    /// CandySwap - A decentralized exchange
    CandySwap,
    /// Memebox - A decentralized exchange
    Memebox,
    /// BasoFinance - A decentralized exchange
    BasoFinance,
    /// DerpDex - A decentralized exchange
    DerpDex,
    /// Satori - A decentralized exchange
    Satori,
    /// HorizonDex - A decentralized exchange
    HorizonDex,
    /// BaseX - A decentralized exchange
    BaseX,
    /// LeetSwap - A decentralized exchange
    LeetSwap,
    /// RobotsFram - A decentralized exchange
    RobotsFram,
    /// CitadelSwap - A decentralized exchange
    CitadelSwap,
    /// Velocimeter - A decentralized exchange
    Velocimeter,
    /// DiamondSwap - A decentralized exchange
    DiamondSwap,
    /// SharkSwap - A decentralized exchange
    SharkSwap,
    /// Infusion - A decentralized exchange
    Infusion,
    /// NineMm - A decentralized exchange
    NineMm,
    /// RocketSwap - A decentralized exchange
    RocketSwap,
    /// Solidly - A decentralized exchange
    Solidly,
    /// GammaSwap - A decentralized exchange
    GammaSwap,
    /// Synthswap - A decentralized exchange
    Synthswap,
    /// IziSwap - A decentralized exchange
    IziSwap,
    /// Equalizer - A decentralized exchange
    Equalizer,
    /// SwapBased - A decentralized exchange
    SwapBased,
    /// Unknown DEX type
    Unknown,
}

/// Information about a token's listing on a decentralized exchange
#[derive(Clone, Default, Debug)]
pub struct TokenDex {
    /// The decentralized exchange where the token is listed
    pub dex: Dex,
    /// The address of the pair or pool contract containing the token
    pub pair_or_pool_address: Address,
    /// Indicates token position in the pair
    ///
    /// A flag indicating whether the token is the first token (token_0)
    /// in the pair; if false the token is token_1. This is important for
    /// determining price calculation direction and other pair-specific operations.
    pub is_token_0: bool,
    /// The fee tier of the pool (in basis points)
    ///
    /// For Uniswap V2-style pools, this is typically 3000 (0.3%)
    /// For Uniswap V3, common values are 500 (0.05%), 3000 (0.3%), 10000 (1%)
    pub fee: u32,
}
