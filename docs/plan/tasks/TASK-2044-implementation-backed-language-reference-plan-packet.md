# TASK-2044: Implementation-Backed Language Reference Plan Packet

**Status:** Complete
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Audit:** [AUDIT-206](../audits/AUDIT-206-implementation-backed-language-reference.md)

## Description

Create the evidence-led planning packet, census, task decomposition, and navigation records for a
new manual rooted at `docs/reference/language/`. This task creates no language-reference pages.

## Requirements

- Establish live code and tests as the source of truth and record the parser-to-runtime status
  lattice.
- Inventory active parser domains, fixture-bounded execution, internal-only carriers, and removed
  forms without presenting removed syntax as source examples.
- Record `docs/reference/` versus top-level `reference/` placement and evidence conflicts.
- Define EBNF/sequent authoring and validation obligations for follow-on tasks.

## Handoffs and dependencies

- **Consumes:** `parse_module::module_file`, parser/typeck/lowering/Engine tests, the machine
  readable corpus/traceability records, and existing documentation policy.
- **Produces:** PLAN-206, AUDIT-206, TASK-2045 through TASK-2054, PLAN-INDEX and changelog
  routing.
- **Downstream owner:** TASK-2045 owns the placement decision and manual skeleton.
- **Non-goals:** implementation changes, a reference page, source/spec changes, or legacy-corpus
  migration.

## TDD and verification steps

1. Build a failing evidence checklist: each proposed topic needs grammar/static/lowering/runtime
   status, code/test evidence, conflicts, destination, and owner.
2. Populate the census from targeted current source and tests, treating absent routes as gaps.
3. Create the plan and standalone task records with exact documentation destinations.
4. Run `git diff --check` and the planning packet's available documentation gates; TASK-2054
   re-runs the complete documentation closeout after manual pages and its fence validator exist.

## Completion checklist

- [x] PLAN-206 and AUDIT-206 exist and distinguish implementation, evidence, and parity.
- [x] The census records active, partial, internal-only, and excluded forms.
- [x] All follow-on manual tasks have self-contained files and dependency handoffs.
- [x] Removed forms are prohibited from current-reference examples.
- [x] PLAN-INDEX and CHANGELOG contain this planning task.

## Verification evidence

The audit retried Rust language-server activation first; the workspace started no Rust server, so
research used the documented baseline-only path (targeted `rg`/source reads and executable tests).

2026-07-29 verification at the audited worktree revision:

- `cargo test -p ash-parser --test fn_parser_tests contracts_and_types` — 6 passed; verifies
  current function `requires:`/`ensures:` parsing.
- `cargo test -p ash-parser --test stdlib_parsing test_runtime_args_usage_surface` — 1 passed;
  verifies source `capability Args` in a type position.
- `cargo test -p ash-parser --test task_1809_computation_row_parser` — 7 passed; verifies mixed
  row families, tails, whole-row variables, and operation separators.
- `cargo test -p ash-typeck --test task_2013_handler_row_typing
  task_2013_every_nonempty_or_open_residual_keeps_resume_affine` — 1 passed; verifies the
  non-operation/open-tail residual boundary.
- `cargo test -p ash-parser --test task_708_fail_with_error` — 8 passed; verifies source failure,
  scoped handler parsing, and Core carrier lowering.
- `cargo test -p ash-engine --test function_contracts_integration` — 32 passed; verifies the
  selected Engine contract/obligation evidence used by the audit.
- `python3 tools/docs/validate_orientation_indexes.py --self-test` — passed.
- `bash scripts/check-docs-gate.sh` — passed; it checked 1,529 changed-Markdown links and
  semantic traceability.
- `git diff --check` — passed after the packet edits.
