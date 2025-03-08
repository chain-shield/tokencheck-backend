use ethers::types::Address;

pub const UNISWAP_V2_FEE: u32 = 3000;

// list of dexes
#[derive(Clone, Default, Debug)]
pub enum Dex {
    #[default]
    UniswapV2,
    UniswapV3,
    UniswapV4,
    Aerodrome,
    Sushiswap,
    Balancer,
    Curve,
    DackieSwap,
    BasedSwap,
    AlienBase,
    OasisSwap,
    LFGSwap,
    IcecreamSwap,
    Glacier,
    CrescentSwap,
    Throne,
    EtherVista,
    KokonutSwap,
    BakerySwap,
    CbsSwap,
    MoonBase,
    DegenBrains,
    Fwx,
    CandySwap,
    Memebox,
    BasoFinance,
    DerpDex,
    Satori,
    HorizonDex,
    BaseX,
    LeetSwap,
    RobotsFram,
    CitadelSwap,
    Velocimeter,
    DiamondSwap,
    SharkSwap,
    Infusion,
    NineMm,
    RocketSwap,
    Solidly,
    GammaSwap,
    Synthswap,
    IziSwap,
    Equalizer,
    SwapBased,
    Unknown,
}

/// the top dex the token is listed on
#[derive(Clone, Default, Debug)]
pub struct TokenDex {
    pub dex: Dex,
    pub pair_or_pool_address: Address,
    /// A flag indicating whether the token is the first token (token_0)
    /// in the Uniswap pair; if false the token is token_1.
    pub is_token_0: bool,
    pub fee: u32,
}
