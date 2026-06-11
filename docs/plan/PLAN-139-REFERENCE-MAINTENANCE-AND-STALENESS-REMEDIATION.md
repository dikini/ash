# PLAN-139: Reference Maintenance and Staleness Remediation

> **For Hermes:** This is a planning-only handoff. Use `subagent-driven-development`, `verification-before-completion`, and `write-docs-workflow` before executing any implementation task.

**Goal:** Remediate reference documentation staleness drift identified after Phase 138 closeout. Update verification baselines, add missing agent cards, and establish a repeatable reference-maintenance procedure.

## Status: ✅ Complete
**Depends on:** [PLAN-138](PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md), [PLAN-125](PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
**Task range:** TASK-1395 through TASK-1398

---

## Live context verified before planning

After Phase 138 closeout, the following reference drift was identified:

1. `reference/stdlib/algebra.md` lacks metadata frontmatter (`id`, `verified_against`, `refresh_trigger`, etc.) — all other stdlib pages have it.
2. `reference/agents/cards/` lacks a `stdlib-algebra.md` agent card — cards exist for act, proc, workflow, result but not algebra.
3. `reference/INDEX.md` does not link `stdlib/algebra.md` in the Standard Library section.
4. All stdlib reference pages (README, act, proc, workflow, result) and the INDEX have stale `verified_against.git_commit: 710340f` and `last_verified: 2026-06-01` — HEAD is now past Phase 138.
5. No formal reference-maintenance procedure exists for post-phase closeout reference refresh.

---

## Implementation plan

### TASK-1395: Add metadata frontmatter to `reference/stdlib/algebra.md`

**Type:** Reference/Docs

**Objective:** Add YAML metadata frontmatter matching the pattern of other stdlib reference pages.

**Files:**
- Modify: `reference/stdlib/algebra.md`

**Acceptance:**
- Frontmatter includes `id: ref.stdlib.algebra`, `kind: reference`, `authority: canonical-adjacent`
- `verified_against.git_commit` points to the commit that added/verified the content
- `verified_against.specs` links SPEC-078, SPEC-079, DESIGN-NOTE-INTERFACE-LAWS
- `verified_against.tasks` links TASK-1388 through TASK-1394
- `verified_against.code` links all `std/src/algebra/*.ash` and `std/src/option.ash`, `std/src/result.ash`
- `verified_against.tests` links parser law/proof tests
- `refresh_trigger` lists all upstream files that should trigger a re-verify

### TASK-1396: Add `stdlib-algebra.md` agent card

**Type:** Reference/Docs

**Objective:** Create the missing agent derivative card for `ref.stdlib.algebra`.

**Files:**
- Create: `reference/agents/cards/stdlib-algebra.md`

**Acceptance:**
- Card follows the exact pattern of `stdlib-act.md`, `stdlib-proc.md`, etc.
- `canonical_page: ref.stdlib.algebra`
- `dependency_order: stdlib-algebra`
- Retrieval tags cover: algebra interfaces, semigroup/monoid/functor/applicative/monad, comonad/kleisli, law declarations, proof declarations, by test delegation, source-visible laws, Eq evidence, option/result instances, do notation, interface evidence constraints
- Stale-claim warnings prevent common overclaims (by_definition validation, generated test execution, Comonad instances, Kleisli wrappers, Coapplicative existence)
- Edit preflight lists the exact test files to run before editing

### TASK-1397: Refresh verification baselines across reference corpus

**Type:** Reference/Docs

**Objective:** Update `last_verified` and `verified_against.git_commit` on all stale reference pages.

**Files:**
- Modify: `reference/INDEX.md`
- Modify: `reference/stdlib/README.md`
- Modify: `reference/stdlib/act.md`
- Modify: `reference/stdlib/proc.md`
- Modify: `reference/stdlib/workflow.md`
- Modify: `reference/stdlib/result.md`

**Acceptance:**
- All pages updated to `last_verified: 2026-06-11` (or current closeout date)
- All pages updated to `verified_against.git_commit: <current HEAD>`
- No other content changes

### TASK-1398: Add reference-maintenance procedure and closeout

**Type:** Reference/Docs

**Objective:** Document a repeatable post-phase closeout reference refresh procedure, update CHANGELOG, and reconcile PLAN-INDEX.

**Files:**
- Modify: `reference/maintenance/refresh-procedure.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md`

**Acceptance:**
- Reference maintenance procedure includes: check for missing pages/cards, add metadata frontmatter to new pages, create agent cards for new canonical pages, update verification baselines on affected pages, run markdown link check, run docs gate
- CHANGELOG entry under `[Unreleased]`
- PLAN-INDEX summary table shows Phase 139 as Complete
- PLAN-INDEX phase body shows all TASK-1395–1398 as Complete

---

## Key risks and decisions

1. **Verification baseline updates are mechanical but error-prone.** Use a script or exact find-replace, not manual edits.
2. **Agent cards must stay derivative.** Do not redefine semantics; always point to canonical pages first.
3. **Missing metadata frontmatter breaks staleness detection.** All new reference pages must include it.
4. **The procedure must be repeatable.** Document exact commands so future phases can follow the same steps.

## Recommended execution approach

Execute in the main worktree (no isolated worktree needed — all changes are docs-only):

```bash
cd /home/dikini/Projects/ash
```

Start with TASK-1395 and TASK-1396 in parallel (they are independent), then TASK-1397, then TASK-1398.
