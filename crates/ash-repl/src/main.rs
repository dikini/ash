//! Standalone REPL binary for Ash workflow language.

use ash_repl::Repl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut repl = Repl::new(false)?;
    repl.run().await?;

    Ok(())
}
