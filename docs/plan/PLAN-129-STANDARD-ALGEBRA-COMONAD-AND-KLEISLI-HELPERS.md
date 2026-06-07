# PLAN-129: Standard Algebra Comonad and Kleisli Helper Surfaces

**Status:** ✅ Complete
**Spec:** [SPEC-079](../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
**Depends on:** [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md), [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-077](../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
**Task range:** TASK-1030 through TASK-1037

## Goal

Extend the stable `std::algebra` namespace with an implementation-grade plan for `Comonad`, Kleisli helpers, Cokleisli helpers, and an explicit Coapplicative decision. The phase keeps category-level abstractions deferred unless a later packet introduces them, and it avoids adding unsupported syntax or unsound instances.

## Architecture

The phase is library-first and audit-first. `Comonad` and helper modules live under `std/src/algebra/` only after TASK-1031 proves the exact live syntax and implementation seams. Kleisli helpers reuse Phase 133 `Monad<M>` evidence. Cokleisli helpers reuse `Comonad<W>` evidence. Coapplicative is a decision-gated surface: it must be precisely lawed with a lawful carrier or remain deferred.

Opaque tower carriers (`Act`, `Proc`, `Workflow`) are not Comonads in this plan. Extracting a value from those carriers would imply inspecting or executing runtime-managed computation outside the public tower algebra. `Option`, `Result`, and ordinary `List` are also rejected as default Comonad instances unless a later carrier-specific spec supplies a total/focused context.

## Deferral / planned-feature reconciliation

| Prior item | Source | Original reason | Now satisfied by | Decision | Gate |
|---|---|---|---|---|---|
| Comonad future work | SPEC-078 §Namespace Decision | `std::algebra` needed to stabilize first | Phase 133 complete | Plan now, implement after audit | `std::algebra::comonad` final-surface gate or explicit audit block |
| Category hierarchy | SPEC-078 §Namespace Decision | Category abstractions were beyond algebra MVP | Not satisfied | Keep deferred | No `std::category` or `Category` interface in this phase |
| Kleisli composition | SPEC-078/SPEC-054 Monad helpers | Generic helper syntax and law tests deferred | Monad evidence exists; generic helpers still constrained | Add helper module if expressible; otherwise named deferral | Final-path helper tests or audit-owned deferral |
| Cokleisli composition | New Comonad follow-on | Requires Comonad evidence first | Not satisfied until TASK-1032 | Implement only after Comonad surface | Cokleisli tests import Comonad final path |
| Coapplicative | User request / category duals | No current Ash definition or laws | Unknown | Decision gate before implementation | Precise lawed interface with lawful carrier, or explicit deferral |
| Generated law execution | TASK-1029 | Phase 133 law runner not implemented yet | TASK-1029 planned | Extend handoff; do not claim execution | TASK-1036 law-profile owner row |

## Task breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-1030](tasks/TASK-1030-comonad-kleisli-packet.md) | Create the SPEC-079/PLAN-129 packet, task files, index rows, and changelog entry | Docs/Planning | 6 | ✅ Complete |
| [TASK-1031](tasks/TASK-1031-comonad-kleisli-audit-gate.md) | Audit live algebra/interface/module/evidence seams and freeze exact syntax plus verification commands | Audit/Planning | 8 | ✅ Complete |
| [TASK-1032](tasks/TASK-1032-std-algebra-comonad-interface.md) | Add `std::algebra::comonad` interface/module if audit validates exact syntax | Stdlib/Typeck | 10 | ✅ Complete |
| [TASK-1033](tasks/TASK-1033-std-algebra-kleisli-helpers.md) | Add Kleisli helper surface over existing Monad evidence or defer exact helpers honestly | Stdlib/Helpers | 8 | ✅ Complete |
| [TASK-1034](tasks/TASK-1034-std-algebra-cokleisli-helpers.md) | Add Cokleisli helper surface over Comonad evidence or defer exact helpers honestly | Stdlib/Helpers | 8 | ✅ Complete (deferred source) |
| [TASK-1035](tasks/TASK-1035-coapplicative-decision-gate.md) | Define a precise Coapplicative first slice with laws and a lawful carrier, or defer it explicitly | Design/Audit | 6 | ✅ Complete (deferred source) |
| [TASK-1036](tasks/TASK-1036-comonad-law-profile-and-reference.md) | Extend law-profile handoff and update reference/corpus docs for implemented/planned surfaces | Docs/Test Runner Planning | 8 | ✅ Complete |
| [TASK-1037](tasks/TASK-1037-comonad-kleisli-closeout.md) | Run broad verification, independent review, status reconciliation, and closeout | Closeout | 8 | ✅ Complete |

Total estimate: 62h.

## Execution order

1. TASK-1030 creates the planning packet only. It does not implement stdlib source.
2. TASK-1031 is a hard gate. It must inspect live parser/typechecker/std/module-loader/evidence seams and replace downstream placeholder commands with exact focused non-zero gates.
3. TASK-1032 may begin only after TASK-1031 freezes the accepted `Comonad` source surface.
4. TASK-1033 may begin after TASK-1031 because it uses existing Monad evidence, but it must not edit the same files as TASK-1032 without coordination.
5. TASK-1034 depends on TASK-1032 because Cokleisli helpers need `Comonad` evidence.
6. TASK-1035 may run in parallel with helper planning only as a docs/design decision task; source implementation of Coapplicative waits for its decision.
7. TASK-1036 updates TASK-1029's generated-law-test scope and reference docs after the actual implemented/deferred surfaces are known.
8. TASK-1037 closes the phase only after all implemented surfaces have final-path tests and all deferred surfaces are named honestly.

## Decision gates

- D1: No new syntax. Implementation uses existing modules, interfaces, impls, ordinary functions, imports, and function types.
- D2: `std::algebra` remains canonical; `std::category` is not introduced.
- D3: `Comonad` requires total extraction. Partial or opaque carriers do not receive instances by symmetry.
- D4: Kleisli helpers reuse existing `Monad<M>` evidence and do not alter `do:K` lowering.
- D5: Cokleisli helpers reuse `Comonad<W>` evidence and do not claim a general category implementation.
- D6: Coapplicative must be lawed and instance-backed or deferred.
- D7: Law execution remains tied to SPEC-077/TASK-1029-style generated tests; this phase must create concrete handoff ownership for new laws.
- D8: Final-surface tests must import/check/use stdlib modules. Local fixture-only interfaces are not enough.

## Sub-agent delegation model

Use a fresh sub-agent per task. Each implementation task must include three prompts:

1. Implementer prompt: create RED tests or audit artifact, implement the minimal slice, and run focused gates.
2. Spec-review prompt: verify against SPEC-079, PLAN-129, and the task acceptance rows, especially negative instance policy and Coapplicative scope.
3. Quality-review prompt: inspect for unsupported syntax, fake instances, fixture-only tests, category overclaims, stale docs, and missing changelog/status updates.

Parallelism is limited. TASK-1033 and TASK-1035 may run after TASK-1031 if they do not edit the same source/reference files. TASK-1034 waits for TASK-1032. TASK-1037 waits for all prior tasks.

## Verification strategy

TASK-1031 must replace implementation-task placeholders with exact commands and non-zero guards. Minimum phase gates:

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

Filtered cargo commands must be paired with `-- --list`, a test-count assertion, or an artifact assertion proving the target exists and ran. TASK-1037 also runs markdown link/status checks for touched docs and verifies stale SPEC-078 future-work wording has been reconciled without overclaiming category support.

## Completion checklist

- [x] SPEC-079, PLAN-129, PLAN-INDEX, spec README, task files, and CHANGELOG are coherent.
- [x] Audit gate freezes exact accepted source syntax and replaces downstream verification placeholders.
- [x] `std::algebra::comonad` is implemented or blocked with a named audit reason.
- [x] Kleisli helpers are implemented through final stdlib paths or deferred with exact blockers.
- [x] Cokleisli helpers are implemented through final stdlib paths or deferred with exact blockers.
- [x] Coapplicative is precisely implemented or explicitly deferred.
- [x] Unsound Comonad instances remain absent with negative evidence.
- [x] Law-profile generated-test ownership covers new laws.
- [x] Reference docs and stale future-work wording are reconciled.
- [x] Broad verification and independent review pass before status promotion.
