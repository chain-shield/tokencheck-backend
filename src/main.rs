use anyhow::Result;
use chainshield_backend::{data::contracts::CONTRACT, utils::logging::setup_logger};
use dotenv::dotenv;
use ethers::providers::{Provider, Ws};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    let ws_url = CONTRACT.get_address().ws_url.clone();
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider.clone());

    Ok(())
}
