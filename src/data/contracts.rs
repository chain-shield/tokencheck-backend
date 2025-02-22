/*!
    This module provides functionality to load and manage smart contract addresses
    for different blockchain networks. The addresses are read from a TOML configuration
    file (contracts.toml) and made available via a global, lazily-initialized instance.
*/

use ethers::types::Chain;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{collections::HashMap, fs};

use crate::app_config::CHAIN;

/// Ethereum placeholder address used for representing ETH.
pub const ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
/// Bitcoin placeholder address.
pub const BTC: &str = "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB";
/// USD placeholder address.
pub const USD: &str = "0x0000000000000000000000000000000000000348";

/// Holds various smart contract addresses used by the application.
#[derive(Debug)]
pub struct ContractAddresses {
    /// Wrapped Ether (WETH) contract address.
    pub weth: String,
    /// Chainlink (LINK) token contract address.
    pub link: String,
    /// Uniswap Swap Router contract address.
    pub uniswap_swap_router: String,
    /// Uniswap V2 Router contract address.
    pub uniswap_v2_router: String,
    /// Uniswap V2 Factory contract address.
    pub uniswap_v2_factory: String,
    /// Uniswap Factory contract address.
    pub uniswap_factory: String,
    /// Uniswap Quoter contract address.
    pub uniswap_quoter: String,
    /// WebSocket endpoint URL for blockchain access.
    pub ws_url: String,
    /// HTTP endpoint URL for blockchain access.
    pub http_url: String,
    /// Alchemy API URL for blockchain access.
    pub alchemy_url: String,
}

/// Maps blockchain chains to their corresponding contract addresses.
#[derive(Debug)]
pub struct ContractAddressMap {
    /// A mapping from blockchain chain identifiers to their smart contract addresses.
    pub addresses: HashMap<Chain, ContractAddresses>,
}

impl ContractAddressMap {
    /// Constructs a new `ContractAddressMap` from the given chain configurations.
    ///
    /// This method extracts the relevant contract addresses for supported chains and
    /// builds a mapping for quick access.
    fn new(chains: Chains) -> Self {
        let mut addresses = HashMap::<Chain, ContractAddresses>::new();

        // Insert contract addresses for the Base chain.
        addresses.insert(
            Chain::Base,
            ContractAddresses {
                uniswap_factory: chains.base.uniswap_factory,
                uniswap_swap_router: chains.base.uniswap_swap_router,
                uniswap_v2_factory: chains.base.uniswap_v2_factory,
                uniswap_v2_router: chains.base.uniswap_v2_router,
                uniswap_quoter: chains.base.uniswap_quoter,
                weth: chains.base.weth,
                link: chains.base.link,
                ws_url: chains.base.ws_url,
                http_url: chains.base.http_url,
                alchemy_url: chains.base.alchemy_url,
            },
        );

        // Insert contract addresses for the Mainnet chain.
        addresses.insert(
            Chain::Mainnet,
            ContractAddresses {
                uniswap_factory: chains.mainnet.uniswap_factory,
                uniswap_swap_router: chains.mainnet.uniswap_swap_router,
                uniswap_quoter: chains.mainnet.uniswap_quoter,
                uniswap_v2_factory: chains.mainnet.uniswap_v2_factory,
                uniswap_v2_router: chains.mainnet.uniswap_v2_router,
                weth: chains.mainnet.weth,
                link: chains.mainnet.link,
                ws_url: chains.mainnet.ws_url,
                http_url: chains.mainnet.http_url,
                alchemy_url: chains.mainnet.alchemy_url,
            },
        );
        Self { addresses }
    }

    /// Retrieves the contract addresses corresponding to the current chain defined in the app configuration.
    ///
    /// # Panics
    ///
    /// Panics if the current chain is not supported (i.e., missing from the addresses mapping).
    pub fn get_address(&self) -> &ContractAddresses {
        self.addresses.get(&CHAIN).expect("Chain not supported")
    }
}

#[derive(Deserialize, Debug)]
struct Chains {
    /// Contract address configuration for the Base chain.
    base: ChainContracts,
    /// Contract address configuration for the Mainnet chain.
    mainnet: ChainContracts,
}

#[derive(Deserialize, Debug)]
struct ChainContracts {
    /// Address for the Uniswap Swap Router.
    uniswap_swap_router: String,
    /// Address for the Uniswap Factory.
    uniswap_factory: String,
    /// Address for the Uniswap Quoter.
    uniswap_quoter: String,
    /// Address for the Uniswap V2 Router.
    uniswap_v2_router: String,
    /// Address for the Uniswap V2 Factory.
    uniswap_v2_factory: String,
    /// Wrapped Ether (WETH) contract address.
    weth: String,
    /// Chainlink (LINK) token contract address.
    link: String,
    /// WebSocket URL for accessing the blockchain node.
    ws_url: String,
    /// HTTP URL for accessing the blockchain node.
    http_url: String,
    /// Alchemy API URL for enhanced blockchain connectivity.
    alchemy_url: String,
}

impl Chains {
    /// Loads the chain configuration from the `contracts.toml` file.
    ///
    /// # Panics
    ///
    /// Panics if the configuration file cannot be read or parsed.
    /// Consider propagating errors using the `?` operator (and returning a `Result`)
    /// for more robust error handling.
    fn load() -> Self {
        let config = fs::read_to_string("contracts.toml").expect("failed to read toml file");
        toml::from_str(&config).expect("failed to parse toml to chain config")
    }
}

/// Lazily initialized global instance holding all contract addresses for the configured chain.
///
/// The instance is created when first accessed using the configuration defined in `contracts.toml`.
pub static CONTRACT: Lazy<ContractAddressMap> = Lazy::new(|| {
    // Load chain configurations from the file.
    let chains = Chains::load();
    // Construct the mapping of chains to their respective contract addresses.
    ContractAddressMap::new(chains)
});
