# Review: SPEC-038 Language Server Protocol (LSP) & MCP Interface

**Review Date:** 2026-04-15  
**Reviewer:** Hermes Agent  
**Scope:** SPEC-038 quality, completeness, and consistency with Ash project conventions.  
**Related Files:** `docs/spec/SPEC-038-LANGUAGE-SERVER.md`, `docs/plan/tasks/TASK-569-lsp-mcp-implementation.md`, `docs/spec/SPEC-005-CLI.md`

---

## Executive Summary

SPEC-038 is structurally sound but **not implementation-grade** in its current form. It contains aspirational timelines, invented type names that do not match the actual parser API, under-specified integration points with the existing compiler front-end, and critical omissions (configuration, crash recovery, multi-crate workspace loading). The 160-hour / 5-week estimate is **optimistic by roughly 30–50%** given the greenfield formatter work and MCP bridge scope.

---

## 1. Implementation Phases (5 Weeks) — Realism

**Severity: HIGH**

The 5-week timeline is **not realistic** for the stated scope.

| Phase | Claimed Work | Realistic Estimate |
|-------|--------------|--------------------|
| Week 1 | LSP skeleton + VFS + incremental sync + parser diagnostics | 1 week is feasible if parser integration is clean. |
| Week 2 | Hover + go-to-definition + document symbols + typeck diagnostics | Feasible **only if** name-binding tables are already LSP-ready. The spec does not verify this precondition. |
| Week 3 | MCP bridge + 5 tools + end-to-end testing with Claude/Codex | **Unrealistic.** MCP SDK integration (`rmcp` is immature), tool schema design, and agent testing is easily 1.5–2 weeks alone. |
| Week 4 | **Formatter** + code actions + workspace symbols + VSCode extension | **Severely unrealistic.** Ash has no formatter, no comment-trivia preservation, and no pretty-printer. Building one is 1.5–2 weeks minimum. |
| Week 5+ | Semantic tokens + reference caching + Salsa + rename | 1 week for semantic tokens and rename is possible, but adding Salsa-based incremental analysis is a multi-week refactor of its own. |

**Finding:** The spec treats the formatter as a side quest in Week 4 when it is actually the largest greenfield item. The MCP bridge is also squeezed into a single week alongside LSP feature work. For an implementation-grade spec, either the scope must be cut (defer formatter and Salsa to a follow-up spec) or the timeline extended to **7–8 weeks**.

---

## 2. Ambiguous Requirements

**Severity: CRITICAL**

### 2.1 "SurfaceAST" is not a real type
The spec invents `SurfaceAST` in function signatures:

```rust
pub fn parse_file(vfs: &Vfs, path: VfsPath) -> (SurfaceAST, Vec<ParseError>)
```

The actual parser produces `ash_parser::surface::ModuleFile` (or `Program` for entry points). The surface-to-parser contract (`docs/reference/surface-to-parser-contract.md`) never mentions `SurfaceAST`. **Implementation-grade specs must use exact crate-qualified type names.**

### 2.2 "Salsa-like query cache"
The spec says:

> **Phase 2 (Polish):** Introduce `salsa` or a hand-rolled query system

This is a **false choice** that papers over a major architecture decision. `salsa` is a specific crate with specific traits and database requirements. A "hand-rolled query system" is a completely different investment. The spec does not define:
- Query granularity beyond the four example functions.
- How invalidation propagates through the module graph.
- Whether `ash-typeck` will be refactored to expose queryable units.

**Recommendation:** Replace "Salsa-like" with a concrete Phase-2 RFC or delete the Phase-5 Salsa work and defer it to a separate specification.

### 2.3 `check_file` signature is fantasy
```rust
pub fn check_file(vfs: &Vfs, path: VfsPath, graph: &ModuleGraph)
    -> (TypedSurface, Vec<TypeError>)
```

There is no evidence that `ash-typeck` exposes a `TypedSurface` type or accepts a `ModuleGraph` in this shape. The spec must reference real `ash-typeck` APIs or define the required refactoring as explicit blockers.

---

## 3. MCP Tool List vs. LSP Capability Priorities

**Severity: MEDIUM**

### 3.1 Alignment gaps
The MCP tools mostly map 1:1 to LSP methods, but there are **priority mismatches**:

| MCP Tool | LSP Priority | Issue |
|----------|--------------|-------|
| `ash_find_references` | Priority 2 (Week 3–4) | Listed in MCP Phase 3 (Week 3) alongside Priority-1 features. Acceptable, but the spec should note that this depends on a cross-file reference index that does not yet exist. |
| `ash_workspace_symbols` | Priority 3 (Week 5+) | Listed in MCP Phase 3 (Week 3). **Misaligned.** Workspace symbols require index infrastructure not built until Week 4–5. |
| `ash_code_action` | Priority 2 (Week 3–4) | Listed in MCP Phase 3 (Week 3). Acceptable. |
| `ash_apply_edit` | N/A | **Conceptually mismatched.** `workspace/applyEdit` is an LSP *client* request, not a server capability. An MCP tool named `ash_apply_edit` implies the server will write files directly. This contradicts LSP architecture and raises security concerns (see §14). |

