# TASK-2054: Language Reference Verification and Closeout

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2046, TASK-2047, TASK-2048, TASK-2049, TASK-2050, TASK-2051, TASK-2052, TASK-2053

**Semantic task classification:** non-semantic-workflow-enforcement

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

- [x] Every page is reachable from the language index and has current evidence/status.
- [x] Every example has a verified parser/static/lowering/runtime classification.
- [x] EBNF and sequent fences have external-tool validation evidence.
- [x] Stale/excluded forms and legacy conflicts have explicit dispositions.
- [x] Documentation gates, links, indexes, changelog, PLAN-INDEX, and diff hygiene are verified.

## Completion evidence

- **Manual inventory and navigation:** the complete language manual is rooted at
  `docs/reference/language/index.md`; every page is reachable through its chapter index or the
  shared navigation/status pages. The closeout retained the source-code-and-tests evidence order
  and did not convert any partial route into a general execution claim.
- **Fence-validator tests:** `node --test tools/docs/validate_language_reference_fences.test.mjs`
  passed **23/23** tests. The task-owned helper then scanned the actual manual and validated
  **30** fences: **16 EBNF** and **14 sequent**.
- **External fence consumers:** `/home/dikini/Projects/railroad` completed `npm run check`
  (**38/38**) and `npm run build`; its two vendor `-0` warnings were non-fatal. The
  `/home/dikini/Projects/sequent-md` `npm test` run completed **26/26**, and `npm run build`
  completed successfully.
- **Repository documentation evidence:** `python3
  tools/docs/validate_orientation_indexes.py --self-test`, `bash scripts/check-docs-gate.sh`
  (**2,032 links, zero missing**), and `git diff --check` completed successfully. The repository
  gate is navigation/link evidence only; it does not replace the railroad or sequent checks.
- **Stale-material disposition:** `docs/spec/README.md` now routes current source claims to this
  implementation-backed manual. The legacy top-level `reference/` corpus and its fourteen
  dangling evidence links were recorded as a separate health report (`checked=98`, `errors=14`)
  and left unchanged; they do not establish or invalidate a current-language claim.
- **Target/future disposition:** the status map and AUDIT-206 now explicitly record that no
  target-only/planned source-language feature is known at the audited revision. Current partial,
  below-spec, bounded, and closed routes remain current limitations rather than future features.
- **Workspace identity:** implementation evidence remains reviewed against `423f603c`; the
  documentation closeout is recorded at the current uncommitted workspace state without inventing
  a commit hash.
