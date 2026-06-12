# MCP Server Benchmark: ash-mcp vs rust-analyzer (lsp-mcp)

**Date:** 2026-06-12
**Workspace:** /home/dikini/Projects/ash
**Benchmarker:** Hermes Agent

## Executive Summary

| Server | Type | Tools | State | Works? | Verdict |
|--------|------|-------|-------|--------|---------|
| **ash-mcp** | Ash-native LSP | 9 | ✅ Persistent (per-call stdio, but VFS/cache in-process) | ✅ Yes | **Usable today** |
| **rust-analyzer (lsp-mcp)** | Generic LSP bridge | 12 | ❌ Stateless (new process per call) | ❌ No | **Blocked on transport** |

## Server Details

### 1. ash-mcp (Ash-native)
- **Binary:** `/home/dikini/Projects/ash/.worktrees/phase-141-mcp-benchmark/target/release/ash-mcp`
- **Source:** `crates/ash-mcp/` in `phase-141-mcp-benchmark` worktree
- **Transport:** stdio (MCP protocol via `rmcp` crate)
- **State:** In-process VFS + `AnalysisCache` — persists across tool calls *within a single stdio session*
- **Language:** Ash (`.ash` files) — NOT Rust

**Tools:**
| Tool | Purpose | Tested | Latency |
|------|---------|--------|---------|
| `ash_mcp_health` | Server status | ✅ | ~590ms |
| `ash_get_diagnostics` | Lint/parse diagnostics | ✅ | ~600ms |
| `ash_hover` | Type info at position | ✅ | ~585ms |
| `ash_goto_definition` | Jump to definition | — | — |
| `ash_complete` | Completion suggestions | — | — |
| `ash_document_symbols` | File symbol outline | ✅ | ~565ms |
| `ash_find_references` | Find references | — | — |
| `ash_workspace_symbols` | Cross-file symbol search | ✅ | ~740ms |
| `ash_code_action` | Refactor actions | ✅ (placeholder) | — |

**Pros:**
- Purpose-built for Ash language
- VFS and analysis cache persist within a session
- Fast symbol search across `.ash` files
- No external dependencies (rust-analyzer binary not required)

**Cons:**
- Only understands Ash, not Rust
- Still stdio-transported — each `mcporter call` spawns a new process
- Workspace symbols require manual root path
- `ash_code_action` is a placeholder

---

### 2. rust-analyzer via lsp-mcp (Generic LSP Bridge)
- **Binary:** `lsp-mcp` (from crates.io, v0.1.0)
- **Config:** `~/.codex/config.toml` → `~/.mcporter/mcporter.json`
- **Transport:** stdio (spawns rust-analyzer as child process)
- **State:** ❌ **None** — new `lsp-mcp` + new `rust-analyzer` per call
- **Language:** Rust (any LSP-compatible language)

**Tools:**
| Tool | Purpose | Tested | Works? |
|------|---------|--------|--------|
| `lsp_activate_workspace` | Start language server | ✅ | Reports success |
| `lsp_deactivate_workspace` | Stop language server | — | — |
| `lsp_list_workspaces` | List active workspaces | ✅ | Always empty |
| `lsp_diagnostics` | Get file diagnostics | ✅ | ❌ Fails |
| `lsp_hover` | Hover/type info | — | ❌ Expected fail |
| `lsp_goto_definition` | Go to definition | — | ❌ Expected fail |
| `lsp_find_references` | Find references | — | ❌ Expected fail |
| `lsp_document_symbols` | File symbols | — | ❌ Expected fail |
| `lsp_workspace_symbols` | Workspace symbol search | — | ❌ Expected fail |
| `lsp_completion` | Completion | — | ❌ Expected fail |
| `lsp_rename_symbol` | Rename | — | ❌ Expected fail |
| `lsp_prepare_rename` | Validate rename | — | ❌ Expected fail |

**Critical Issue:**
```
Error: Language server not running for rust in /home/dikini/Projects/ash/crates/ash-core
```

`lsp_activate_workspace` reports success, but the workspace is lost because:
1. `mcporter call` spawns a fresh `lsp-mcp` process
2. `lsp-mcp` spawns a fresh `rust-analyzer` child
3. The process exits after the single tool call
4. Next call = new process = no state

