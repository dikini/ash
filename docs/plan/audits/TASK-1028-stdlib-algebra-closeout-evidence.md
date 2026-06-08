# TASK-1028 Stdlib Algebra Closeout Evidence

## Status

Complete. This artifact records guarded focused evidence for TASK-1020 through TASK-1027 plus broad verification for SPEC-078 / PLAN-128 closeout.

## RED evidence

Before closeout, broad verification exposed stale adjacent baselines and a test-harness blocker:

- `RUSTC_WRAPPER= cargo test -p ash-cli --all-targets -- --nocapture` initially failed in `phase128_release_deployment_acceptance` because `CARGO_BIN_EXE_ashgrove` was unset when running from the `ash-cli` package test context.
- After that fix, the same broad gate exposed a stale stdlib corpus baseline: discovered stdlib files were 47 while the baseline expected 41.
- `RUSTC_WRAPPER= cargo test --workspace` exposed stale parser stdlib public-function lists for `option.ash` and `result.ash` after Phase 133 added algebra instance helpers (`pure`, `apply`, `and_then`).

## GREEN evidence

The closeout remediated those broad-gate findings without weakening acceptance intent:

- `phase128_release_deployment_acceptance` now builds/locates the sibling `ashgrove` workspace binary explicitly and still exercises real `ashgrove install`, `ashgrove lock`, authenticated dependency redaction, selected toolchain runtime-support capture, `ash check`, and `ash run`.
- `stdlib_corpus_check` now classifies all six `std/src/algebra/*.ash` files as expected-pass stdlib corpus files; `files=47`, `pass=41`, `fail=6`, `reference_only=0`.
- `stdlib_parsing` now expects the Phase 133 public helper functions added to `option.ash` and `result.ash`.

## Focused task evidence

Each filtered cargo test command is paired with a `-- --list` non-zero guard or an explicit artifact assertion. The exact command text below includes package, filter, and test_count / artifact assertion details.

| Task | command | package | filter | test_count / artifact assertion |
|---|---|---:|---|---|
| TASK-1020 | `python3 - <<'PY' ... assert TASK-1020 audit gate rows and downstream Verification metadata ... PY` | artifact | audit gate | artifact assertion: audit gate exists, downstream commands contain non-zero guards or artifact assertions |
| TASK-1021 | `RUSTC_WRAPPER= cargo test -p ash-engine std_algebra_namespace -- --list && RUSTC_WRAPPER= cargo test -p ash-engine std_algebra_namespace -- --nocapture` | ash-engine | `std_algebra_namespace` | test_count > 0 |
| TASK-1022 | `RUSTC_WRAPPER= cargo test -p ash-typeck pure_algebra_instances -- --list && RUSTC_WRAPPER= cargo test -p ash-typeck pure_algebra_instances -- --nocapture` | ash-typeck | `pure_algebra_instances` | test_count > 0 |
| TASK-1022 | `RUSTC_WRAPPER= cargo test -p ash-engine pure_algebra_instances -- --list && RUSTC_WRAPPER= cargo test -p ash-engine pure_algebra_instances -- --nocapture` | ash-engine | `pure_algebra_instances` | test_count > 0 |
| TASK-1023 | `RUSTC_WRAPPER= cargo test -p ash-typeck tower_monad_evidence -- --list && RUSTC_WRAPPER= cargo test -p ash-typeck tower_monad_evidence -- --nocapture` | ash-typeck | `tower_monad_evidence` | test_count > 0; hidden bridge leakage negative coverage |
| TASK-1023 | `RUSTC_WRAPPER= cargo test -p ash-engine tower_monad_evidence -- --list && RUSTC_WRAPPER= cargo test -p ash-engine tower_monad_evidence -- --nocapture` | ash-engine | `tower_monad_evidence` | test_count > 0; opaque runtime carrier evidence |
| TASK-1024 | `RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_do_evidence -- --list && RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_do_evidence -- --nocapture` | ash-typeck | `stdlib_do_evidence` | test_count > 0 |
| TASK-1024 | `RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_comprehension_evidence -- --list && RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_comprehension_evidence -- --nocapture` | ash-typeck | `stdlib_comprehension_evidence` | test_count > 0 |
| TASK-1024 | `RUSTC_WRAPPER= cargo test -p ash-engine stdlib_do_evidence -- --list && RUSTC_WRAPPER= cargo test -p ash-engine stdlib_do_evidence -- --nocapture` | ash-engine | `stdlib_do_evidence` | test_count > 0 |
| TASK-1025 | `RUSTC_WRAPPER= cargo test -p ash-engine algebra_combinators -- --list && RUSTC_WRAPPER= cargo test -p ash-engine algebra_combinators -- --nocapture` | ash-engine | `algebra_combinators` | test_count > 0; Phase 135 cleanup now verifies algebra modules are interface-only and concrete concat helpers are carrier-owned |
| TASK-1025 | `RUSTC_WRAPPER= cargo test -p ash-cli algebra_examples -- --list && RUSTC_WRAPPER= cargo test -p ash-cli algebra_examples -- --nocapture` | ash-cli | `algebra_examples` | test_count > 0 |
| TASK-1026 | `python3 - <<'PY' ... docs/plan/audits/TASK-1026-algebra-law-test-handoff.md ... TASK-1029 ... SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md ... PY` | artifact | law-profile handoff | artifact assertion: Semigroup, Monoid, Functor, Applicative, Monad profiles, TASK-1029 generated-law owner, and the referenced SPEC-077 file present |
| TASK-1027 | `python3 - <<'PY' ... scoped stale wording check ... PY` | artifact | reference/stale wording | artifact assertion: `reference/stdlib/algebra.md` exists and stale Monad deferral wording is historical or follow-up scoped |

