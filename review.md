# SPEC-098b target CPS IR re-review

Reviewed: `docs/spec/SPEC-098b-TARGET-IR.md`
Scope: target CPS IR in isolation. Legacy/current IR and upper/pre-IR tiers ignored except where they affect target IR semantics.

No spec files were edited. This report overwrites `review.md` as requested.

## Verdict

PASS.

The target CPS IR is structurally sound as an isolated draft.

The target CPS IR is structurally sound at the spec level for the reviewed scope. The prior material blocker around shallow-handler residual row accounting is resolved, and no new material target-CPS issue was found.

## Material checks

### Shallow handler residual rows

PASS.

The previous issue was that shallow handlers were described operationally, but the row rule still looked like aggregate subtraction over the whole body row.

The current spec now makes the necessary segmentation explicit:

- shallow handlers remove only operation occurrences dispatched to the installed handler frame in the delimited body segment: lines 531-535;
- the row equation is explicitly restricted to the delimited pre-resume segment: lines 537-540;
- `captured_resume.local` is added separately and may include the same operation: lines 542-548;
- user-handler resume excludes the matched handler frame: lines 1107-1116 and 1137-1138.

That is enough to avoid the unsound “remove same-op effects after resume” interpretation. The shallow operational semantics and residual-row accounting now agree.

### Provider persistence

PASS.

Provider frames are now consistently modeled as persistent boundary frames:

- provider row removal is justified because the provider remains active after resume: lines 550-560;
- provider resume captures `prefix ++ matched ++ parent`: lines 1118-1127;
- capture-chain semantics distinguishes user handlers from providers: lines 1137-1141;
- provider body execution under `parent` is explicitly separated from provider persistence in the captured resume chain: lines 1152-1154.

This resolves the earlier provider/user-handler conflation.

### `LetRec` construction vs latent invocation rows

PASS.

`LetRec` now separates construction effects from latent body effects:

- construction local/total rows are charged at binding construction: lines 192-199;
- recursive `Lam` latent rows are recorded in function/body rows and charged on `Call`, not on `LetRec`: lines 201-202;
- recursive CPS functions are explicitly represented with `LetRec`: lines 1301-1316.

This resolves the previous over-accounting concern.

### Local/total row discipline

PASS.

The spec consistently distinguishes local rows from total rows:

- judgment form: lines 224-244;
- `Call.row` as cached total row: lines 257-259 and 342-346;
- `Raise.row` as operation/local row with resume effects in total row: lines 253-255 and 453-456;
- `Handle.row` as local residual row with continuation effects added to total row: lines 518-521;
- `LetCont` includes internal continuation body effects in local rows: lines 273-279.

### Cross-spec consistency

PASS.

No material contradiction found against `SPEC-096b-TARGET-EFFECT-SYSTEM.md` or `SPEC-097b-TARGET-TYPE-SYSTEM.md` for:

- rows-as-requirements;
- trap vs recoverable failure;
- contract discharge recording;
- capability/provider dispatch;
- provider persistence;
- resources and ambient discharge;
- `HandlerClause.row`;
- `Call.row`;
- local/total row mapping.

## Final assessment

PASS.

Within the requested scope — target CPS IR in isolation, ignoring legacy/current and upper/pre-IR work-in-progress tiers — `SPEC-098b` is now sound enough to proceed. I found no remaining material issue affecting the target CPS IR semantics or applicability of the core examples.
