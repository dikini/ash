# TASK-1993: Verus Pilot 2 — Frame-Ordered Operation Dispatch

**Status:** Complete (scoped model proof; direct Rust refinement remains deferred)
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1992 and the `λAsh-Effect` lookup-rule freeze

## Description

Verify that production operation dispatch selects the innermost matching handler or provider frame
according to one shared ordering relation.

## Requirements

- Target `HandlerChain::find_operation_frame` and the selection boundary consumed by `eval_raise`.
- Prove absence, bounds/match, greatest matching index, payload provenance, matching-frame
  shadowing, nonmatching-frame preservation, and equal handler/provider ordering.
- Keep provider execution, handler body evaluation, and resume behavior outside the pilot theorem.
- Add a mutation gate that fails when search order becomes outermost-first.
- Pilot one LLM lemma-repair loop whose output is accepted only after Verus checks it.

## TDD Steps

1. Strengthen inner-provider/outer-handler and inner-handler/outer-provider property fixtures.
2. Define the canonical frame-stack view and checked correspondence.
3. Add the lookup/shadowing proofs and deliberate proof-break/repair benchmark.
4. Run Verus, focused Rust, conformance, traceability, and documentation gates.

## Completion Checklist

- [x] The lookup/shadowing model theorem set passes reproducibly: the pinned runner reports
  `8 verified`, `0 errors` for `frame_lookup.rs` under `--no-cheating --rlimit 120`.
- [x] Current production dispatch uses the corresponding reverse-scan algorithm and focused Rust
  tests include an outermost-first mutation sentinel. This is executable correspondence evidence,
  not a claim that production Rust has been directly proved.
- [x] Assumptions, logical-escape categories, model boundary, release checksum, shared Rust 1.96.0
  baseline, and tool version are explicit in the manifest and report.
- [x] The deliberately false nonmatching-frame candidate is checker-rejected, while the repaired
  preservation lemma is checker-verified. No authoring-tool or LLM provenance is evidenced, so the
  benchmark explicitly makes no LLM-generation or LLM-repair claim.

## Evidence

- `verification/verus/frame-lookup-manifest.json`, `run-frame-lookup.sh`, and
  `frame-lookup-report.json` are dedicated TASK-1993 artifacts; they reuse only the pinned release
  and shared Rust 1.96.0 baseline accepted by TASK-1991.
- The runner executes from a temporary directory outside the checkout. It requires both outcomes:
  repaired model exit 0 / 8 verified / 0 errors, and the deliberately broken candidate exit 1 /
  1 error. It writes no generated result to the repository root.
- `verification/verus/FRAME-LOOKUP-README.md` records the exact selection theorem and its direct
  production-refinement gap. The trace graph marks the finite model proof as proved while retaining
  `REQ-CPS-FRAME-LOOKUP-DIRECT-BRIDGE-001` as deferred.
- Provider execution, handler-body evaluation, shallow resume, and Rust representation/error
  correspondence remain outside the pilot theorem. Their proof obligations remain owned by the
  programme closeout and subsequent refinement work.
