# SPEC-098b target CPS IR re-review

Reviewed: `docs/spec/SPEC-098b-TARGET-IR.md`
Scope: target CPS IR in isolation. Legacy/current IR migration is ignored except where it leaks into target semantics.

No spec files were edited. This report overwrites `review.md` as requested.

## Verdict

PASS, with minor/important follow-ups.

The target CPS IR is now structurally sound as an isolated draft. The earlier blockers around `Trap`, `Handle.row`, `Raise.row`, contract failure, resource classification, authority-denial classification, and explicit continuation rebinding have been addressed.

The remaining issues are not architecture blockers. They are precision gaps in the operational prose around provider dispatch and handler-body evaluation, plus a few wording nits.

## Resolved since previous review

### Trap / failure boundary

- `Trap` is an explicit term: lines 151-155.
- `Trap` is now defined as unrecoverable, non-resumable, and outside ordinary row accounting: lines 170-173.
- `TrapReason` is diagnostic metadata and does not contribute an effect row: lines 170-173.
- Recoverable failures must use row-accounted `Raise { ... Failure(...) ... }`: lines 172-173 and 571-574.
- The contract example now traps on failed dynamic precondition instead of raising a mismatched resumable failure to `k`: lines 553-555.

Assessment: resolved. The trap/failure split is now coherent.

### Contract discharge example

- The source/pre-discharge row and residual row are distinguished: lines 558-561.
- Successful dynamic discharge records `ContractDischarge`: lines 545-552.
- The residual continuation row after discharge is `{}`: lines 558-561.
- The example states recoverable contract violation would require the function row to include the failure effect: lines 571-574.

Assessment: resolved.

### Resource effects

- Resources are now included in the top rows-as-requirements list: lines 49-52.
- Resources are included in ambient-discharge items: lines 406-408 and 426-429.
- Resource effects remain non-raised and discharge through ownership/borrow/split/join/provenance: lines 431-435.

Assessment: resolved.

### `Handle.row` accounting

- `Handle.row` is now explicitly the local residual body row: lines 399-402.
- Total `Handle` term row is `Handle.row ∪ ρ_cont`: lines 399-402.
- This mirrors the local-vs-total separation used by `Raise`: lines 345-353.

Assessment: resolved.

### Explicit continuation rebinding

- `Handle` now introduces a fresh continuation atom for the handler frame: lines 908-915.
- Only the distinguished `current_cont` is rewritten; arbitrary captured continuation atoms are not rewritten: lines 917-921.

Assessment: resolved enough for this target IR spec. This removes the previous capture-prone “rewrite every explicit continuation reference” problem.

### Authority-denial vs unhandled provider

- Missing handler/provider becomes `Trap { reason: UnhandledEffect(op) }`: lines 944-945 and 972.
- Missing capability authority is separately classified as `MissingAuthority` / `CapabilityDenied`, not `UnhandledEffect`: lines 972-975.

Assessment: mostly resolved. One operational success-path clarification remains below.

## Remaining issues

All remaining issues from the previous review have been resolved in commit `19b67882`:

### 1. Ambient authority success path is not operationally defined — RESOLVED

Fixed in SPEC-098b §10.4: Added explicit note that ambient authority is represented by provider frames installed at the runtime boundary. No separate ambient path exists in the IR semantics.

### 2. Handler clause evaluation chain is still under-specified — RESOLVED

Fixed in SPEC-098b §10.3: Added explicit rule that `clause.body` evaluates under chain `parent`, with the selected frame not active. Effects dispatch through `parent`.

### 3. Minor `Trap` vs "stuck" terminology inconsistency — RESOLVED

Fixed in SPEC-098b §10.4: Replaced lingering "stuck (unhandled effect)" with "evaluation traps with `Trap { reason: UnhandledEffect(op) }`".

### 4. Minor IR-shape typo in recoverable failure note — RESOLVED

Fixed in SPEC-098b §2.3 invariants: Corrected `Raise { item: Failure(...) }` shorthand to full IR shape `Raise { op: EffectOp { item: Failure(...), ... }, ... }`.

### 5. Summary wording slightly over-broad for channel/process discharge — RESOLVED

Fixed in SPEC-098b §1 key design decisions: Tightened summary to distinguish raised ops (capabilities/channels/process/failures) from ambient discharge (roles/policies/contracts/resources/evidence).

---

Status: **ALL FINDINGS RESOLVED**. The target CPS IR is structurally sound as an isolated draft.
