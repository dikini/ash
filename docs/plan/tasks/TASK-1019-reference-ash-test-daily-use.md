# TASK-1019: Reference Ash Test Daily Use

## Status: ✅ Complete

## Description

Add a focused top-level reference page for daily `ash test` use, preserving the `reference/` metadata, tone, example policy, and Alpha limitation boundaries.

## Specification Reference

- [SPEC-005: CLI](../../spec/SPEC-005-CLI.md)
- [SPEC-075: Reference Slice 2 Runtime, Toolchain, and Maintenance](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-077: Ash Test Runner Synthesized and Small-World Completion](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [PLAN-INDEX Phase 76B](../PLAN-INDEX.md)
- [PLAN-INDEX Phase 130](../PLAN-INDEX.md)

## Dependencies

- TASK-509 through TASK-515 for the current `ash test` runner surface.
- TASK-1010 and TASK-1011 for the Phase 76B structured-snapshot boundary and final remediation.
- TASK-995 for the existing CLI command-map reference surface.

## Requirements

1. Create `reference/tools/test.md` with SPEC-071-compatible frontmatter.
2. Cover authored discovery, metadata directives, kind/tag filtering, output formats, fail-fast, timeout, quiet/color/verbosity, property and small-world knobs, and synthesized controls.
3. State that ordinary CLI raw-source synthesized compatibility rows are deferred skips, not full live checked/lowered synthesized execution.
4. Link the new page from `reference/tools/README.md`, `reference/tools/cli.md`, and `reference/INDEX.md`.
5. Update `CHANGELOG.md` under `[Unreleased]`.
6. Run the focused reference verification commands.

## Work Steps

1. Inspect existing `reference/` style and frontmatter pages.
2. Inspect `ash test --help` plus `crates/ash-cli/src/commands/test.rs` and `crates/ash-cli/src/test_runner/**`.
3. Write concise, example-led daily-use prose without unsupported syntax.
4. Update indexes and changelog.
5. Run verification and report any sandbox-specific command adjustments.

## Verification

```yaml
strictness: focused
commands:
  - CARGO_NET_OFFLINE=true CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo run -p ash-cli -- test --help
  - python3 tools/reference/check_frontmatter.py --pilot
  - git diff --check
checklist:
  - [x] Daily-use `ash test` page added.
  - [x] Existing reference link surfaces updated.
  - [x] Phase 76B synthesized limitation stated explicitly.
  - [x] Changelog updated.
```

## Completion Notes

Completed on 2026-06-03. The initial live-help command failed in this sandbox because the configured `sccache` wrapper returned `Operation not permitted` and then an unrestricted rerun attempted network access. The successful help verification used the offline Cargo/network and rustc-wrapper overrides recorded above.
