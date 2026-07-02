# PLAN-178: Source-to-Core Row Lowering Bridge

**Status:** ✅ Complete (9/9 tasks complete; source-to-Core explicit row bridge closed with review remediation)
**Spec:** [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md); [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-020: Computation Row Taxonomy](../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md); [NOTE-021: Row, Callable, Where, and Fact Syntax](../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
**Depends on:** [PLAN-177: Target Ash Row Syntax and Core/CPS Alignment](PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md), especially TASK-1814 through TASK-1816.
**Task range:** TASK-1817 through TASK-1825.

## Goal

Bridge Phase 177's parsed and validated target row syntax into the source-to-typechecker/Core lowering path. Phase 178 must thread inline rows and `where row` rows into function/type summaries and Core callable rows while preserving the boundary that rows are requirements, not provider/admission/handler authority.

## Rationale

Phase 177 deliberately stopped short of a full source-to-Core row bridge. Its closeout records the remaining boundary:

```text
target row syntax
  -> parsed surface row carriers
  -> surface validation, impl-qualified identity checks, and import-signature retention
  -> current rowless typechecker `Type::Fn` conversion boundary
```

Core and CPS can now represent supported row families independently, and parser/typechecker validation can preserve row syntax and reject malformed row spellings. The next integration step is to remove the validation-only gap by carrying parsed callable rows into the lowered summaries and Core callable types that later phases can execute or discharge.

Phase 178 should make this bounded vertical slice true:

```text
parsed surface callable row
  -> validated source row summary
  -> lowered function/type summary with row
  -> CoreType::Function row / Core callable row
  -> Core typecheck and public summary evidence
```

It should not implement row inference, provider/admission runtime, handler execution, or authority discharge.

## Scope

Phase 178 owns:

- auditing the current source-to-typechecker/Core row-loss boundary, especially rowless `Type::Fn` conversion and public callable summaries;
- adding a row-bearing source callable summary shape for parsed inline rows and expanded `where row` rows;
- threading parsed callable rows through module validation, imports/exports, and typechecker-facing callable signatures where the current pipeline already transports function types;
- lowering supported Phase 177 source row families into `CoreRow` and `CoreType::Function { row, .. }`;
- preserving imported public callable rows through summaries without making imported rows executable authority;
- adding negative tests proving row requirements do not install providers, admission facts, handlers, host hooks, resource ownership, roles, or workflow authority;
- adding end-to-end parser -> engine/typecheck -> Core row preservation tests.

## Non-goals

- No row-polymorphic inference or solving beyond explicit rows and already validated row variables.
- No provider/admission runtime wiring, role admission, handler installation, or host/FFI implementation.
- No target handler execution surface.
- No broad stdlib/example corpus migration.
- No fact/evidence declaration body lowering beyond preserving evidence row references already accepted by Phase 177.
- No declaration that target Ash is fully implemented.

## Decision gates

| Gate | Question | Owner task | Default decision |
|---|---|---|---|
| D1 | Where exactly do Phase 177 source rows disappear when converting to typechecker/Core surfaces? | TASK-1818 | Audit before changing carriers; patch downstream task scope if the live path differs. |
| D2 | What is the minimal row-bearing source callable summary shape? | TASK-1819 | Add row metadata beside existing callable type summaries instead of replacing unrelated type infrastructure. |
| D3 | How should explicit source rows enter typechecker-facing function/type summaries without broad row inference? | TASK-1820 | Thread explicit rows only; rowless callables keep existing behavior. |
| D4 | How should source rows lower to Core rows? | TASK-1821 | Lower supported Phase 177 row families to `CoreRow`; unsupported row variables or families fail closed with precise diagnostics. |
| D5 | What proves row requirements remain authority-neutral? | TASK-1822 | Negative tests for providers, admission, handlers, host hooks, resources, roles, and workflow authority. |
| D6 | What proves the end-to-end bridge exists? | TASK-1823 | Parser -> engine/typecheck -> Core row preservation tests that inspect actual lowered summaries/Core rows. |
| D7 | What docs/status updates are needed to avoid overclaiming target-Ash completion? | TASK-1824 | Reconcile only Phase 178 surfaces and keep future work explicit. |
| D8 | What broad gates and review are required before closeout? | TASK-1825 | Full affected crate tests, workspace checks, docs gates, and independent review focused on row loss and authority leakage. |

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1817](tasks/TASK-1817-phase-178-plan-packet.md) | Create the Phase 178 source-to-Core row lowering bridge packet | ✅ Complete |
| [TASK-1818](tasks/TASK-1818-source-to-core-row-loss-audit.md) | Audit source-to-typechecker/Core row-loss boundaries | ✅ Complete |
| [TASK-1819](tasks/TASK-1819-row-bearing-callable-summary-carriers.md) | Add row-bearing callable summary carriers | ✅ Complete |
| [TASK-1820](tasks/TASK-1820-thread-parsed-rows-into-type-summaries.md) | Thread parsed rows into function/type summaries | ✅ Complete |
| [TASK-1821](tasks/TASK-1821-lower-source-rows-to-core-callable-rows.md) | Lower source rows into Core callable rows | ✅ Complete |
| [TASK-1822](tasks/TASK-1822-row-requirements-authority-neutrality-tests.md) | Prove row requirements do not install authority | ✅ Complete |
| [TASK-1823](tasks/TASK-1823-parser-engine-typecheck-core-row-preservation.md) | Add parser -> engine/typecheck -> Core row preservation tests | ✅ Complete |
| [TASK-1824](tasks/TASK-1824-phase-178-docs-spec-reconciliation.md) | Reconcile docs/spec/status for Phase 178 boundaries | ✅ Complete |
| [TASK-1825](tasks/TASK-1825-phase-178-closeout.md) | Close out Phase 178 with gates and review | ✅ Complete |

