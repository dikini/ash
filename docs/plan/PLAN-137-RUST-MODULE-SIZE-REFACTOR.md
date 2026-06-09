# Phase 137: Rust Module Size and Discoverability Refactor Plan

> **For Hermes:** Use `subagent-driven-development`, `rust-skills`, `test-driven-development`, `code-review`, and `verification-before-completion` to implement this plan task-by-task. Each implementation task must finish with Codex review for code quality, semantic preservation, style, and split-rule compliance before moving on.

**Goal:** Reduce oversized Rust files and modules so Ash source is easier for humans and LLM agents to read, load, navigate, review, and modify without changing language semantics.

**Architecture:** This is a behavior-preserving refactor phase. Work is divided by crate, prioritized by measured file size and architectural centrality. Each task extracts feature-owned submodules, preserves existing public APIs with `pub use`/compatibility reexports where needed, and adds a size-report gate so future work can see whether the module-size budget is improving.

**Tech Stack:** Rust 2024 workspace, Cargo, rustfmt, clippy, project-local shell/Python size audit, existing Ash test suites.

---

## Baseline size audit

Measured on `main` at `975ccea8` after Phase 136 merge.

- Workspace crates scanned: 18
- Rust files scanned: 663
- Rust files larger than 500 lines: 165
- Rust files larger than 10KB: 284

Highest-priority outliers by line count:

| Rank | File | Lines | Size |
|---:|---|---:|---:|
| 1 | `crates/ash-typeck/src/type_env.rs` | 20,935 | 807.1KB |
| 2 | `crates/ash-engine/src/module_loader.rs` | 8,248 | 294.7KB |
| 3 | `crates/ash-cli/src/test_runner/synthesized.rs` | 7,524 | 283.6KB |
| 4 | `crates/ash-interp/src/eval.rs` | 6,545 | 224.8KB |
| 5 | `crates/ash-typeck/src/check_expr.rs` | 5,701 | 208.4KB |
| 6 | `crates/ash-parser/src/surface.rs` | 4,722 | 140.6KB |
| 7 | `crates/ashgrove/src/lib.rs` | 4,694 | 151.7KB |
| 8 | `crates/ash-parser/src/parse_module.rs` | 4,314 | 136.5KB |
| 9 | `crates/ash-parser/src/lower.rs` | 4,148 | 146.0KB |
| 10 | `crates/ash-interp/src/execute.rs` | 3,468 | 123.7KB |

## Rust split rules for this phase

Apply these `rust-skills` rules explicitly:

1. **`proj-mod-by-feature`**: split modules by semantic feature/phase boundary, not by arbitrary type bucket. Good examples: `type_env::interface_evidence`, `module_loader::semantic_summary`, `synthesized::law`, `eval::pattern`.
2. **`proj-pub-crate-internal` / `proj-pub-super-parent`**: new helper modules should default to `pub(crate)` or `pub(super)`. Do not widen APIs merely to cross module boundaries.
3. **`proj-pub-use-reexport`**: preserve stable public paths via narrow `pub use` reexports from the original module where external or cross-crate users depend on them.
4. **`proj-lib-main-split`**: executable entrypoints stay thin; move logic into named library modules where needed.
5. **`api-common-traits` / `doc-all-public`**: when moving public types, keep derives and docs attached to the type; do not hide docs in compatibility shims.
6. **`err-result-over-panic`**: extraction must not introduce new panic paths, `unwrap`, or `expect` in production code.
7. **`own-borrow-over-clone`**: prefer passing context by reference across new module seams rather than cloning large AST/type/env values.
8. **`anti-over-abstraction` / `anti-type-erasure`**: avoid generic helper frameworks or trait objects solely to make a split possible; move cohesive code first.
9. **`test-integration-dir` / `test-descriptive-names`**: split oversized tests by behavior area with descriptive filenames; preserve test intent and names when possible.
10. **`lint-rustfmt-check`**: every split task must pass rustfmt and clippy on the affected crate.

## Size budgets and acceptance targets

This phase is not allowed to change Ash semantics. The measurable objective is structural.

### Per-file target

- Preferred maximum for ordinary implementation files: **≤500 lines**.
- Temporary compatibility shells may remain larger only when they contain reexports/tests and are documented in the task closeout.
- No production `src/**/*.rs` file should remain above **2,500 lines** after its owning crate task completes, unless the task records a deliberate follow-on owner.
- New files should start under **500 lines** unless generated or fixture-heavy.