### 3.2 Missing MCP tools
- **No `ash_open_file` / `ash_close_file` tool.** The spec says MCP tools should auto-open files (§8.5), but it does not specify session lifetime or how the agent signals that a file is no longer needed.
- **No `ash_get_type` tool.** LSP hover provides types, but there is no dedicated MCP tool for "what is the type of this expression?" which is a common agent need.

---

## 4. Open Questions — Are They Answerable Now?

**Severity: MEDIUM**

The spec lists three open questions (§16). **All three are answerable now** with minimal codebase inspection.

| Question | Answerability | Actual Answer |
|----------|---------------|---------------|
| 1. Multi-crate workspaces? | **Answerable.** `ash-engine` already has crate-graph support (`parse_crate_root`, `DependencyDecl`, etc.). The LSP should support multi-crate workspaces to remain consistent with the compiler. | **Answer: Yes, it must support multi-crate workspaces.** The server should discover `ash.toml` / `.ash.toml` and use `ash-engine` to load the crate graph. |
| 2. Does `ash-parser` preserve comment trivia? | **Answerable.** Inspecting `crates/ash-parser/src/token.rs` and `surface.rs` would reveal whether `Span` or `Token` carries comment data. Based on the formatter section (§9.2) stating "If comment trivia is not currently stored, add it...", the answer is likely **no**. | **Answer: No, comment trivia is not preserved.** The spec already implies this; it should be stated as a known blocker rather than an open question. |
| 3. MCP bridge as separate binary or subcommand? | **Answerable.** SPEC-005 defines `ash lsp [options]`; Ash conventions favor subcommands over separate binaries for user-facing tools. | **Answer: Subcommand (`ash lsp --mcp`).** A separate binary (`ash-mcp`) is acceptable only if it is an internal implementation detail, but the CLI entry point should be `ash lsp --mcp`. |

**Recommendation:** Remove §16 and replace it with a "Known Blockers & Pre-conditions" section.

---

## 5. Contradictions with Existing Specs (SPEC-005 CLI)

**Severity: HIGH**

### 5.1 `tower-lsp` vs. `tower-lsp-server`
- **SPEC-005** (Implementation Notes): "Use `tower-lsp` for LSP implementation"
- **SPEC-038** / **TASK-569**: Use `tower-lsp-server = "0.23"`

This is a **dependency-level contradiction.** If `tower-lsp` is deprecated or unmaintained and the fork `tower-lsp-server` is the new standard, SPEC-005 must be updated. If not, SPEC-038 must justify the deviation.

### 5.2 TCP transport
- **SPEC-005** says `ash lsp` supports `--port <n>` (TCP).
- **SPEC-038** mentions TCP only under MCP transport (§8.2) and omits it for the LSP layer. The LSP layer must also support TCP to satisfy SPEC-005.

### 5.3 CLI subcommand shape
- **SPEC-005** defines `ash lsp [options]` with `--stdio` and `--port`.
- **SPEC-038** proposes `ash-mcp` as a possible separate binary (§16 Q3) and does not mention how the MCP interface is launched from the CLI. This contradicts the unified CLI surface defined in SPEC-005.

**Fix required:** SPEC-038 must state that `ash lsp --mcp` is the canonical launch mode, and that TCP is supported for both LSP and MCP transports.

---

## 6. Missing Sections

**Severity: HIGH**

The following sections are **required** for an implementation-grade spec but are missing or severely under-specified:

### 6.1 Configuration
- How does `ash-lsp` read `.ash.toml` / `ash.toml`?
- Are there LSP-specific config keys (e.g., debounce interval, max diagnostics)?
- How are initialization options handled (`initialize` params)?

### 6.2 Logging / Observability
- No mention of `tracing` integration, structured logging, or LSP message logging.
- No mention of how to debug MCP tool calls.

### 6.3 Crash Recovery & Error Handling
- What happens if the parser panics on malformed input?
- What is the LSP server restart strategy?
- No mention of `catch_unwind` or request isolation.

### 6.4 Multi-Crate Workspace Support
- Only asked as an open question. No design for:
  - Workspace root discovery (`ash.toml`, `.git`).
  - Crate graph loading via `ash-engine`.
  - Cross-crate goto-definition or symbol resolution.

