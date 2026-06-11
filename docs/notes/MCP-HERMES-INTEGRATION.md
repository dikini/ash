# MCP-Hermes Integration Notes

## Quick Start

1. Build the release binary:
   ```bash
   cargo build -p ash-mcp --release
   ```

2. Add to Hermes (one-time):
   ```bash
   hermes mcp add ash-mcp --command "$(pwd)/target/release/ash-mcp"
   ```

3. Verify:
   ```bash
   hermes mcp list
   hermes mcp test ash-mcp
   ```

4. Start a new Hermes session (`/reset` or new `hermes` invocation) to pick up the tools.

## Project-Local Config

The file `.hermes/mcp_servers.yaml` in the project root is automatically loaded by Hermes when running inside the project directory. This is checked into version control so all agents share the same configuration.

## Troubleshooting

### "Connection closed" on `hermes mcp add`

- **Cause**: Hermes tests the server by launching it and waiting for the MCP handshake. If the binary doesn't exist or the build is stale, the connection fails immediately.
- **Fix**: Run `cargo build -p ash-mcp --release` and use the absolute path to the binary.

### Tools not appearing in session

- **Cause**: Toolset changes only take effect on new sessions (`/reset` or new `hermes` invocation).
- **Fix**: Start a new session. Do NOT expect tools to appear mid-conversation.

### `cargo run` vs release binary

- `cargo run -p ash-mcp --quiet` works for manual testing but is too slow for Hermes's connection timeout during `mcp add`.
- Always use the release binary for Hermes integration.

### Stdio cleanliness

- `ash-mcp` writes tracing logs to stderr and MCP messages to stdout.
- Hermes's stdio transport expects stdout to be pure JSON-RPC.
- The `RUST_LOG=warn` env var keeps stderr quiet under normal operation.