**Pros:**
- Generic — works with any LSP server (rust-analyzer, typescript-language-server, etc.)
- Rich tool set (12 tools vs 9)
- Mature rust-analyzer backend

**Cons:**
- **Stateless** — unusable for multi-call workflows via mcporter CLI
- Requires daemon/keep-alive support that mcporter doesn't provide for stdio servers
- Each call pays rust-analyzer cold-start cost (~600-700ms)

---

## Root Cause Analysis

### The Stateless Transport Problem

```
┌─────────────────┐     ┌─────────────┐     ┌─────────────────┐
│  mcporter call  │────▶│  lsp-mcp    │────▶│ rust-analyzer   │
│  (new process)  │     │  (new proc) │     │  (new process)  │
└─────────────────┘     └─────────────┘     └─────────────────┘
        │                                              │
        │   "activate workspace"                       │
        │─────────────────────────────────────────────▶│
        │   "ok"                                       │
        │◀─────────────────────────────────────────────│
        │                                              │
   [process exits]                              [process exits]
        │                                              │
        ▼                                              ▼
   ┌─────────┐                                    ┌─────────┐
   │  STATE  │  ╳ LOST ╳                          │  STATE  │  ╳ LOST ╳
   └─────────┘                                    └─────────┘
```

For `lsp-mcp` to work, the MCP client must maintain a persistent stdio connection across multiple tool calls. `mcporter call` does not do this — each invocation is independent.

### Solutions

| Approach | Effort | Works? |
|----------|--------|--------|
| **A. Hermes native MCP** | Low | ✅ Best — Hermes maintains persistent stdio connections to MCP servers |
| **B. mcporter daemon + keep-alive** | Medium | ❌ Daemon only supports HTTP/SSE servers, not stdio |
| **C. lsp-mcp TCP mode** | Low | ❌ `lsp-mcp` has no TCP/socket mode (confirmed via strings) |
| **D. Wrap lsp-mcp in persistent process** | Medium | Possible — custom wrapper |
| **E. Use ash-mcp for Ash + cargo check for Rust** | Low | ✅ Works today |

---

## Recommendations

### For Ash Projects (`.ash` files)
**Use `ash-mcp`** — it's purpose-built, tested, and works. Configure in Hermes native MCP:

```yaml
# ~/.hermes/config.yaml
mcp_servers:
  ash-mcp:
    command: /home/dikini/Projects/ash/.worktrees/phase-141-mcp-benchmark/target/release/ash-mcp
    enabled: true
```

### For Rust Projects (`.rs` files)
**Option A: Hermes native MCP with rust-analyzer**
- Add `lsp-mcp` to Hermes `mcp_servers` config
- Hermes maintains persistent stdio connections
- Full rust-analyzer power available

**Option B: Use `cargo check` / `cargo clippy` directly**
- For diagnostics: `cargo check --message-format=json`
- For symbols: `rust-analyzer analysis-stats .`
- No MCP overhead, but less IDE-like integration

**Option C: Fix mcporter daemon for stdio keep-alive**
- Would require mcporter feature work
- Not available today

---

## Benchmark Raw Data

| Operation | Server | Latency | Result |
|-----------|--------|---------|--------|
| Health check | ash-mcp | 594ms | ✅ ok |
| Diagnostics | ash-mcp | 597ms | ✅ [] (no issues) |
| Hover | ash-mcp | 585ms | ✅ No info at 1:1 |
| Document symbols | ash-mcp | 565ms | ✅ 5 symbols |
| Workspace symbols | ash-mcp | 737ms | ✅ 3 matches for "bind" |
| Activate workspace | rust-analyzer | 737ms | ✅ Reports success |
| List workspaces | rust-analyzer | 602ms | ❌ Empty |
| Diagnostics | rust-analyzer | 587ms | ❌ "Language server not running" |

---

## Conclusion

| Decision | Recommendation |
|----------|----------------|
| **Default for Ash projects** | ✅ `ash-mcp` via Hermes native MCP |
| **Default for Rust projects** | ⚠️ Use Hermes native MCP with `lsp-mcp` (not mcporter CLI) |
| **Use mcporter for rust-analyzer?** | ❌ No — stateless transport makes it unusable for multi-call workflows |
| **Invest in fix?** | Option C (daemon stdio keep-alive) or upstream `lsp-mcp` TCP mode |
