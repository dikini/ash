# PLAN-128: Standard Algebra Library and Monad Remediation

**Status:** 🚧 In Progress
**Spec:** [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
**Depends on:** [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-069](../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md), [SPEC-077](../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
**Task range:** TASK-1020 through TASK-1028

## Goal

Add a usable `std::algebra` standard-library namespace for `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad`; reconcile `do:K` and comprehension lowering with source-visible stdlib/prelude evidence; and retire obsolete hidden-dictionary deferrals without adding new language syntax.

## Architecture

The phase is library-first. Interfaces and helper functions live in Ash source under `std/src/algebra/`. Pure data instances should be ordinary Ash source where current interface/impl bodies can express them. Opaque tower carriers (`Act`, `Proc`, `Workflow`) keep Rust runtime implementations, but their public algebra evidence must point to public stdlib operations or named compiler-prelude shims, never anonymous hidden sequencing authority.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Now satisfied by | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-1020](tasks/TASK-1020-stdlib-algebra-audit-gate.md) | Audit live algebra/interface/do seams and freeze exact syntax, file targets, and evidence gates | Audit/Planning | 6 | ✅ Complete |
| [TASK-1021](tasks/TASK-1021-std-algebra-namespace-and-interfaces.md) | Add `std::algebra` namespace and source-visible algebra interfaces | Stdlib/Typeck | 10 | ✅ Complete |
| [TASK-1022](tasks/TASK-1022-pure-algebra-instances.md) | Add pure data instances for `Option`, `Result`, `List`, and string/list semigroups/monoids | Stdlib/Typeck | 14 | ✅ Complete |
| [TASK-1023](tasks/TASK-1023-tower-algebra-instances-and-bridge-remediation.md) | Add `Act`/`Proc`/`Workflow` algebra evidence and remove/quarantine hidden bridge authority | Typeck/Runtime Boundary | 16 | ✅ Complete |
| [TASK-1024](tasks/TASK-1024-do-and-comprehension-stdlib-evidence.md) | Rewire `do:K` and comprehensions to use stdlib/prelude Monad evidence | Typeck/Engine | 16 | Planned |
| [TASK-1025](tasks/TASK-1025-algebra-combinators-and-examples.md) | Add usable algebra combinators and executable examples | Stdlib/Examples | 10 | Planned |
| [TASK-1026](tasks/TASK-1026-algebra-law-profile-generated-test-handoff.md) | Create law-profile and generated-test follow-up packet without implementing law runner support | Docs/Test Runner Planning | 8 | Planned |
| [TASK-1027](tasks/TASK-1027-algebra-reference-and-corpus-migration.md) | Update reference docs and reconcile stale Monad/stdlib deferral wording | Docs/Reference | 8 | Planned |
| [TASK-1028](tasks/TASK-1028-stdlib-algebra-closeout.md) | Broad verification, independent review, status reconciliation, and closeout | Closeout | 8 | Planned |

Total estimate: 96h.

## Execution Order

1. TASK-1020 is a hard audit gate. It must inspect live parser/typechecker/std/module-loader/do-target seams and replace downstream placeholder verification with exact non-zero commands.
2. TASK-1021 creates the final source namespace and importable interfaces.
3. TASK-1022 adds pure data instances after the namespace exists.
4. TASK-1023 reconciles tower carriers with public evidence and runtime opacity.
5. TASK-1024 rewires `do:K` and comprehension lowering to the selected evidence path.
6. TASK-1025 adds ergonomic library functions and examples only after evidence selection is stable.
7. TASK-1026 creates a concrete law-profile generated-test follow-up task/phase seed so law tests are deferred explicitly to an owner, not forgotten or reduced to a prose-only note.
8. TASK-1027 updates daily-use docs/reference pages and removes stale deferral wording.
9. TASK-1028 runs broad gates and closes the phase only after positive visibility and negative leakage evidence are reconciled.

## Decision Gates

- D1: No new language syntax. All work uses existing modules, interfaces, impls, functions, imports, `do:K`, and comprehensions.
- D2: Canonical namespace is `std::algebra`; no direct root `std` re-export is required for first-slice acceptance.
- D3: Final-surface tests must import/check/use stdlib algebra modules. Inline local `interface Monad` fixtures are not sufficient evidence.
- D4: Rust runtime specialization may implement opaque tower carriers, but runtime behavior must not exceed the public Ash algebra surface.
- D5: Anonymous hidden Act/Proc/Workflow dictionaries must be removed or quarantined as named compiler-prelude evidence tied to public stdlib operations.
- D6: `do:K` and comprehensions must use the same selected `Monad<K>` evidence path for pure and tower carriers.
- D7: Law proof/checking and law-test derivation are follow-on implementation phases integrated with generated tests; this phase must create a concrete follow-up task/phase seed, not a prose-only handoff and not silent omission.
- D8: Missing and ambiguous evidence remain fail-closed.

## Sub-Agent Delegation Model

Use a fresh sub-agent per task. Each implementation task must include three prompts:

1. Implementer prompt: create RED tests, implement the minimal slice, run focused gates.
2. Spec-review prompt: verify against SPEC-078 and task acceptance rows, especially deferral disposition and final-surface gates.
3. Quality-review prompt: inspect for hidden bridge leakage, fixture-only tests, scope creep, and stale docs.

Parallelism is limited: TASK-1022 and TASK-1023 may use separate implementers only if TASK-1021 is complete and they do not edit the same test files. TASK-1027 may begin after TASK-1025, but closeout cannot start until all earlier tasks are complete.

## Verification Strategy

Every implementation task must record RED/GREEN evidence in the task file. Focused gates are set by TASK-1020. Minimum phase gates:

Filtered cargo test commands must be paired with `-- --list` or equivalent artifact assertions that record a non-zero matching test count before a task may claim focused verification passed. TASK-1020 must replace placeholder filters with exact commands and non-zero guards before downstream implementation starts.

```bash
cargo fmt --check
RUSTC_WRAPPER= cargo check --workspace
RUSTC_WRAPPER= cargo test -p ash-typeck --all-targets
RUSTC_WRAPPER= cargo test -p ash-engine --all-targets
RUSTC_WRAPPER= cargo test -p ash-cli --all-targets
RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTC_WRAPPER= cargo test --workspace
git diff --check
```

TASK-1028 also runs documentation/link/status checks for edited markdown and verifies no stale `Monad` deferral language remains in current normative surfaces except where explicitly marked historical or follow-up.

## Completion Checklist

- [ ] `std::algebra` namespace exists and is importable.
- [ ] Standard algebra interfaces parse/check through the real stdlib path.
- [ ] Pure data instances resolve from stdlib evidence.
- [ ] Tower carrier evidence is tied to public stdlib/prelude operations.
- [ ] `do:Option`, `do:Result<_, E>`, `do:Act`, `do:Proc`, and `do:Workflow` use selected Monad evidence.
- [ ] Comprehensions reuse the same evidence path.
- [ ] Anonymous hidden bridge authority is removed or quarantined with negative leakage tests.
- [ ] Usable combinators and examples compile/run.
- [ ] Law-test/proof work is explicitly scheduled as a generated-test follow-up.
- [ ] Reference docs, spec index, PLAN-INDEX, task files, and CHANGELOG are reconciled.