## Broad verification

Commands run from `/home/dikini/Projects/ash/.worktree/phase-133-standard-algebra` after remediation:

- `RUSTC_WRAPPER= cargo check --workspace` — passed.
- `RUSTC_WRAPPER= cargo test -p ash-typeck --all-targets` — passed.
- `RUSTC_WRAPPER= cargo test -p ash-engine --all-targets` — passed.
- `RUSTC_WRAPPER= cargo test -p ash-cli --all-targets -- --nocapture` — passed; includes `phase128_release_deployment_acceptance`, `stdlib_corpus_check`, and `task_1025_algebra_examples`.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed after collapsing the nested sibling-binary helper `if`.
- `RUSTC_WRAPPER= cargo test --workspace` — passed after updating parser stdlib expectations.
- `RUSTC_WRAPPER= cargo doc --workspace --no-deps` — passed.
- `cargo fmt --check` — passed.
- `git diff --check` — passed.

Post-review docs-only wording remediation then reran:

- targeted stale-wording assertion scripts — passed.
- `cargo fmt --check` — passed.
- `git diff --check` — passed.
- `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1025_algebra_combinators -- --nocapture` — passed, 3 tests.
- `RUSTC_WRAPPER= cargo test -p ash-cli --test phase128_release_deployment_acceptance -- --nocapture` — passed, 1 test.

## Independent review

TASK-1024 independent review returned PASS with no blockers.

TASK-1028 closeout review was run through Codex after broad verification and post-review docs remediation. Earlier review attempts found stale SPEC/example wording and an A78-9 proof-depth overclaim; those blockers were remediated and focused checks were rerun.

A later post-evidence final sanity review found two remaining status/honesty blockers: SPEC-054 still had current-looking Phase-105 target/dictionary wording in the computation-target section, and the PLAN-INDEX progress summary still listed Phase 133 as `9 | 2 | 🚧 In Progress`. Both were remediated, targeted assertion/fmt/diff/focused tests were rerun, and the final Codex sanity review returned `APPROVED` with no blockers.

## Acceptance matrix A78-1 through A78-12

- A78-1: satisfied by `std/src/algebra/*.ash`, `std/src/lib.ash`, namespace/import coverage, and stdlib corpus baseline.
- A78-2: satisfied by engine/std module check tests and parser stdlib tests.
- A78-3: satisfied by pure `Option`, `Result`, `List`, and `String` evidence tests.
- A78-4: satisfied by tower evidence tests tying `Act`, `Proc`, and `Workflow` to named public/prelude evidence.
- A78-5: satisfied by `stdlib_do_evidence` typeck/engine tests for `do:Option` and `do:Result<_, E>`.
- A78-6: satisfied by `stdlib_comprehension_evidence` tests.
- A78-7: satisfied by missing/ambiguous evidence fail-closed tests in the task-owned evidence suites.
- A78-8: satisfied by hidden bridge leakage negative tests and do-target audit coverage.
- A78-9: satisfied by algebra combinator engine tests, executable final-surface monoid helper examples, and algebra example CLI check coverage for the broader currently expressible helper/import surface.
- A78-10: satisfied by `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md` and `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md`.
- A78-11: satisfied by `reference/stdlib/algebra.md`, generalized-do reference updates, SPEC-054/SPEC-055/SPEC-066/SPEC-067 stale deferral wording cleanup, and the scoped stale deferral sweep.
- A78-12: satisfied by the broad verification commands above.

## Remaining follow-up

Generated algebra law-test execution is intentionally not part of Phase 133. It is owned by `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md` and seeded by `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`.
