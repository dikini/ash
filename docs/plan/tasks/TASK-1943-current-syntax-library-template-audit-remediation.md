# TASK-1943: Current-Syntax Library/Template Audit Remediation

**Status:** Complete
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Review and revise productive stdlib modules, examples, and template-like files to current target
syntax before building new app templates.

## Requirements

- Audit `std/src`, `examples`, `tests/std`, and template-like workflow/example assets.
- Classify files as current executable, current reference, historical/reference-only, or removed
  from productive paths.
- Revise productive libraries and examples to current syntax where required.
- Add parse/check/run or artifact assertions for files promoted to productive paths.

## TDD Steps

1. Add inventory checks or focused CLI/engine tests for productive library/example candidates.
2. Confirm stale syntax is detected.
3. Revise selected files to current syntax.
4. Re-run checks and record classification evidence.

## Completion Checklist

- [x] Productive library/example/template candidates are inventoried.
- [x] Historical/reference-only files are explicitly excluded from productive paths.
- [x] Required libraries are revised to current syntax.
- [x] Promoted productive files have executable or artifact gates.

## Evidence

- Added [AUDIT-199](../audits/AUDIT-199-current-syntax-library-template-inventory.md),
  classifying every `.ash` candidate under `std/src`, `examples`, `tests/std`, and
  `tests/workflows`.
- Added `phase199_current_syntax_audit`, a focused integration gate that fails if the audit omits a
  candidate file, uses an unsupported classification, or promotes a current/productive file without
  an executable or artifact gate.
- Repaired the productive `std/README.md` usage snippet so it no longer teaches stale
  `act ... with` syntax.
- Focused verification:
  `cargo test -p ash-cli --test phase199_current_syntax_audit -- --nocapture` and
  `cargo test -p ash-cli --test example_corpus_check --test stdlib_corpus_check -- --nocapture`.
