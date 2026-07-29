# TASK-2053: Language Reference for Public Stdlib, Diagnostics, and Limitations

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-017, LANG-018.

## Description

Document only source-visible public standard-library surfaces with checked evidence, plus a
diagnostic/limitation inventory that prevents static declarations from being mistaken for runnable
Engine features.

## Requirements

- Create `docs/reference/language/library/index.md`,
  `docs/reference/language/library/public-stdlib-and-limitations.md`, and the fixed diagnostic
  page `docs/reference/language/library/diagnostics-and-limitations.md`.
- Build a module/API inventory from `std/src/**`, export/import code, checked corpus tests, and
  selected Engine e2e tests. State module-specific runtime evidence rather than blanket support.
- Document parser/typecheck/admission/terminal diagnostic categories with exact tests and status.
- Preserve historical Act/Proc/Workflow stdlib material as excluded old links; do not turn it into
  a current library API.

## Handoffs and dependencies

- **Consumes:** `std/src/**`, module loader, parser/typeck/CLI diagnostic paths.
- **Evidence:** `cargo test -p ash-cli --test stdlib_corpus_check`, `cargo test -p ash-parser
  --test stdlib_parsing`, `cargo test -p ash-engine --test task_968_installed_stdlib`, `--test
  json_stdlib_e2e`, `cargo test -p ash-cli --test check_parse_diagnostics`.
- **Produces:** a limitation inventory for TASK-2054's stale-claim sweep.
- **Non-goals:** a guarantee every declared stdlib name executes, historical tower modules, or
  undocumented internal helpers.

## TDD and verification steps

1. Start from a module/API/evidence matrix; entries with import-only proof must fail the
   “runnable” column.
2. Verify corpus/import and selected Engine e2e results; classify unsupported modules explicitly.
3. Validate examples, links, and representative grammar/sequent fences where applicable.

## Completion checklist

- [ ] Public stdlib claims have module-specific evidence and limitations.
- [ ] Diagnostic status distinguishes parse/static/admission/runtime output.
- [ ] Removed forms never appear as current examples.
- [ ] Indexes, changelog, and PLAN-INDEX are updated.
