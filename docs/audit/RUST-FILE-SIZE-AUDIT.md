# Rust File Size Audit

Phase 137 baseline and final audit for the `phase-137-module-size` worktree.

## Audit commands

```bash
python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-final-size.md
python3 tools/dev/rust_file_size_report.py --json > /tmp/phase137-final-size.json
python3 tools/dev/rust_file_size_report.py --markdown --tests-only > /tmp/phase137-final-tests-size.md
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```

The audit script derives workspace package ownership from `cargo metadata --format-version 1 --no-deps`, excludes `.git/`, `target/`, and `.worktrees/`, and provides a Phase 137 regression guard for largest-file line/byte maxima without treating added split modules or medium-file counts as regressions.

## Phase 137 final summary

| Metric | Baseline | Final | Delta |
|---|---:|---:|---:|
| Workspace crates scanned | 18 | 18 | +0 |
| Rust files scanned | 663 | 757 | +94 |
| Rust files >500 lines | 165 | 187 | +22 |
| Rust files >10KB | 284 | 325 | +41 |

The file count intentionally increased because Phase 137 split large modules into smaller feature-owned files. The `>500` and `>10KB` counts are not required to monotonically decrease: splitting very large files can create medium-sized modules that still exceed advisory thresholds while substantially improving navigation and context load.

## Tests-only final summary

| Metric | Final |
|---|---:|
| Test/support Rust files scanned | 480 |
| Test/support Rust files >500 lines | 61 |
| Test/support Rust files >10KB | 152 |

TASK-1386 split the highest-confidence test binaries while preserving Cargo test binary roots and test identifiers. Remaining oversized tests are recorded as honest future opportunities rather than hidden semantic work.

## Key reductions and remaining exceptions

- `ash-typeck/src/type_env.rs` was replaced by a feature-owned module directory; remaining `ash-typeck` oversized files are mostly semantic/typechecking slices that require separate, behavior-preserving phases.
- `ash-cli/src/test_runner/synthesized.rs` was split into synthesized-runner feature modules; the largest remaining file is its extracted tests module.
- `ash-engine/src/module_loader.rs`, `ash-parser` parser surfaces, `ash-interp` eval/execute/runtime-state roots, `ashgrove/src/lib.rs`, and secondary roots were reduced or converted into feature modules.
- Remaining exceptions are accepted for Phase 137 closeout because they either need dedicated future decomposition (`ash-engine`/`ash-typeck`/`ash-interp` internals) or are test/support files where TASK-1386 split the highest-confidence targets without broad fixture churn.

## Remaining production outlier ownership

Phase 137 intentionally stops after behavior-preserving extraction of the highest-impact seams. Production files still above roughly 2,500 lines have explicit future owners below; each owner should be planned as a separate behavior-preserving split packet before implementation.

| Remaining file/family | Current size | Follow-on owner |
|---|---:|---|
| `crates/ash-engine/src/module_loader.rs` | 5,409 lines | Future `ash-engine` module-loader round-2 split: import graph, source discovery, dependency-root validation, and summary transport modules. |
| `crates/ash-typeck/src/check_expr/mod.rs` | 4,553 lines | Future `ash-typeck` expression-checker round-2 split: expression forms, call/evidence lookup, control forms, and diagnostics modules. |
| `crates/ash-typeck/src/type_env/support.rs` and large `type_env/*` slices | 4,358 lines and related 2K+ slices | Future `ash-typeck::type_env` support/slice follow-up: support helper family extraction and per-feature submodule decomposition after TASK-1379 compatibility shell stabilizes. |
| `crates/ash-interp/src/execute.rs` | 3,331 lines | Future `ash-interp` workflow execution round-2 split: act/proc/workflow operation families, async control, and execution test ownership. |
| `crates/ash-interp/src/eval.rs` | 3,308 lines | Future `ash-interp` evaluation round-2 split: residual expression evaluators and eval test/support decomposition after TASK-1384 facade stabilization. |
| `crates/ash-cli/src/test_runner/synthesized/tests.rs` | 3,137 lines | Future synthesized-runner test split: contract/policy/obligation/law/small-world test modules. |
| `crates/ash-parser/src/parse_module.rs` | 3,103 lines | Future parser module-declaration split: declarations, definitions, imports/exports, and recovery diagnostics. |
| `crates/ash-typeck/src/lib.rs`, `crates/ash-engine/src/lib.rs`, `crates/ash-typeck/src/check_pattern.rs` | 2,536–3,030 lines | Future crate-root/check-pattern split packets once public compatibility surfaces are stable. |

## Per-crate final summary

