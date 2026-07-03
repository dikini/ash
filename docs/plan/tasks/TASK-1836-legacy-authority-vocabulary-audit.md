# TASK-1836: Classify legacy authority vocabulary docs

## Description

Audit older Ash specs and notes that still use legacy authority vocabulary (`capability/provider`, `capability binding`, `capability invocation`, capability availability/admissibility). Decide which documents describe current-state compatibility or implemented substrate, and which are superseded historical references. Target correctness takes priority: compatibility language is retained only where it documents current implementation behavior or deliberate legacy lowering/admission inputs.

## Requirements

- Review stale authority vocabulary hits across `docs/spec` and `docs/notes`.
- Classify high-risk old authority docs in the orientation indexes.
- Add target reconciliation fences to documents whose main body still reads like current target guidance.
- Preserve explicit current-state compatibility wording where target specs intentionally admit legacy inputs.
- Update `CHANGELOG.md`.

## Completion criteria

- [x] Scan results for legacy authority vocabulary are reviewed and summarized.
- [x] `SPEC-INDEX.md` distinguishes target-state authority docs from current-state compatibility, implemented compatibility substrate, superseded historical, and deferred background docs.
- [x] Any edited spec/note includes local reconciliation wording where index metadata alone is insufficient.
- [x] `NOTE-INDEX.md` routes related notes through NOTE-022/023/025 for target-Ash authority planning.
- [x] `CHANGELOG.md` includes this task.
- [x] `python3 tools/docs/validate_orientation_indexes.py --self-test`, `bash scripts/check-docs-gate.sh`, and `git diff --check` pass.

## Evidence

- Scan reviewed:
  - `rg -n 'capability/provider|provider/capability|capability binding|capability bindings|CapabilityBinding|missing capability|capability invocation|capability availability|policy/capability|capability admissibility' docs/spec docs/notes -g '*.md'`
- Classified early capability/role specs:
  - `SPEC-002`, `SPEC-017`, `SPEC-018`, `SPEC-019`, and `SPEC-022` as current-state compatibility or historical/current-state compatibility.
  - `SPEC-024` as superseded historical capability-role surface.
- Classified tower/runtime specs:
  - `SPEC-047`, `SPEC-049`, and `SPEC-051` as current-state compatibility substrate.
  - `SPEC-069`, `SPEC-070`, and `SPEC-072` as implemented compatibility specs or implemented specs with compatibility notes.
- Classified unsuffixed unified-effect drafts:
  - `SPEC-096`, `SPEC-097`, `SPEC-098`, and `SPEC-099` as historical bridge drafts superseded by current/target split specs.
- Added reconciliation notices to edited high-risk specs and notes so legacy capability/provider vocabulary cannot override target-Ash provider/handler admission and operation-identity direction.
- Rephrased `SPEC-101` force-time authority wording to use provider/handler operation authority and resource access rather than legacy capability/provider phrasing.
- Updated `SPEC-INDEX.md` and `NOTE-INDEX.md` read paths and status rows to route target authority work through `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-099b`, `SPEC-100`, `NOTE-022`, `NOTE-023`, and `NOTE-025`.
- Verification passed:
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`

## Depends on

- PLAN-181.
