# TASK-765: Canonicalize Small Examples

## Status: 📝 Planned

## Description

Rewrite the small control-flow and IO examples to current Ash syntax so they become checkable conformance examples instead of stale historical syntax samples.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-005](../../spec/SPEC-005-CLI.md)

## Dependencies

- 📝 TASK-760: CLI Corpus Baseline Harness
- 📝 TASK-764: Parser Comments and Diagnostics

## Requirements

1. Canonicalize `examples/02-control-flow/*.ash` to parser-supported workflow syntax.
2. Canonicalize `examples/03-io/*.ash` or reduce them to honest checkable demonstrations.
3. Preserve teaching intent where practical; otherwise add comments explaining current limitations.
4. Add repaired files to the expected-pass example corpus.
5. Do not introduce unsupported syntax just to match historical docs.

## Files

- Modify: `examples/02-control-flow/01-conditionals.ash`
- Modify: `examples/02-control-flow/02-foreach.ash`
- Modify: `examples/02-control-flow/03-sequential.ash`
- Modify: `examples/02-control-flow/04-sequential.ash`
- Modify: `examples/03-io/directory_listing.ash`
- Modify: `examples/03-io/file_read_write.ash`
- Modify: `examples/03-io/path_operations.ash`
- Test: `crates/ash-cli/tests/example_corpus_check.rs`

## TDD Steps

1. Add the target files as expected-pass in the example corpus test and watch them fail.
2. Repair examples incrementally with `ash check <file>` after each file.
3. Keep examples small and current-syntax focused.
4. Update corpus expected-pass list.

## Verification Checklist

- [ ] `cargo run -q -p ash-cli -- check examples/02-control-flow/01-conditionals.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check examples/02-control-flow/02-foreach.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check examples/02-control-flow/03-sequential.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check examples/02-control-flow/04-sequential.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check examples/03-io/directory_listing.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check examples/03-io/file_read_write.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check examples/03-io/path_operations.ash` passes.
- [ ] Example corpus test passes for the expanded expected-pass set.
- [ ] Independent review confirms examples teach current syntax.