| Crate | .rs files | >500 lines | >10KB | Largest by lines | Largest by bytes |
|---|---:|---:|---:|---|---|
| `ash-cli` | 62 | 11 | 22 | `crates/ash-cli/src/test_runner/synthesized/tests.rs` (3,137) | `crates/ash-cli/src/test_runner/synthesized/tests.rs` (120.8KB) |
| `ash-core` | 54 | 14 | 21 | `crates/ash-core/src/ast.rs` (1,715) | `crates/ash-core/src/semantic_summary.rs` (57.7KB) |
| `ash-diagnostic` | 1 | 0 | 0 | `crates/ash-diagnostic/src/lib.rs` (177) | `crates/ash-diagnostic/src/lib.rs` (5.2KB) |
| `ash-doc-tests` | 1 | 0 | 0 | `crates/ash-doc-tests/src/main.rs` (337) | `crates/ash-doc-tests/src/main.rs` (9.5KB) |
| `ash-engine` | 123 | 24 | 48 | `crates/ash-engine/src/module_loader.rs` (5,409) | `crates/ash-engine/src/module_loader.rs` (195.7KB) |
| `ash-interp` | 97 | 28 | 48 | `crates/ash-interp/src/execute.rs` (3,331) | `crates/ash-interp/src/eval.rs` (119.5KB) |
| `ash-lint` | 3 | 0 | 2 | `crates/ash-lint/src/rules.rs` (463) | `crates/ash-lint/src/rules.rs` (17.4KB) |
| `ash-lsp` | 3 | 0 | 2 | `crates/ash-lsp/src/main.rs` (478) | `crates/ash-lsp/src/main.rs` (15.2KB) |
| `ash-lsp-core` | 10 | 1 | 4 | `crates/ash-lsp-core/src/hover.rs` (525) | `crates/ash-lsp-core/src/hover.rs` (19.2KB) |
| `ash-macros` | 2 | 0 | 0 | `crates/ash-macros/src/lib.rs` (144) | `crates/ash-macros/src/lib.rs` (4.4KB) |
| `ash-mcp` | 3 | 0 | 1 | `crates/ash-mcp/src/lib.rs` (461) | `crates/ash-mcp/src/lib.rs` (15.8KB) |
| `ash-parser` | 121 | 29 | 42 | `crates/ash-parser/src/parse_module.rs` (3,103) | `crates/ash-parser/src/parse_module.rs` (96.8KB) |
| `ash-provenance` | 7 | 4 | 5 | `crates/ash-provenance/src/export.rs` (836) | `crates/ash-provenance/src/export.rs` (24.9KB) |
| `ash-repl` | 16 | 1 | 4 | `crates/ash-repl/src/ast.rs` (1,084) | `crates/ash-repl/src/ast.rs` (34.8KB) |
| `ash-std` | 1 | 0 | 0 | `std/src/lib.rs` (4) | `std/src/lib.rs` (246B) |
| `ash-typeck` | 203 | 64 | 108 | `crates/ash-typeck/src/check_expr/mod.rs` (4,553) | `crates/ash-typeck/src/check_expr/mod.rs` (167.1KB) |
| `ashgrove` | 30 | 11 | 18 | `crates/ashgrove/src/source.rs` (1,016) | `crates/ashgrove/src/source.rs` (33.0KB) |
| `spec_processor` | 20 | 0 | 0 | `apps/spec_processor/src/meta_validation.rs` (193) | `apps/spec_processor/src/pipeline.rs` (6.6KB) |

## Top 20 remaining files by line count

| Rank | File | Lines | Size |
|---:|---|---:|---:|
| 1 | `crates/ash-engine/src/module_loader.rs` | 5,409 | 195.7KB |
| 2 | `crates/ash-typeck/src/check_expr/mod.rs` | 4,553 | 167.1KB |
| 3 | `crates/ash-typeck/src/type_env/support.rs` | 4,358 | 152.9KB |
| 4 | `crates/ash-interp/src/execute.rs` | 3,331 | 119.3KB |
| 5 | `crates/ash-interp/src/eval.rs` | 3,308 | 119.5KB |
| 6 | `crates/ash-cli/src/test_runner/synthesized/tests.rs` | 3,137 | 120.8KB |
| 7 | `crates/ash-parser/src/parse_module.rs` | 3,103 | 96.8KB |
| 8 | `crates/ash-typeck/src/lib.rs` | 3,030 | 113.3KB |
| 9 | `crates/ash-engine/src/lib.rs` | 2,640 | 99.9KB |
| 10 | `crates/ash-typeck/src/check_pattern.rs` | 2,536 | 84.7KB |
| 11 | `crates/ash-interp/src/eval/tests.rs` | 2,500 | 73.9KB |
| 12 | `crates/ash-parser/src/surface.rs` | 2,459 | 71.9KB |
| 13 | `crates/ash-parser/src/lower.rs` | 2,443 | 83.9KB |
| 14 | `crates/ash-parser/src/import_resolver/tests.rs` | 2,361 | 76.3KB |
| 15 | `crates/ash-interp/src/runtime_state.rs` | 2,329 | 84.3KB |
| 16 | `crates/ash-typeck/src/type_env/associated_families_and_capabilities.rs` | 2,299 | 89.1KB |
| 17 | `crates/ash-typeck/src/type_env/imported_summaries_and_domains.rs` | 2,294 | 94.1KB |
| 18 | `crates/ash-typeck/src/type_env/surface_types_laws_and_prelude.rs` | 2,292 | 91.0KB |
| 19 | `crates/ash-typeck/src/type_env/type_function_lowering_and_propositions.rs` | 2,262 | 88.7KB |
| 20 | `crates/ash-typeck/src/type_env/type_functions.rs` | 2,254 | 89.0KB |

## Final verification evidence

All commands below passed during TASK-1386/TASK-1387 closeout:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test --workspace
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo doc --workspace --no-deps
git diff --check
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```
