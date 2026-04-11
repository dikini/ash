# TASK-509: Ash Test Runner Substrate

## Status: ✅ Complete

## Description

Add the first `ash test` runner substrate to the CLI: command surface, authored test discovery roots, canonical suite/result reporting, and the shared runner-side execution model entry point.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Requirements

1. Add an `ash test` CLI command.
2. Discover authored Ash tests from the agreed repository roots.
3. Define the canonical suite/result envelope used by later tasks.
4. Emit both human and JSON output formats.
5. Keep synthesized tests out of the default discovery path.

## Likely Files

- Modify: `crates/ash-cli/src/main.rs`
- Create: `crates/ash-cli/src/commands/test.rs`
- Create/Modify: output/reporting modules under `crates/ash-cli/src/`
- Add tests under `crates/ash-cli/tests/`

## TDD Steps

### Red
- Add CLI tests showing `ash test --help` and empty-suite discovery behavior are not yet implemented.

### Green
- Implement command parsing, discovery roots, and suite/result reporting.

## Completion Checklist

- [x] `ash test` command exists
- [x] authored test discovery roots implemented
- [x] human output implemented
- [x] JSON output implemented
- [x] canonical suite/result model exists
- [x] default run excludes synthesized tests