### 6.5 Partial Parse / Error Recovery
- LSP must work on **broken code**. The spec assumes `parse_file` returns `(AST, errors)` but does not discuss:
  - Whether `ash-parser` supports error recovery (returning a partial AST).
  - How hover and completion behave when the file does not parse.

### 6.6 Reference Index Lifecycle
- "Find References" and "Rename" require a cross-file index. The spec does not specify:
  - When the index is built (on startup? on demand?).
  - How it is invalidated on `didChange`.
  - Memory bounds for large workspaces.

---

## 7. 160-Hour Estimate vs. 5-Week Timeline

**Severity: HIGH**

- **TASK-569** says: "4–5 weeks (1 engineer full-time)"
- **160 hours** = exactly 4 weeks at 40 h/wk.

**The estimate is not reasonable for the scope.**

| Work Item | Conservative Estimate |
|-----------|----------------------|
| LSP skeleton + VFS + incremental sync | 24 h |
| Parser/typeck integration + diagnostics | 24 h |
| Hover + go-to-definition + document symbols | 24 h |
| Completion + find references + index | 32 h |
| Formatter (greenfield, comment trivia, AST printing) | 40–56 h |
| MCP bridge + tool schema + testing | 32–40 h |
| VSCode extension + docs + integration tests | 16–24 h |
| Polish, semantic tokens, rename, Salsa | 24–40 h |
| **Total** | **216–304 hours** (5.5–7.5 weeks) |

**Recommendation:** Cut the formatter and Salsa work into follow-up tasks (TASK-XXX, TASK-YYY) and scope TASK-569 to **MVP LSP + MCP only** (≈160–180 hours / 4.5–5 weeks).

---

## 8. Next Step Inconsistency

**Severity: LOW (but embarrassing)**

SPEC-038 §418 says:

> **Next Step:** Create `docs/plan/tasks/TASK-XXX-lsp-mcp-implementation.md`

But `docs/plan/tasks/TASK-569-lsp-mcp-implementation.md` already exists and is linked from the spec's own task reference. **Fix:** Update the Next Step to reference TASK-569 directly.

---

## 9. Additional Issues

**Severity: MEDIUM**

### 9.1 VFS `VfsPath = String` is under-specified
Using raw LSP URI strings as map keys ignores:
- `file://` scheme normalization
- Percent-encoding differences
- Platform-specific path separators

**Fix:** Define a normalized path type or use `lsp_types::Url` with canonicalization.

### 9.2 Debounce code example is racey
The `schedule_validation` snippet (§7.3) shows a pseudo-code token cancellation pattern but uses `DashMap` and raw `tokio::time::sleep` without a structured background task loop. In an implementation-grade spec, this should reference a real task-spawning pattern (e.g., `tokio::select!` with an `mpsc` channel or `CancellationToken`).

### 9.3 `ash-lint` library extraction is a blocker, not a footnote
The spec notes `ash-lint` needs "lib extraction" but does not list it as a formal blocker. Since `crates/ash-lint/Cargo.toml` currently points only to `src/main.rs`, this is a **hard prerequisite.** TASK-569 correctly lists it under "Blocked By"; SPEC-038 should do the same.

### 9.4 `rmcp` dependency is speculative
```toml
rmcp = "0.1"          # or equivalent community crate
# If no mature Rust MCP crate exists, hand-roll JSON-RPC 2.0 over stdio.
```

This is not implementation-grade. The spec must either:
- Commit to `rmcp` and define the required version/features, **or**
- Define the hand-rolled protocol schema.

"If no mature crate exists" is not a plan.

---

## Recommendations Summary

| Action | Priority |
|--------|----------|
| Replace invented types (`SurfaceAST`, `TypedSurface`) with real `ash_parser::surface::ModuleFile` and actual `ash-typeck` types. | **P0** |
| Cut formatter and Salsa to separate follow-up specs; reduce TASK-569 scope. | **P0** |
| Add missing sections: Configuration, Logging, Crash Recovery, Workspace Root Discovery, Partial Parse Handling. | **P0** |
| Resolve `tower-lsp` vs. `tower-lsp-server` contradiction with SPEC-005. | **P0** |
| Convert open questions (§16) to known blockers with concrete answers. | **P1** |
| Fix MCP `ash_apply_edit` to align with LSP client edits or rename it and add a security note. | **P1** |
| Update Next Step to reference TASK-569. | **P2** |
| Commit to `rmcp` or define the hand-rolled MCP protocol schema. | **P1** |
| Revise estimate to 180–200 hours or cut scope to match 160 hours. | **P0** |

---

## Overall Verdict

**SPEC-038 is a good architectural sketch but fails the Ash convention of being "implementation-grade, not aspirational."** It must be revised to use real types, real APIs, concrete blockers, and a scope-matched timeline before engineering work begins.
