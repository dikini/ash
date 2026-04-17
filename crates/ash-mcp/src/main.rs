//! Ash MCP server binary entry point.
//!
//! Launch via `ash lsp --mcp` (per SPEC-005) or directly as `ash-mcp`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to stderr so it doesn't interfere with stdio MCP transport.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { ash_mcp::run_stdio().await })
}
