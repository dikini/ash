# Rust File Size Audit

Phase 137 baseline generated from the current `phase-137-module-size` worktree using:

```bash
python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-size-audit.md
python3 tools/dev/rust_file_size_report.py --json > /tmp/phase137-size-audit.json
python3 tools/dev/rust_file_size_report.py --tests-only > /tmp/phase137-tests-only.md
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```

The audit script derives workspace package ownership from `cargo metadata --format-version 1 --no-deps`, excludes `.git/`, `target/`, and `.worktrees/` paths before counting Rust source files, and provides a Phase 137 regression guard for oversized-file counts and largest-file line/byte maxima without treating added split modules as regressions.

# Rust File Size Report

- Workspace crates scanned: 18
- Rust files scanned: 663
- Rust files larger than 500 lines: 165
- Rust files larger than 10.0KB: 284
- Ignored directories: `.git/`, `.worktrees/`, `target/`

## Per-crate summary

| Crate | .rs files | >500 lines | >10KB | Largest by lines | Largest by bytes |
|---|---:|---:|---:|---|---|
| `ash-cli` | 51 | 8 | 17 | `crates/ash-cli/src/test_runner/synthesized.rs` (7,524) | `crates/ash-cli/src/test_runner/synthesized.rs` (283.6KB) |
| `ash-core` | 54 | 14 | 21 | `crates/ash-core/src/ast.rs` (1,715) | `crates/ash-core/src/semantic_summary.rs` (57.7KB) |
| `ash-diagnostic` | 1 | 0 | 0 | `crates/ash-diagnostic/src/lib.rs` (177) | `crates/ash-diagnostic/src/lib.rs` (5.2KB) |
| `ash-doc-tests` | 1 | 0 | 0 | `crates/ash-doc-tests/src/main.rs` (337) | `crates/ash-doc-tests/src/main.rs` (9.5KB) |
| `ash-engine` | 111 | 21 | 45 | `crates/ash-engine/src/module_loader.rs` (8,248) | `crates/ash-engine/src/module_loader.rs` (294.7KB) |
| `ash-interp` | 81 | 28 | 47 | `crates/ash-interp/src/eval.rs` (6,545) | `crates/ash-interp/src/eval.rs` (224.8KB) |
| `ash-lint` | 2 | 1 | 1 | `crates/ash-lint/src/lib.rs` (831) | `crates/ash-lint/src/lib.rs` (29.0KB) |
| `ash-lsp` | 1 | 1 | 1 | `crates/ash-lsp/src/main.rs` (897) | `crates/ash-lsp/src/main.rs` (29.8KB) |
| `ash-lsp-core` | 10 | 1 | 4 | `crates/ash-lsp-core/src/hover.rs` (525) | `crates/ash-lsp-core/src/hover.rs` (19.2KB) |
| `ash-macros` | 2 | 0 | 0 | `crates/ash-macros/src/lib.rs` (144) | `crates/ash-macros/src/lib.rs` (4.4KB) |
| `ash-mcp` | 2 | 1 | 1 | `crates/ash-mcp/src/lib.rs` (663) | `crates/ash-mcp/src/lib.rs` (21.9KB) |
| `ash-parser` | 95 | 22 | 33 | `crates/ash-parser/src/surface.rs` (4,722) | `crates/ash-parser/src/lower.rs` (146.0KB) |
| `ash-provenance` | 7 | 4 | 5 | `crates/ash-provenance/src/export.rs` (836) | `crates/ash-provenance/src/export.rs` (24.9KB) |
| `ash-repl` | 13 | 2 | 4 | `crates/ash-repl/src/ast.rs` (1,084) | `crates/ash-repl/src/ast.rs` (34.8KB) |
| `ash-std` | 1 | 0 | 0 | `std/src/lib.rs` (4) | `std/src/lib.rs` (246B) |
| `ash-typeck` | 188 | 54 | 94 | `crates/ash-typeck/src/type_env.rs` (20,935) | `crates/ash-typeck/src/type_env.rs` (807.1KB) |
| `ashgrove` | 23 | 8 | 11 | `crates/ashgrove/src/lib.rs` (4,694) | `crates/ashgrove/src/lib.rs` (151.7KB) |
| `spec_processor` | 20 | 0 | 0 | `apps/spec_processor/src/meta_validation.rs` (193) | `apps/spec_processor/src/pipeline.rs` (6.6KB) |

## Top 20 files by line count

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
| 11 | `crates/ash-engine/src/lib.rs` | 3,416 | 127.1KB |
| 12 | `crates/ash-typeck/src/lib.rs` | 3,388 | 127.5KB |
| 13 | `crates/ash-parser/src/import_resolver.rs` | 3,305 | 116.9KB |
| 14 | `crates/ash-parser/src/parse_expr.rs` | 2,696 | 86.5KB |
| 15 | `crates/ash-typeck/src/check_pattern.rs` | 2,536 | 84.7KB |
| 16 | `crates/ash-interp/src/runtime_state.rs` | 2,530 | 90.6KB |
| 17 | `crates/ash-parser/src/parse_workflow.rs` | 2,423 | 75.7KB |
| 18 | `crates/ash-parser/src/lift.rs` | 2,075 | 75.4KB |
| 19 | `crates/ash-typeck/src/normalizer.rs` | 1,933 | 68.4KB |
| 20 | `crates/ash-typeck/src/runtime_verification.rs` | 1,835 | 57.0KB |
