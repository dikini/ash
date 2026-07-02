# PLAN-177: Target Ash Row Syntax and Core/CPS Alignment

**Status:** ✅ Complete (11/11 tasks complete; bounded parser/validation, operation identity, Core/CPS taxonomy, cross-boundary evidence, closeout, and row syntax review remediation complete)
**Spec:** [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md); [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-020: Computation Row Taxonomy](../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md); [NOTE-021: Row, Callable, Where, and Fact Syntax](../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md); [NOTE-022: Effects as Interfaces](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md); [NOTE-023: Handler Surface](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
**Depends on:** Phase 176 closeout and interphase TASK-1803 through TASK-1805 status reconciliation.
**Task range:** TASK-1806 through TASK-1816.

## Goal

Start implementing target Ash by connecting the source-facing computation-row/effect syntax to parser and validation carriers, while aligning Core and CPS row carriers enough to avoid silent loss once a later source-to-Core row bridge exists. Phase 177 is the first narrow implementation packet after target cleanup: it accepts and validates the target row surface, preserves impl-qualified operation identity where proven, and aligns Core/CPS row taxonomy enough that rows are requirements with kind-specific families rather than a legacy capability-only effect list.

## Rationale

Phases 167 through 176 established the target surface, macro/lowering boundaries, Core/CPS substrate, and cleanup state. The remaining target-Ash gap is no longer conceptual foundation; it is integration.

Current code already has important pieces:

- `crates/ash-parser/src/surface.rs` carries source AST, callable summaries, macros, notation, and existing `where` surfaces.
- `crates/ash-core/src/core_ash.rs` has `CoreRow`, `CoreRowItem`, row-bearing function/continuation types, and target Core carriers.
- `crates/ash-core/src/cps.rs` has `EffectRow`, `EffectItem`, `EffectOp`, handlers, and continuation metadata, but the item taxonomy is narrower than Core's target row taxonomy.
- `crates/ash-core/src/core_ash_lower.rs` bridges Core rows to CPS effect rows and is therefore the key lossiness boundary.

Phase 177 should make one bounded integration slice true, with the source-to-typechecker/Core row bridge still explicitly validation-only:

```text
target row syntax
  -> parsed surface row carriers
  -> surface validation, impl-qualified identity checks, and import-signature retention
  -> current rowless typechecker `Type::Fn` conversion boundary

Core row items
  -> CPS row/effect carriers
  -> diagnostics and regression evidence
```

## Scope

Phase 177 owns:

- auditing live row syntax, type annotation, `where`, Core row, CPS effect row, and lowering seams;
- reconciling NOTE-021/NOTE-022/NOTE-023/NOTE-025 pre-spec deltas into task-local implementation decisions without overclaiming full target completion;
- parsing and preserving inline callable rows and expanded `where row { ... }` blocks for function declarations and callable type positions;
- enforcing duplicate-row and row-tail validation rules;
- representing source row items for operation, resource, role, policy, channel, process, failure, evidence, group, and row tail forms;
- resolving operation row identity to impl-type-qualified forms such as `F::read` and `PosixFs::read` where the live name-resolution substrate can prove them;
- aligning Core and CPS row item taxonomies so lowering preserves supported families or fails closed with a precise diagnostic;
- adding cross-boundary tests that prove parsed row items survive parser/engine retention and validation without granting authority, plus independent Core-to-CPS tests proving supported Core row families lower without silent row loss.

## Non-goals

- No full target handler execution surface beyond row/identity carriers required by this packet.
- No provider/admission runtime, authority granting, or host/FFI implementation.
- No full row-polymorphic type inference. Phase 177 may parse and carry tails, but solving remains bounded to explicit or already-supported contexts.
- No arbitrary proof/evidence discharge machinery beyond preserving evidence row requirements.
- No broad migration of all stdlib/examples to target rows.
- No declaration of `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, or `SPEC-098c` as implemented MVP.

## Decision gates

| Gate | Question | Owner task | Default decision |
|---|---|---|---|
| D1 | What live parser/type/Core/CPS row surfaces already exist, and where is row information currently lost? | TASK-1807 | Audit first; patch downstream task scope if a presumed seam differs. |
| D2 | Which pre-spec deltas are implementable now without inventing new foundation? | TASK-1808 | Implement only NOTE-021/022/023/025 deltas needed for this row/identity slice. |
| D3 | What parsed surface carriers should represent inline and expanded row syntax? | TASK-1809 | Add source-preserving carriers with spans and no authority semantics. |
| D4 | What operation identity is safe to resolve in Phase 177? | TASK-1810 | Impl-qualified identities only when proven; ambiguous/unresolved impl-qualified identity fails closed. Lowercase/source-path operation rows remain unresolved requirement metadata in this validation-only slice. |
| D5 | Which validation rules must reject before Core lowering? | TASK-1811 | Duplicate row spelling, misplaced tails, unsupported row item forms, and raw predicate bodies fail closed. |
| D6 | How should Core row item names/taxonomy change without breaking existing implemented phases? | TASK-1812 | Add compatibility aliases or conversion helpers only where they prevent churn and remain honest. |
| D7 | Can CPS rows preserve every supported Core row family? | TASK-1813 | Preserve supported families; fail closed for unsupported families rather than silently dropping them. |
| D8 | What proves the bounded slice works without overclaiming source-to-Core lowering? | TASK-1814 | Parser/engine row-retention tests, typechecker non-authority tests, and independent Core/CPS preservation tests. |

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1806](tasks/TASK-1806-phase-177-plan-packet.md) | Create the Phase 177 target-Ash row syntax and Core/CPS alignment packet | ✅ Complete |
| [TASK-1807](tasks/TASK-1807-row-syntax-core-cps-seam-audit.md) | Audit row syntax, Core row, CPS row, and lowering seams | ✅ Complete |
| [TASK-1808](tasks/TASK-1808-row-syntax-spec-delta-reconciliation.md) | Reconcile target row/effect syntax deltas into implementation decisions | ✅ Complete |
| [TASK-1809](tasks/TASK-1809-surface-computation-row-parser-carriers.md) | Add surface computation-row parser and AST carriers | ✅ Complete |
| [TASK-1810](tasks/TASK-1810-impl-qualified-operation-identity-resolution.md) | Resolve impl-qualified operation row identities | ✅ Complete |
| [TASK-1811](tasks/TASK-1811-row-validation-and-diagnostics.md) | Validate row syntax and emit fail-closed diagnostics | ✅ Complete |
| [TASK-1812](tasks/TASK-1812-core-row-taxonomy-alignment.md) | Align Core row taxonomy with target computation-row families | ✅ Complete |
| [TASK-1813](tasks/TASK-1813-cps-row-taxonomy-bridge.md) | Align CPS row/effect carriers and Core-to-CPS row lowering | ✅ Complete |
| [TASK-1814](tasks/TASK-1814-row-syntax-core-cps-cross-boundary-tests.md) | Add parser/engine/Core/CPS cross-boundary row preservation tests | ✅ Complete |
| [TASK-1815](tasks/TASK-1815-phase-177-closeout.md) | Close out Phase 177 with gates, review, and status reconciliation | ✅ Complete |
| [TASK-1816](tasks/TASK-1816-phase-177-row-syntax-review-remediation.md) | Remediate Phase 177 row syntax review findings | ✅ Complete |

## Implementation order

1. TASK-1807 audits live seams and records a current ownership map before implementation.
2. TASK-1808 writes the implementation decisions for NOTE-021/022/023/025 deltas and patches downstream task scope if needed.
3. TASK-1809 adds parsed surface row carriers and focused parser tests.
4. TASK-1810 adds bounded operation identity resolution and negative ambiguity tests.
5. TASK-1811 adds validation diagnostics for duplicate rows, row tails, unsupported items, and raw predicate leakage.
6. TASK-1812 aligns Core row item naming/taxonomy and row normalization properties.
7. TASK-1813 aligns CPS row/effect carriers and Core-to-CPS lowering without silent row loss.
8. TASK-1814 proves parser/engine retention, typechecker non-authority, and Core/CPS carrier preservation while recording the source-to-typechecker rowless boundary.
9. TASK-1815 runs broad gates, obtains independent review, and reconciles docs/changelog/status.
10. TASK-1816 remediates post-closeout row syntax review findings around whole-row variables, target open-row tail syntax, and operation separator preservation.

## Acceptance criteria

- [x] Inline callable rows and expanded `where row { ... }` blocks are parsed into explicit surface carriers with spans.
- [x] A callable cannot specify both an inline row and an expanded `where row { ... }` block.
- [x] Row tails are accepted only in final position and preserved as row variables.
- [x] Operation row items use impl-type-qualified identity where proven; unresolved or ambiguous impl-qualified operation identities fail closed, while lowercase/source-path rows remain unresolved requirement metadata until a later source-to-Core row bridge.
- [x] Supported row item families are represented in Core row carriers without being collapsed into legacy capability-only effects.
- [x] Core-to-CPS lowering either preserves supported row families or emits a precise unsupported-row diagnostic; it must not silently drop target row facts.
- [x] Row syntax remains requirement metadata and does not grant provider, role, admission, host, or workflow authority.
- [x] Cross-boundary tests cover parser row carriers, module/engine signature retention, typechecker non-authority, Core row carriers, CPS row lowering, and the recorded validation-only source-to-typechecker boundary.
- [x] PLAN-INDEX, task files, specs/docs references, and CHANGELOG agree on Phase 177 status.

## Verification baseline

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-core
cargo test -p ash-engine
cargo test -p ash-typeck
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Focused tasks should add narrower commands for the affected crates and tests. Closeout must run the full baseline unless a task records a user-approved deferral.

## Expected follow-on after Phase 177

If Phase 177 closes cleanly, the next plausible packets are target handler execution surface, provider/admission runtime wiring, row-polymorphic inference, fact/evidence declaration syntax, or stdlib/example corpus migration onto target row syntax. Those should remain separate phases unless Phase 177 proves the shared substrate is smaller than expected.

## Completion evidence

- TASK-1806 created and registered this planning packet, task files TASK-1806 through TASK-1815, PLAN-INDEX entries, and a CHANGELOG planning entry.
- TASK-1807 added the Phase 177 row syntax/Core/CPS seam audit with parser, typechecker, engine/module, Core, CPS, and Core-to-CPS ownership mapping plus named validation-only and lossy boundaries.
- TASK-1808 recorded Phase 177 implementation decisions for `Row` terminology, alternate inline/`where row` layouts, duplicate row errors, evidence row requirements, impl-qualified operation identity, and handler execution scope.
- TASK-1810 added bounded typechecker operation-row identity resolution for concrete impl identities such as `PosixFs::read`, abstract type-parameter identities such as `F::read` under `F: Fs`, and fail-closed diagnostics for interface-qualified or unknown identities.
- TASK-1811 added typechecker validation before function signature lowering for duplicate inline/expanded callable rows, row-tail placement, duplicate tails, and predicate-like row items that must use evidence references in this Phase 177 slice.
- TASK-1812 aligned Core row taxonomy with target-facing operation terminology by adding explicit operation helpers over the retained legacy `Capability` storage variant, parser aliases for `operation`/`op`, and focused normalization/public-summary/text round-trip tests.
- TASK-1813 implemented explicit CPS kinds for resource/process/evidence/failure families in both row and op lowering, plus open-row tail fail-closed diagnostics with regression coverage.
- TASK-1814 added parser, engine/module, typechecker, and Core/CPS cross-boundary tests for row span preservation, imported signature row retention, authority non-leakage, and CPS family preservation. It records source-to-typechecker callable rows as a validation-only boundary because current `Type::Fn` conversion remains rowless.
- TASK-1815 closed Phase 177 after focused and broad verification plus independent review remediation. Closeout clarifies that lowercase/source-path operation rows remain unresolved requirement metadata in this validation-only slice and that full source-to-Core row lowering, row-polymorphic inference, provider/admission runtime wiring, and end-to-end source row preservation into CPS remain future work.
- TASK-1816 remediated post-closeout row syntax review findings by adding whole-row variable carriers, target open-row tail parsing without a comma, operation separator preservation, and multi-character row-variable regression coverage while preserving fail-closed validation for predicate-like bare row names.
