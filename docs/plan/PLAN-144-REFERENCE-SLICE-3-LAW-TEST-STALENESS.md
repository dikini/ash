# PLAN-144: Reference Slice 3 — Law-Test Runner Integration and Staleness Checker

> **For Hermes:** This is a planning-only handoff. Use `subagent-driven-development`, `verification-before-completion`, and `write-docs-workflow` before executing any implementation task. Do not change Ash parser, typechecker, or runtime semantics. All work is tooling, reference, and test infrastructure.

## Phase: 144

## Status: ✅ Complete

## Goal

Make the `std::algebra` law declarations from Phase 136 executable via generated property tests, and automate reference staleness detection so the `reference/` corpus stays maintainable without manual drift audits.

This phase is the natural continuation of:
- **Phase 130** (Reference Slice 2) — established the `reference/` corpus structure
- **Phase 136** (Interface Law Syntax) — added `law`/`proof` syntax to the language
- **Phase 139** (Reference Maintenance) — added metadata, agent cards, and refresh procedures

## Background

After Phase 136 closeout, the `std::algebra` interfaces have `law` declarations (Semigroup, Monoid, Functor, Applicative, Monad) and `proof` blocks, but:
1. **No generated tests execute these laws** — the runner extracts laws (TASK-1368) but does not generate or run property tests from them
2. **Staleness detection is manual** — the `reference/` corpus has metadata frontmatter but no automated checker to flag stale pages

Phase 144 closes both gaps without expanding language semantics.

## Scope

### In scope

1. **Generated algebra law tests** (TASK-1029 continuation):
   - Law profile data structures for Semigroup, Monoid, Functor, Applicative, Monad
   - Pure carrier test generation: `String`, `List`, `Option`, `Result<_, E>`
   - Tower carrier gating: `Act`, `Proc`, `Workflow` explicitly deferred with diagnostics
   - Runner integration: SPEC-077 framework discovers and reports algebra law families

2. **Reference staleness checker automation** (TASK-1440–TASK-1442):
   - Implement `tools/reference/check_staleness.py` with `--slice` support
   - Git-diff-based staleness detection using `verified_against.git_commit`
   - Per-page `needs-inspection` flagging with refresh-trigger matching
   - JSON and human-readable output modes

3. **Reference Slice 3 pages** (TASK-1443–TASK-1444):
   - `reference/stdlib/algebra.md` — full surface documentation for `std::algebra`
   - Agent card for `ref.stdlib.algebra`
   - Update `reference/INDEX.md` with algebra links

4. **Closeout and verification** (TASK-1445):
   - Full workspace gate: `cargo test`, `cargo clippy`, `cargo fmt --check`
   - Reference validator: `python3 tools/reference/validate.py`
   - Markdown link check
   - CHANGELOG.md entry
   - PLAN-INDEX update

### Out of scope

- Comonad/Kleisli/Cokleisli law tests (deferred to future phase; TASK-1036 handoff preserved)
- Tower carrier law execution (blocked on bounded equivalence metadata)
- Proof body verification/totality checking (Phase 136 deferred boundary)
- Moving or rewriting historical `docs/` corpus
- Runtime, parser, typechecker semantic changes

## Architecture

### Parallel Development Streams

```
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 144 — Parallel Streams                     │
├─────────────────────────────┬───────────────────────────────────────┤
│   Stream A: Law Tests       │   Stream B: Staleness Checker         │
│   (TASK-1440, TASK-1441)    │   (TASK-1442)                         │
├─────────────────────────────┼───────────────────────────────────────┤
│ 1. Law profile structs      │ 1. Git diff scanner                   │
│ 2. Pure carrier generators    │ 2. Frontmatter parser                 │
│ 3. Runner integration         │ 3. Refresh-trigger matcher            │
│ 4. Property test execution    │ 4. JSON/human output formatter        │
│ 5. Tower carrier gating       │ 5. --slice reference-slice-3 support  │
└─────────────────────────────┴───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│   Stream C: Reference Pages (TASK-1443, TASK-1444)                  │
│   ── Can proceed in parallel with A and B after metadata model set   │
├─────────────────────────────────────────────────────────────────────┤
│ 1. `reference/stdlib/algebra.md` with SPEC-071 frontmatter           │
│ 2. `reference/agents/cards/stdlib-algebra.md` derivative card        │
│ 3. `reference/INDEX.md` crosslinks                                  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│   TASK-1445: Closeout and Verification                              │
│   ── Sequential gate after A, B, C complete                        │
└─────────────────────────────────────────────────────────────────────┘
```

### Worktree Organization

| Worktree | Branch | Tasks | Purpose |
|----------|--------|-------|---------|
| `.worktrees/phase-144-law-tests/` | `feat/phase-144-law-tests` | TASK-1440, TASK-1441 | Generated test implementation |
| `.worktrees/phase-144-staleness/` | `feat/phase-144-staleness` | TASK-1442 | Staleness checker tooling |
| `.worktrees/phase-144-reference/` | `feat/phase-144-reference` | TASK-1443, TASK-1444 | Reference pages |

