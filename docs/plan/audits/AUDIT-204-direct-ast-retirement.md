---
id: audit.204.direct-ast-retirement
title: Direct AST Retirement Inventory
kind: audit
status: complete
authority: planning
owner: language-semantics
last_verified: 2026-07-28
---

# AUDIT-204: Direct AST Retirement Inventory

## Frozen inventory

The machine-readable [manifest](AUDIT-204-direct-ast-retirement.json) is the complete finite
inventory for this audit revision. It contains 309 explicit, repository-relative file and symbol
records, with no paths expressed as globs or generated feature classes.

- **Repository revision:** `c933985dbf57ca0d9524c9630a05f5606ffdfe3e`
- **Schema:** `direct-ast-retirement-audit/v1`
- **Sorted-entry digest:** `sha256:51b96a587220005fe87f712d8bc173e5c000bbe65440942ececf702f31ccfdbf`
- **Validator:** `python3 tools/docs/validate_direct_ast_retirement.py --root . --manifest docs/plan/audits/AUDIT-204-direct-ast-retirement.json --format json`

The digest is over every entry sorted by stable ID and canonical JSON encoding. A manifest row
names one existing file and one symbol or text locator. `current`, `historical`, and
`deferred_separate_project` describe present authority; they do not report implementation,
evidence, or target-spec parity.

## Catalogue decisions

| Inventory | Disposition and downstream owner |
|---|---|
| 34 direct-AST evaluator/module/export/consumer/test/benchmark/metadata records | Delete the evaluator and its direct tests under TASK-2040; TASK-2038 replaces the synthesized Core-expression oracle. |
| 22 public checked-CPS executor and direct-CPS test records | Move execution and validation behind the Engine-owned boundary under TASK-2037. |
| 135 Rust differential module, test, script, workflow, and finite corpus artifacts | Delete under TASK-2040. All 123 corpus files are listed individually in the JSON manifest. |
| 28 current and 2 historical direct/differential specifications, coverage, traceability, tasks, plans, and reference records | Relabel or replace current records, and preserve historical records without current authority, under TASK-2041; PLAN-203 is retained and reverified as the Engine-only authority. |
| 7 finite unsupported synthesized-contract shapes | TASK-2038 must emit the named fail-closed result until TASK-2035 supplies a target-authorized source wrapper. |
| 4 synthesized test-client records, 3 REPL records, 6 client-route records, and 5 workspace/dependency records | Replace with Engine-issued admitted-program requests under TASK-2037 through TASK-2042. |
| 37 Lean source, proof, documentation, example, build, and workflow records | Retain as `deferred_separate_project` with the external `lean-reference-project` handoff; do not delete or count them as a current Ash execution route. |
| 5 Lean-authority documents, 6 separate-project planning documents, 14 historical Lean task records, and 1 historical verification note | TASK-2041 relabels stale current-Ash authority or preserves explicitly historical records; the six planning documents are handed to the separate Lean project. |

## Deferred finite cases

The seven `deferred` rows are exact test-shape records, not a generated domain. Each names its
missing target-spec/source-wrapper obligation and required result:

- `test:contract_postcondition_without_executable_target_metadata`
- `test:contract_postcondition_without_structured_oracle_metadata`
- `test:contract_postcondition_with_unsupported_target_kind_defers`
- `test:contract_postcondition_with_missing_setup_defers`
- `test:contract_postcondition_explicit_finite_setup_defers`
- `test:contract_postcondition_unsupported_body_defers`
- `test:contract_postcondition_missing_exact_input_defers`

All seven remain explicit `deferred` outcomes; no local Core-expression or direct-AST evaluation
may make one pass.

## Lean handoff

Lean is explicitly deferred to `external:lean-reference-project`. Its retained records carry an
external project, owner, handoff, retained path, and prohibition on current Ash execution or
runtime-proof authority. They are not Phase-205 deletion candidates. Any future refinement claim
belongs to that separate project and must define and check its own refinement bridge.

## Audit boundary

This audit does not remove, migrate, prove, or test the catalogued behavior. Its downstream
completion condition is the sole canonical route:

```text
Surface Ash → checked Core → checked CPS → Engine executor → terminal envelope
```