## Implementation order

1. TASK-1818 audits the live row-loss path and records owner files/test seams.
2. TASK-1819 adds the row-bearing summary/carrier model without changing behavior broadly.
3. TASK-1820 threads explicit parsed rows through function/type summary paths and imports/exports.
4. TASK-1821 lowers supported source rows into Core callable rows.
5. TASK-1822 adds negative authority-neutrality tests and patches leaks found by those tests.
6. TASK-1823 adds end-to-end parser/engine/typecheck/Core preservation tests.
7. TASK-1824 reconciles plan/spec/docs/changelog wording after implementation outcomes are known.
8. TASK-1825 runs broad gates, obtains independent review, fixes findings, and closes the phase.

## Acceptance criteria

- [x] The current source-to-typechecker/Core row-loss boundary is audited and documented before implementation.
- [x] Parsed inline callable rows are represented in row-bearing function/type summaries.
- [x] Parsed `where row` callable rows are represented in row-bearing function/type summaries.
- [x] Imported/exported callable summaries preserve explicit row requirements.
- [x] Supported Phase 177 row item families lower to `CoreRow` and Core callable rows.
- [x] Rowless functions keep existing behavior unless they have explicit row syntax.
- [x] Row requirements remain authority-neutral and do not install providers, admission, handlers, host hooks, resources, roles, or workflow authority.
- [x] End-to-end tests prove parser -> engine/typecheck -> Core row preservation.
- [x] Row-polymorphic inference and provider/admission runtime wiring remain explicitly out of scope.
- [x] PLAN-INDEX, task files, docs/spec references, and CHANGELOG agree on Phase 178 status.

## Verification baseline

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-engine
cargo test -p ash-typeck
cargo test -p ash-core
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Focused tasks should add narrower crate/test commands for each seam. Closeout must run the full baseline unless the task records a user-approved deferral.

## Expected follow-on after Phase 178

If Phase 178 closes cleanly, the next plausible packets are row-polymorphic inference, provider/admission runtime wiring, target handler execution surface, fact/evidence declaration lowering, or stdlib/example corpus migration onto target row syntax. Those should remain separate phases.

## Completion evidence

- TASK-1817 created and registered this planning packet, task files TASK-1817 through TASK-1825, PLAN-INDEX entries, and a CHANGELOG planning entry.
- TASK-1818 created `docs/audit/PHASE-178-source-to-core-row-loss.md`, mapping parser, engine, typechecker, semantic-summary, and Core row-loss boundaries before implementation.
- TASK-1819 added engine callable row requirement metadata for explicit inline rows and expanded `where row` rows while leaving rowless callables unchanged.
- TASK-1820 threaded explicit callable row requirements through local and imported workflow summaries.
- TASK-1821 lowered supported explicit source rows into `CoreType::Function` row metadata while preserving rowless callable compatibility.
- TASK-1822 added local and imported authority-neutrality regressions for provider, admission, resource, handler, workflow-summary, and host-hook boundaries.
- TASK-1823 added parser -> engine/typecheck -> Core row preservation regressions for inline rows, `where row`, open tails, imports, and rowless defaults.
- TASK-1824 reconciled Phase 178 docs/spec boundaries without overclaiming row inference or runtime authority wiring.
- TASK-1825 completed broad verification and independent review remediation.