All three merge to `feat/phase-144` for TASK-1445 closeout, then to `main`.

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1440](tasks/TASK-1440-law-profile-structures.md) | Law profile data structures and pure carrier generators | 6 | Phase 136, TASK-1026 | ✅ Complete |
| [TASK-1441](tasks/TASK-1441-runner-law-test-integration.md) | Runner integration: discover, generate, execute algebra law tests | 8 | TASK-1440, SPEC-077 | ✅ Complete |
| [TASK-1442](tasks/TASK-1442-staleness-checker-automation.md) | Implement `tools/reference/check_staleness.py` with `--slice` support | 8 | Phase 130, Phase 139 | ✅ Complete |
| [TASK-1443](tasks/TASK-1443-stdlib-algebra-reference-page.md) | Write `reference/stdlib/algebra.md` with SPEC-071 frontmatter | 4 | TASK-1442 (metadata model) | ✅ Complete |
| [TASK-1444](tasks/TASK-1444-stdlib-algebra-agent-card.md) | Create `reference/agents/cards/stdlib-algebra.md` derivative card | 3 | TASK-1443 | ✅ Complete |
| [TASK-1445](tasks/TASK-1445-phase-144-closeout.md) | Closeout: gates, CHANGELOG, PLAN-INDEX, drift report | 4 | TASK-1441, TASK-1442, TASK-1444 | ✅ Complete |

**Total estimated hours:** 33 hours

## Key Risks and Decisions

1. **Tower carrier law tests must stay gated.** Do not implement `Act`/`Proc`/`Workflow` law execution without bounded equivalence metadata. Emit explicit `deferred` diagnostics.

2. **Staleness checker must not overclaim.** If git diff is inconclusive, flag `needs-inspection` rather than `stale` or `fresh`. The checker is an aid, not an authority.

3. **Law test generators must be deterministic.** Use seeded proptest or equivalent. Reproducible failures are required for debugging.

4. **Reference pages stay derivative.** `reference/stdlib/algebra.md` must not redefine semantics; always point to canonical spec pages first.

5. **No runtime semantic changes.** This phase is tooling-only. If a law test reveals a semantic bug, file a separate bug/task rather than fixing it in this phase.

## Verification Gates

### Per-task gates

- **TASK-1440/1441:** `cargo test -p ash-cli generated_algebra_laws -- --nocapture` passes with non-zero generated test counts for pure carriers
- **TASK-1442:** `python3 tools/reference/check_staleness.py --slice reference-slice-3` runs without error and produces structured output
- **TASK-1443/1444:** `python3 tools/reference/validate.py` passes; markdown link check passes

### Phase closeout gate (TASK-1445)

```bash
# Rust gates
cargo fmt --check
cargo check --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace

# Docs gates
python3 tools/reference/validate.py
python3 tools/reference/check_staleness.py --slice reference-slice-3
# (or documented human audit if automation intentionally deferred)

# Git gates
git diff --check
git diff --cached --check
```

## Acceptance Criteria

- [x] Pure carrier algebra laws generate and execute via `ash test` runner
- [x] Tower carrier laws emit explicit `deferred` diagnostics with context
- [x] Staleness checker produces JSON and human-readable output
- [x] `reference/stdlib/algebra.md` has SPEC-071 frontmatter and crosslinks
- [x] `reference/agents/cards/stdlib-algebra.md` follows derivative card pattern
- [x] All reference pages in Slice 3 pass validation
- [x] CHANGELOG.md has `[Unreleased]` entry for Phase 144
- [x] PLAN-INDEX.md shows Phase 144 as Complete

## Recommended Execution Order

1. **Parallel start:** Create all three worktrees and begin TASK-1440, TASK-1442, TASK-1443 simultaneously
2. **Mid-phase sync:** TASK-1441 waits for TASK-1440; TASK-1444 waits for TASK-1443
3. **Merge to integration branch:** Merge all three worktrees to `feat/phase-144`
4. **Closeout:** TASK-1445 runs full verification gate

## References

- [SPEC-071](../spec/SPEC-071-REFERENCE-CORPUS-METADATA.md) — Reference corpus metadata
- [SPEC-075](../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md) — Reference Slice 2
- [SPEC-077](../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md) — Generated test runner
- [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md) — Standard algebra library
- [SPEC-079](../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md) — Comonad/Kleisli helpers
- [PLAN-125](PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md) — Reference Slice 2
- [PLAN-136](PLAN-136-INTERFACE-LAW-SYNTAX.md) — Interface law syntax
- [PLAN-139](PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md) — Reference maintenance
- [TASK-1029](tasks/TASK-1029-generated-algebra-law-tests.md) — Original generated law test plan
- [TASK-1026](tasks/TASK-1026-algebra-law-profile-generated-test-handoff.md) — Law profile handoff
- [TASK-1036](tasks/TASK-1036-comonad-law-profile-and-reference.md) — Comonad law profile handoff
