//! Standalone REPL binary for Ash workflow language.

use ash_repl::ReplConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ash_repl::run_with_config(ReplConfig::with_default_history()).await?;

    Ok(())
}
