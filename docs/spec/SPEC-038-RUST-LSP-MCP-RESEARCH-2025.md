# SPEC-038 Research Update: Rust LSP & MCP Stack (2025-04)

> Research date: 2026-04-15  
> Sources: crates.io API, GitHub API, Cargo.toml inspection of relevant repos.

---

## 1. LSP Frameworks

### `tower-lsp-server` (recommended)
- **Latest version:** `0.23.0` (crates.io, 2025-12-07)
- **Repository:** `tower-lsp-community/tower-lsp-server`
- **Maintenance:** Active. Last commit 2026-03-14. The original `ebkalderon/tower-lsp` has been unmaintained since 2024-08; the community fork is now the canonical async/tokio LSP framework.
- **Async/Tokio:** Built on Tower. Default feature `runtime-tokio` uses `tokio` 1.x and `tokio-util`. Fully compatible with the Ash project's tokio-first architecture.
- **Notable details:**
  - Uses `ls-types` (a maintained fork of `lsp-types`) instead of the upstream `lsp-types` crate, due to upstream URI-handling breakage in 0.96+ (`fluent_uri` replacing the `url` crate).
  - Rust 2024 edition, MSRV 1.85.
  - Adopted by `ast-grep` and other active projects.
- **Open concerns:** A few open issues around Windows/WSL2 URI roundtripping and `$/cancelRequest` exposure, but nothing blocking MVP development.

### `lsp-server`
- **Latest version:** `0.7.9` (2025-08-06)
- **Maintenance:** Excellent (rust-analyzer team).
- **Model:** Blocking/sync, crossbeam-channel based. **Not** an async/tokio framework.
- **Verdict:** Incorrect choice for Ash because the project uses tokio everywhere.

### `async-lsp`
- **Latest version:** `0.2.3` (2026-03-04)
- **Maintenance:** Active (150 stars).
- **Model:** Tower-based, optional tokio. Uses older `lsp-types` 0.95.
- **Verdict:** Viable but much smaller ecosystem than `tower-lsp-server`. Less battle-tested.

**Recommendation:** Keep `tower-lsp-server = "0.23"` in SPEC-038. It is the correct choice for a modern tokio-based LSP server.

---

## 2. MCP SDK Options

### `rmcp` (strongly recommended)
- **Latest version:** `1.4.0` (crates.io, 2026-04-10). Release PR for `1.5.0` is already open.
- **Repository:** `modelcontextprotocol/rust-sdk` (official Anthropic SDK)
- **Maintenance:** Extremely active (3.3k stars, 5.1M recent downloads, daily PRs).
- **Maturity:** Production-ready. Supports stdio, SSE, and HTTP transports.
- **Macros:** Provides `#[tool]`, `#[tool_handler]`, `#[prompt]` macros for ergonomic server definition.
- **Notable details:**
  - stdio transport is first-class (`rmcp::transport::stdio`).
  - There is an open enhancement request to remove the hard `tokio/rt` dependency, but for a tokio-based project this is irrelevant.

### `mcp-rs`
- **Latest version:** `0.1.0` (2024-11-29)
- **Downloads:** ~1,900 total. No linked repository.
- **Verdict:** Do not use.

### Hand-rolled JSON-RPC 2.0 over stdio
- **Verdict:** Possible, but unnecessary. Given `rmcp`'s maturity and official backing, hand-rolling adds maintenance burden with no clear benefit.

**Recommendation:** Update SPEC-038 from `rmcp = "0.1"` to **`rmcp = "1.4"`** (pin to `1.4` or `1.5` once released). This is the dominant, actively maintained Rust MCP SDK.

---

## 3. VFS / Caching Libraries

### `dashmap`
- **Latest stable:** `6.1.0` (2024-09-05)
- **Latest pre-release:** `7.0.0-rc2` (2025-03-05) — **not production stable**.
- **Verdict:** SPEC-038's proposed `dashmap = "6.1"` is still the right choice. Wait for 7.0 stable before upgrading.

### `salsa`
- **Latest version:** `0.26.0` (2026-02-07)
- **Maintenance:** Active, driven by rust-analyzer.
- **Maturity:** Production-ready for incremental query caching.
- **Verdict:** Appropriate for Phase 2 (polish) when Ash wants cross-file incremental analysis. For Phase 1 (MVP), a simple per-request cache or hashmap is sufficient.

### `vfs` (manuel-woelker/rust-vfs)
- **Latest version:** `0.13.0`
- **Verdict:** General-purpose virtual filesystem abstraction, not LSP-specific. Adds abstraction overhead without providing incremental text sync, version tracking, or URI mapping — all of which an LSP VFS needs.

### rust-analyzer's `ra_ap_vfs`
- **Latest version:** `0.0.328`
- **Verdict:** Published to crates.io but tightly coupled to rust-analyzer internals. Not recommended for external projects.

**Recommendation:**
- **VFS:** Hand-roll with `DashMap<String, FileSnapshot>` exactly as SPEC-038 proposes. This is standard practice (rust-analyzer also hand-rolls its VFS).
- **Phase 1 cache:** Simple `HashMap` or LRU keyed by `(path, version)`.
- **Phase 2 cache:** Migrate to **`salsa = "0.26"`**.

---

## 4. Notable Recent Developments (2024–2025)

1. **tower-lsp community fork (2025)**  
   The original `tower-lsp` stagnated. The `tower-lsp-community` fork took over, released 0.23, adopted Rust 2024, and switched to `ls-types` to escape the `lsp-types` URI breakage.

2. **`lsp-types` / URI ecosystem split (2024)**  
   `lsp-types` 0.96+ replaced the `url` crate with `fluent_uri`, removing `to_file_path()` and breaking Windows path handling. This caused Helix and `tower-lsp-server` to fork or pin the type crates. `async-lsp` remains on `lsp-types` 0.95.

3. **rust-analyzer continues to push `salsa` (2024–2025)**  
   rust-analyzer bumped to `salsa` 0.26 and continues refining VFS hashing, file-watching, and source-root semantics.

4. **MCP goes mainstream (2024–2025)**  
   Anthropic's `rmcp` SDK exploded from experimental 0.x to stable 1.4/1.5, adding HTTP/SSE transports, `#[tool]` macros, and streaming. It is now the de facto standard for Rust MCP servers.

5. **Rust 2024 Edition adoption**  
   `tower-lsp-server` and other tooling crates have adopted Rust 2024. Ash is already on 2024 edition, so compatibility is assured.

---

## 5. Proposed Dependency Updates for SPEC-038

| Layer | Current Spec Proposal | Recommended Update | Rationale |
|-------|----------------------|--------------------|-----------|
| LSP framework | `tower-lsp-server = "0.23"` | **Keep 0.23** | Active community fork, tokio-native. |
| Async runtime | `tokio = "1.42"` | **Bump to `1.52`** | Latest stable (2026-04-14). |
| Concurrency | `parking_lot = "0.12"` | **Use `0.12.5`** | Latest patch release. |
| VFS map | `dashmap = "6.1"` | **Keep 6.1** | 7.0 is still RC. |
| MCP SDK | `rmcp = "0.1"` | **Bump to `1.4`** (or `1.5`) | 0.1 is obsolete; 1.4 is current stable. |
| Phase-2 cache | (future `salsa`) | **`salsa = "0.26"`** | Current stable, proven in rust-analyzer. |

---

*End of research note.*
