//! Claw-Harness Tool Store CLI
//! 
//! Marketplace for installing, publishing, and managing WASM plugins.

use tool_store::commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    commands::run().await
}