### Exception rules

- The 500-line and 10KB thresholds are audit triggers, not automatic refactor mandates; split tasks should preserve semantics and avoid arbitrary extraction just to satisfy a number.
- Generated, fixture-heavy, compatibility, or mechanically mirrored test files may exceed the preferred budget when the owning task records why splitting would reduce discoverability or increase semantic risk.
- Any production file still above 2,500 lines after its crate task must name a follow-on owner or record why it is intentionally deferred beyond Phase 137.
- Later tasks should compare against `docs/audit/RUST-FILE-SIZE-AUDIT.md` using `tools/dev/rust_file_size_report.py` and report whether the relevant crate improved, stayed flat for a documented reason, or regressed.

### Phase-level target

By TASK-1387 closeout:

- Top 10 production outliers from the baseline are split or have explicit follow-on owners.
- `type_env.rs`, `module_loader.rs`, `synthesized.rs`, and `eval.rs` no longer dominate a single LLM context load.
- Total count of Rust files >500 lines is reduced or, where line count temporarily rises due to splitting tests, the count of production `src/**/*.rs` files >2,500 lines is reduced to zero.
- A repeatable size audit command exists and is referenced from closeout docs.

## Implementation strategy

1. Add a reusable size-audit script/report first so every later task measures progress the same way.
2. Attack the worst crate first (`ash-typeck`) because it has the largest file and the largest count of files above both thresholds.
3. Proceed crate-by-crate, preserving public API through reexports and running crate-local gates before broad gates.
4. Split tests/support files only after production code is stabilized so verification remains trustworthy.
5. Close with a fresh audit and a guard/policy that prevents new mega-files from appearing unnoticed.

## Tasks

| Task | Title | Primary crate | Priority | Status |
|---|---|---|---:|---|
| [TASK-1378](tasks/TASK-1378-module-size-audit-and-policy.md) | Add module-size audit and split policy | workspace | 0 | Complete |
| [TASK-1379](tasks/TASK-1379-split-ash-typeck-type-env.md) | Split `ash-typeck::type_env` into feature modules | `ash-typeck` | 1 | Complete |
| [TASK-1380](tasks/TASK-1380-split-ash-typeck-checking-modules.md) | Split `ash-typeck` expression/checking frontends | `ash-typeck` | 2 | Complete |
| [TASK-1381](tasks/TASK-1381-split-ash-parser-surface-and-lowering.md) | Split parser surface/lowering/import resolver modules | `ash-parser` | 3 | Complete |
| [TASK-1382](tasks/TASK-1382-split-ash-engine-module-loader.md) | Split engine module loading and public engine shell | `ash-engine` | 4 | Complete |
| [TASK-1383](tasks/TASK-1383-split-ash-cli-synthesized-runner.md) | Split synthesized test runner modules | `ash-cli` | 5 | Planned |
| [TASK-1384](tasks/TASK-1384-split-ash-interp-runtime-modules.md) | Split interpreter eval/execute/runtime state modules | `ash-interp` | 6 | Planned |
| [TASK-1385](tasks/TASK-1385-split-ashgrove-and-secondary-crates.md) | Split ashgrove and secondary oversized crates | `ashgrove`, small crates | 7 | Planned |
| [TASK-1386](tasks/TASK-1386-split-oversized-tests-and-fixtures.md) | Split oversized test/support files by behavior | tests | 8 | Planned |
| [TASK-1387](tasks/TASK-1387-module-size-closeout.md) | Closeout: final audit, status, changelog, review | workspace | 9 | Planned |

## Verification matrix

Each task must run at minimum:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
git diff --check
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p <affected-crate>
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p <affected-crate> --all-targets --all-features -- -D warnings
```

Closeout must run:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test --workspace
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo doc --workspace --no-deps
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```

## Independent review requirements

For every implementation task:

- Delegate to Codex after local gates pass.
- Ask Codex to review: semantic preservation, API/reexport compatibility, module visibility, Rust style, file-size budget compliance, and test coverage.
- Treat `REQUEST_CHANGES`/blocking findings as reopening the task.
- Re-run focused gates and Codex re-review after blocking fixes.

## Non-goals

- No behavioral redesign of type checking, parsing, interpreter execution, engine loading, or generated tests.
- No public Ash language syntax changes.
- No dependency updates unless a split absolutely requires a small internal helper crate, which this phase should avoid.
- No broad performance optimization beyond avoiding obvious clone/panic regressions introduced by extraction.
