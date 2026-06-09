# TASK-1381: Split parser surface/lowering/import resolver modules

## Status: 📝 Planned

## Description

Split the largest parser files (`surface.rs`, `parse_module.rs`, `lower.rs`, `import_resolver.rs`, `parse_expr.rs`, `parse_workflow.rs`, and `lift.rs`) into feature-owned modules while preserving parser/lowering behavior and AST compatibility.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Split `surface.rs` into AST groups: module items, interfaces/impls, expressions/workflows, tests/helpers where appropriate.
2. Split `parse_module.rs` by module item parser family while keeping dispatch explicit.
3. Split `lower.rs`/`lift.rs` by source/target semantic families.
4. Split `import_resolver.rs` by graph discovery, module loading, summary merging, and diagnostics if live seams support that.
5. Keep parser keyword and dispatch behavior unchanged.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Create parser module map

Document current parser file responsibilities in the task closeout before moving code.

### Step 2: Extract surface AST groups with reexports

Preserve `ash_parser::surface::*` import compatibility unless all callsites can be updated safely.

### Step 3: Extract parser dispatch helpers carefully

After each extraction, run focused parser tests:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-parser
```

### Step 4: Validate downstream crates

Because parser types cross into engine/typeck/LSP, run:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace
```

### Step 5: Codex review

Ask Codex to verify no surface AST fields or lowering metadata were dropped during extraction.


## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
  - git diff --check
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-parser
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1381-size.md
checklist:
  - [ ] Refactor is behavior-preserving
  - [ ] Public API paths preserved or deliberately documented
  - [ ] ash-parser tests pass
  - [ ] workspace check passes for downstream parser users
  - [ ] ash-parser clippy is clean
  - [ ] Formatting and diff checks pass
  - [ ] Size report shows intended reduction or documented exception
  - [ ] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- Parser modules split by semantic family.
- `surface.rs` and lowering files no longer require loading the entire parser surface for small edits.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
