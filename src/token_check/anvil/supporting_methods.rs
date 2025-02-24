use super::simlator::AnvilTestSimulator;
use crate::abi::erc20::ERC20;
use crate::abi::uniswap_factory_v2::UNISWAP_V2_FACTORY;
use crate::data::chain_data::CHAIN_DATA;
use crate::data::token_data::ERC20Token;
use crate::token_check::anvil::tx_trait::Txs;
use crate::utils::type_conversion::address_to_string;
use anyhow::Result;
use ethers::types::{
    CallFrame, GethDebugTracerType, GethDebugTracingOptions, GethTrace, GethTraceFrame, H256, U256,
};
use ethers::{providers::Middleware, types::Address};
use log::debug;

/// Supporting methods for debugging and diagnosis.
impl AnvilTestSimulator {
    /// Traces a transaction given its hash.
    ///
    /// This function queries the Ethereum client using `debug_trace_transaction` with custom
    /// tracing options. If a revert is detected in the call trace, the function logs and prints
    /// the address of the revert call. Otherwise, it notifies that no revert was found.
    ///
    /// # Parameters
    /// - `tx_hash`: The transaction hash to be traced.
    ///
    /// # Returns
    /// - `Result<()>`: Returns Ok if the transaction trace was successfully processed, or an error otherwise.
    pub async fn trace_transaction(&self, tx_hash: H256) -> Result<()> {
        // Configure tracing options to include storage and stack tracing, and select a built-in call tracer.
        let mut tracing_options = GethDebugTracingOptions::default();
        tracing_options.disable_storage = Some(false); // Enable storage tracing
        tracing_options.disable_stack = Some(false); // Enable stack tracing
        tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(
            ethers::types::GethDebugBuiltInTracerType::CallTracer,
        ));

        // Retrieve the transaction trace using the provider.
        let trace = self
            .signed_client
            .provider()
            .debug_trace_transaction(tx_hash, tracing_options)
            .await?;

        // Process the trace by matching on its type.
        match trace {
            GethTrace::Known(GethTraceFrame::CallTracer(ref call_frame)) => {
                // Recursively search the call trace for a revert.
                if let Some(revert_call) = find_revert(call_frame) {
                    debug!("Revert: {:?}", revert_call);
                    println!("Revert occurred in call to: {:?}", revert_call.to);

                    // TODO: Proceed to decode the function (if needed)
                } else {
                    println!("No revert found in the trace");
                }
            }
            _ => {
                println!("Unexpected trace format");
            }
        }
        Ok(())
    }

    /// Performs a dummy token transfer.
    ///
    /// This function simulates a token transfer by moving tokens from one wallet to another and
    /// then transferring them back using two distinct ERC20 contract instances.
    ///
    /// # Parameters
    /// - `token_address`: The address of the ERC20 token contract.
    /// - `amount`: The amount of tokens to transfer.
    ///
    /// # Returns
    /// - `Result<()>`: Returns Ok if both transfers are successful, or an error otherwise.
    pub async fn do_dummy_transfer(
        &self,
        token_address: Address,
        amount: U256,
    ) -> anyhow::Result<()> {
        // Create contract instances for both wallet clients.
        let token_contract_wallet_1 = ERC20::new(token_address, self.signed_client.clone());
        let token_contract_wallet_2 = ERC20::new(token_address, self.second_signed_client.clone());

        // Transfer tokens from wallet1 to wallet2.
        token_contract_wallet_1
            .transfer(self.second_signed_client.address(), amount)
            .send()
            .await?;

        // Transfer tokens back from wallet2 to wallet1.
        token_contract_wallet_2
            .transfer(self.signed_client.address(), amount)
            .send()
            .await?;

        Ok(())
    }

    /// Displays the Uniswap V2 pair address for a given token.
    ///
    /// This function queries the Uniswap V2 factory contract to retrieve the pair address for the
    /// token and WETH. The order of arguments depends on the token's position (token0 vs. token1).
    /// It then logs both the fetched and the stored (real) pair addresses.
    ///
    /// # Parameters
    /// - `token`: A reference to an `ERC20Token` structure containing details about the token.
    ///
    /// # Returns
    /// - `Result<()>`: Returns Ok if the pair address is successfully retrieved, or an error otherwise.
    pub async fn show_eth_uniswap_v2_pair(&self, token: &ERC20Token) -> anyhow::Result<()> {
        // Retrieve addresses for the Uniswap V2 Factory and WETH contracts.
        let factory_address: Address = CHAIN_DATA.get_address().uniswap_v2_factory.parse()?;
        let weth_address: Address = CHAIN_DATA.get_address().weth.parse()?;

        // Create an instance of the Uniswap V2 Factory contract.
        let factory = UNISWAP_V2_FACTORY::new(factory_address, self.signed_client());

        // Depending on the token orientation, query the appropriate pair.
        if token.token_dex.is_token_0 {
            let pair_address = factory.get_pair(token.address, weth_address).call().await?;
            debug!("pair address for WETH-{}: {:?}", token.name, pair_address);
        } else {
            let pair_address = factory.get_pair(weth_address, token.address).call().await?;
            debug!("pair address for WETH-{}: {:?}", token.name, pair_address);
        }

        // Convert the stored pair address to a string and log it.
        let pair_address = address_to_string(token.token_dex.pair_or_pool_address);
        debug!(
            "REAL pair address for WETH-{}: {:?}",
            token.name, pair_address
        );

        Ok(())
    }
}

/// Recursively searches for a revert in a call trace.
///
/// The function examines the provided call frame and:
/// - Returns the frame immediately if it has an error and there are no nested calls.
/// - Recursively checks nested calls for a revert, returning the first found occurrence.
///
/// # Parameters
/// - `trace`: A reference to a `CallFrame` representing a transaction call trace.
///
/// # Returns
/// - `Option<&CallFrame>`: The call frame where the revert occurred, if found; otherwise, `None`.
pub fn find_revert(trace: &CallFrame) -> Option<&CallFrame> {
    // If this call frame has an error and no nested calls, it's the revert point.
    if trace.error.is_some() && (trace.calls.is_none() || trace.calls.as_ref().unwrap().is_empty())
    {
        return Some(trace);
    }

    // If there are nested calls, search each one recursively.
    if let Some(calls) = &trace.calls {
        for call in calls {
            if let Some(revert_call) = find_revert(call) {
                return Some(revert_call);
            }
        }
    }

    None
}
