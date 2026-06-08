# TASK-1368: Synthetic tests — extract law nodes from AST

## Status: ✅ Complete

## Description

Test runner can iterate over `law` declarations in parsed modules.

## Requirements

1. Add `extract_laws` function to test runner
2. Return structured law metadata (name, params, proposition)
3. Handle both interface laws and module laws

## Acceptance Criteria

- [x] Laws extracted from interface definitions
- [x] Laws extracted from module files
- [x] Test passes
- [x] No regressions

## Verification

- `cargo test -p ash-cli test_runner::synthesized::tests::extract_laws_returns -- --nocapture` — 2 passed
- `cargo clippy -p ash-cli --all-targets --all-features -- -D warnings` — passed
- `cargo check -p ash-cli` — passed

## Completion Notes

- Added runner-facing `RunnerLawMetadata` and `LawScope` structures.
- Added `extract_laws(&ModuleFile)` to collect both module-scoped laws and interface-scoped laws from the parsed surface AST.
- Added source-level summaries for law name, parameters, owner/scope, and proposition.
- Wired extracted laws into `RunnerIntrospectionSnapshot` as `laws` for later synthetic-test generation.
- This task does not generate law test cases, execute laws, add CLI skip flags, implement caching, or add proof verification.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
- [TASK-1361](TASK-1361-parser-law-module-scope.md)
