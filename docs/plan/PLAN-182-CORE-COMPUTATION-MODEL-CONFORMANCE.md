# PLAN-182: Core Computation Model Conformance

**Status:** Complete (10/10 tasks complete)
**Spec:** [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md); [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-019: Target Ash Convergence Plan](../notes/NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md); [NOTE-020: Computation Row Taxonomy](../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md); [NOTE-021: Row Callable Where and Fact Syntax](../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md); [PLAN-181: Legacy Authority Vocabulary Audit](PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md)
**Depends on:** [PLAN-181: Legacy Authority Vocabulary Audit](PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md)
**Task range:** TASK-1837 through TASK-1846.

## Goal

Make the target Core computation model explicit and executable for the first bounded slice: Core Ash is the checked direct-style language, computation rows are requirement metadata, `fn` is the primary computation unit, and target `do { ... }` is direct sequencing sugar rather than an `Act`, `Proc`, or `Workflow` mode.

## Scope

- Add the Phase 182 plan/task packet.
- Audit current surface, lowering, Core, and typecheck boundaries for one-core-model gaps.
- Reconcile target docs so they do not imply `Act`, `Proc`, or `Workflow` are target semantic foundations.
- Implement target ambient `do { ... }` parsing and typechecking as direct-style sequencing over ordinary expression checking.
- Preserve explicit computation rows through parser, engine summaries, and Core callable rows.
- Add end-to-end tests proving target `fn` + row + `do { ... }` reaches Core metadata without granting authority.

## Non-goals

- No handler/provider execution semantics beyond existing explicit row admission checks.
- No broad standard-library or example migration.
- No implementation of process/app/workflow runtime features.
- No compatibility-driven behavior that creates a second target semantic path.

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1837](tasks/TASK-1837-core-computation-plan-packet.md) | Create the Phase 182 plan packet | Complete |
| [TASK-1838](tasks/TASK-1838-core-computation-boundary-audit.md) | Audit Core computation boundaries | Complete |
| [TASK-1839](tasks/TASK-1839-core-computation-spec-reconciliation.md) | Reconcile target Core computation specs | Complete |
| [TASK-1840](tasks/TASK-1840-primary-fn-computation-unit.md) | Prove `fn` as primary row-bearing computation unit | Complete |
| [TASK-1841](tasks/TASK-1841-ambient-do-sequencing-sugar.md) | Implement target `do { ... }` sequencing sugar | Complete |
| [TASK-1842](tasks/TASK-1842-row-requirements-direct-style-preservation.md) | Preserve row requirements through direct-style Core metadata | Complete |
| [TASK-1843](tasks/TASK-1843-demote-tower-language-in-target-docs.md) | Demote tower language in target docs | Complete |
| [TASK-1844](tasks/TASK-1844-core-computation-cross-boundary-fixture.md) | Add canonical cross-boundary target fixture | Complete |
| [TASK-1845](tasks/TASK-1845-phase-182-consistency-review.md) | Review Phase 182 consistency and cross-references | Complete |
| [TASK-1846](tasks/TASK-1846-core-computation-closeout.md) | Close out Phase 182 | Complete |

## Acceptance criteria

- [x] `do { ... }` parses as target ambient sequencing and does not name `Act`, `Proc`, or `Workflow`.
- [x] Target ambient `do { ... }` typechecks by checking direct-style `let`, bind-like sequencing, and final `return`.
- [x] A row-bearing `fn` using target `do { ... }` preserves its explicit row into engine callable summaries and Core callable metadata.
- [x] Tests prove `do { ... }` does not install authority and does not require legacy tower constructors.
- [x] Target specs and indexes route Core computation model work through `fn`, Core Ash, rows, and direct-style checking.
- [x] `CHANGELOG.md` records the phase.
- [x] Required docs, Rust, and changed-crate gates pass.

## Verification

```bash
cargo test -p ash-parser target_ambient_do
cargo test -p ash-typeck target_ambient
cargo test -p ash-engine --test task_1844_core_computation_conformance
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
cargo fmt --check
git diff --check
```
