# TASK-1403: Hermes MCP server configuration

## Status: 📝 Planned

## Description

Wire `ash-mcp` into Hermes so that agent sessions can call Ash language tools natively. Provide both user-profile and project-local configuration options.

## Specification Reference

- [PLAN-140: MCP Agent Intelligence Spike](../PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md)
- Hermes Agent docs: https://hermes-agent.nousresearch.com/docs

## Dependencies

- TASK-1399 complete.

## Requirements

### Functional Requirements

- Add project-local MCP server config at `.hermes/mcp_servers.yaml`:
  ```yaml
  ash-mcp:
    command: cargo run -p ash-mcp --quiet
    cwd: /home/dikini/Projects/ash
  ```
- Document how to add the same config to `~/.hermes/profiles/default/mcp_servers.yaml` for non-project sessions.
- Verify Hermes discovers the tools (e.g., `hermes tools` lists `ash_get_diagnostics`, `ash_workspace_symbols`, etc.).
- Add a short `docs/notes/MCP-HERMES-INTEGRATION.md` note with troubleshooting.

### Non-Functional Requirements

- Config must not hard-code absolute paths in a way that breaks on other machines; use `${ASH_ROOT}` or note that cwd is project root.
- Must not require network access.

## Files

- Create: `.hermes/mcp_servers.yaml`
- Create: `docs/notes/MCP-HERMES-INTEGRATION.md`

## TDD Steps

1. Write config file.
2. Validate YAML syntax.
3. Launch server manually and confirm stdio handshake shape.
4. Verify tool discovery through Hermes CLI.
5. Document findings.

## Verification

- [ ] `ash-mcp` starts via config command without errors.
- [ ] Hermes lists at least the health, diagnostics, symbols, goto, references, and workspace-symbol tools.
- [ ] A sample tool call returns a non-error response.
- [ ] Note file is markdown-link clean.
- [ ] CHANGELOG.md updated.
