# Ash Examples

This directory contains current target-Ash examples that are expected to pass `ash check`.

Phase 201 removed historical Ash source examples from the repository. New examples must use
target Ash syntax only and must not reintroduce removed workflow, capability, or tower carrier
forms.

## Productive Examples

- `10-testing-helpers/testing_helpers.ash`: testing helper imports for assertions, property
  evidence, law evidence, counterexamples, coverage, mutation, flake quarantine, provider evidence,
  deterministic provider-profile fixtures, and common test cases.
- `11-process-channel-helpers/process_channel_helpers.ash`: process/channel helper imports for
  spawn/join plans, bounded worker pools, channel loops, cancellation cleanup, sendability guards,
  channel diagnostics, and process traces.

## App Templates

Generated app skeletons live under [templates/apps](../templates/apps/README.md). They are also
target-Ash examples and are validated through template instantiation and `ash check` gates.

## Verification

Run the example corpus gate with:

```bash
cargo test -p ash-cli --test example_corpus_check -- --nocapture
```
