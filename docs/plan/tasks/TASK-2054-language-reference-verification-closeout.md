# TASK-2054: Language Reference Verification and Closeout

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2046, TASK-2047, TASK-2048, TASK-2049, TASK-2050, TASK-2051, TASK-2052, TASK-2053

## Description

Integrate the manual: refresh evidence, validate examples and navigation, parse/render EBNF and
sequent fences, audit stale claims, and close the documentation phase without overclaiming
language or runtime support.

## Requirements

- Modify `docs/reference/language/index.md`, `status.md`, `source-of-truth.md`, `conventions.md`,
  every domain index, and the limitation/status pages as required by evidence refresh.
- Reconcile stale current-source claims identified in AUDIT-206, including `docs/spec/README.md`
  routing and legacy reference evidence links, by recording disposition; do not rewrite old corpus
  outside authorised task scope.
- Verify every current example as parser-only, checked, lowered, or executed; remove any claim
  whose evidence is missing. Removed forms are never current-reference examples.
- Create the task-owned, read-only `tools/docs/validate_language_reference_fences.mjs`. It must
  recurse only under `docs/reference/language`, extract every `ebnf` and `sequent` fence into the
  caller-supplied temporary directory, preserve source path/line provenance, and never edit the
  manual Markdown.
- The helper must import and call
  `/home/dikini/Projects/railroad/src/ebnf.js::compileEbnf` for each EBNF fence and
  `/home/dikini/Projects/sequent-md/packages/core/src/index.js::render` for each sequent fence.
  It exits nonzero for a malformed fence, a thrown compiler error, any `render(...).diagnostics`,
  an unreadable source file, or zero extracted fences of a required kind after the corresponding
  manual pages exist.

## Handoffs and dependencies

- **Consumes:** all domain pages and their evidence matrices, AUDIT-206, PLAN-206, and placement
  decision.
- **Produces:** an integrated, navigable manual with documented evidence revision and no
  unsupported current claims.
- **Non-goals:** implementation/spec changes, making all old `reference/` metadata valid, or
  converting fixture-bounded execution into full parity.

## TDD and verification steps

1. Create a failing manual inventory/link/fence matrix covering every page, feature status, and
   example classification.
2. Run per-page parser/typeck/Engine commands named by the domain tasks.
3. Create a temporary directory and execute the exact all-fence command:
   `tmpdir=$(mktemp -d) && node tools/docs/validate_language_reference_fences.mjs --root
   docs/reference/language --extract-dir "$tmpdir"; status=$?; rm -rf "$tmpdir"; exit $status`.
   Require the helper to reject EBNF productions without `=`, quoted terminals, terminal `;`, or
   with `::=` before calling `compileEbnf`.
4. Run the external project normal checks after the helper: `(cd /home/dikini/Projects/railroad &&
   npm run check)` and `(cd /home/dikini/Projects/sequent-md && npm test && npm run build)`.
   Record the helper's per-fence source/line result and both projects' tool versions/output; do not
   claim Ash's docs gate performs these checks.
5. Run `python3 tools/docs/validate_orientation_indexes.py --self-test`, `bash
   scripts/check-docs-gate.sh`, `git diff --check`, and any TASK-2045 manual validator. Run
   `python3 tools/reference/validate.py --root .` only as a legacy-corpus health report; separate
   its pre-existing failures from PLAN-206 acceptance.

## Completion checklist

- [ ] Every page is reachable from the language index and has current evidence/status.
- [ ] Every example has a verified parser/static/lowering/runtime classification.
- [ ] EBNF and sequent fences have external-tool validation evidence.
- [ ] Stale/excluded forms and legacy conflicts have explicit dispositions.
- [ ] Documentation gates, links, indexes, changelog, PLAN-INDEX, and diff hygiene are verified.
