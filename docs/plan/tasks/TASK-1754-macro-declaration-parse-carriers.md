# TASK-1754: Add parsed macro declaration and structured invocation-argument carriers

## Status: ✅ Complete

## Description

Add parser surface carriers for macro declarations and structured executable invocation arguments without executing macros. This task establishes source-preserving AST shape for local expression macros and keeps unsupported invocations fail-closed.

## Specification Reference

- PLAN-172
- SPEC-095c Phase 172 macro MVP subsection from TASK-1753
- TASK-1752 audit artifact

## Dependencies

- ✅ TASK-1752: Macro execution seam audit
- ✅ TASK-1753: Macro MVP spec amendments

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Raw macro body only | TASK-1748 | No execution | Partially | Add structured parenthesized expression args, keep raw body for diagnostics | Parser tests cover both |
| Qualified macro paths | Phase 171 review | Not represented | No | Keep rejected | Negative parser test |
| Bracket/brace execution | PLAN-172 D3 | No token-tree parser | No | Preserve carrier, do not execute | Negative expansion tests |

## Requirements

1. Add `Definition::Macro` or equivalent parsed macro declaration carrier in `crates/ash-parser/src/surface.rs`.
2. Add a `MacroDef` carrier with name, params, body expression, visibility, and span.
3. Parse module-level `macro name(params) => expr;` in `crates/ash-parser/src/parse_module.rs`.
4. Extend `MacroInvocation` with structured argument data only for the executable parenthesized subset, or add a separate parsed helper that preserves backward-compatible diagnostic fields.
5. Preserve existing fail-closed behavior for `name![...]`, `name!{...}`, malformed argument lists, and qualified macro-like invocations.
6. Update downstream visitors/renderers/checkers that must explicitly accept or reject the new carrier.
7. Add parser tests only; do not perform expansion yet.

## TDD Steps

### Step 1: Parser tests RED

**Files:**
- `crates/ash-parser/tests/task_1754_macro_declaration_parse.rs`

Test cases:
1. `macro inc(x) => add(x, 1);` parses as a macro definition.
2. Macro declaration preserves params, body, visibility, and span.
3. `inc!(n)` preserves structured one-arg invocation data.
4. `inc![n]` and `inc!{n}` remain diagnostic carriers but not executable structured-arg carriers.
5. `macros::inc!(n)` remains rejected.

### Step 2: Implement carriers and parser

**Files:**
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/parse_expr.rs`
- downstream explicit visitors as needed.

### Step 3: Workspace compatibility

Run parser tests and `cargo check --workspace` because `Definition`/`Expr` variants affect downstream crates.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1754_macro_declaration_parse -- --nocapture
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Parser tests pass.
  - [x] Unsupported forms remain fail-closed.
  - [x] Downstream consumers compile with explicit handling.
  - [x] CHANGELOG.md updated.
```

## Completion Evidence

Added `Definition::Macro` and `MacroDef` surface carriers, parsed module-level `macro name(params) => expr;`, and extended parenthesized `MacroInvocation` carriers with structured expression arguments while keeping bracket/brace carriers non-executable. Updated downstream LSP consumers to handle macro declarations explicitly. Verification passed:

```bash
cargo test -p ash-parser --test task_1754_macro_declaration_parse -- --nocapture
cargo check --workspace
cargo fmt --check
git diff --check
```

Focused test evidence: `task_1754_macro_declaration_parse` ran 4 tests and all passed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides carriers consumed by TASK-1755 local registry and TASK-1756 expansion.
