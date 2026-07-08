# AUDIT-199: Current-Syntax Library/Template Inventory

**Status:** Superseded by Phase 201
**Original task:** [TASK-1943](../tasks/TASK-1943-current-syntax-library-template-audit-remediation.md)
**Superseding audit:** [AUDIT-201](AUDIT-201-deprecated-functionality-removal.md)

## Phase 201 Supersession

The original Phase 199 inventory classified many pre-Phase-201 examples and stdlib tower files as
current executable artifacts. Phase 201 removed those deprecated productive paths, so preserving the
old table would now make false current-state claims.

Current authority for productive Ash source is:

- `examples/10-testing-helpers/testing_helpers.ash`
- `examples/11-process-channel-helpers/process_channel_helpers.ash`
- active `std/src` files accepted by `crates/ash-cli/tests/stdlib_corpus_check.rs`
- app templates accepted by the Phase 199/201 template and removed-form gates

Use [AUDIT-201](AUDIT-201-deprecated-functionality-removal.md) and the current corpus gates for
Phase 201 source inventory. Historical Phase 199 classifications are no longer current guidance and
must not be used to reintroduce deleted examples, removed stdlib carrier modules, or deprecated Ash
forms.
