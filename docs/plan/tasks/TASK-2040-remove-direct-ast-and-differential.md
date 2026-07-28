# TASK-2040: Remove Rust Direct AST and Differential Execution

**Status:** Planned
**Semantic task classification:** semantic-runtime-removal
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2034, TASK-2036, TASK-2037, TASK-2038, TASK-2039, and TASK-2042

## Description

Delete every current/executable Rust entry in the frozen manifest whose disposition is `delete`:
direct AST interpreter code and exports, direct-evaluation tests and benchmarks, and the Rust
differential harness/corpus/oracle/tests/scripts/workflows. Preserve Lean sources and docs as
`deferred_separate_project`; remove or relabel only a Lean link that grants current Ash execution,
conformance, or differential authority. Preserve only explicitly historical prose/reference
material classified by TASK-2034; it must carry a historical label and cannot be executable or a
current read path. Complete the final `ash-interp` to `ash-runtime` rename handoff if TASK-2037
left it pending to keep the support crate evaluator-free.

## Requirements

- Delete only Rust entries with manifest disposition `delete`; preserve Lean entries with
  `deferred_separate_project` fields and relabel current-authority links without deleting Lean.
- Retain required runtime support only under its named non-evaluator owner.
- Add finite-domain property tests showing manifest-supported admitted programs still terminalize
  through Engine and cannot reach a removed evaluator; no test may generate new source forms or
  slices.
- Activation records removal implementation, test evidence, and target-rule parity separately.

## Handoffs

- **Run-route impact:** `active` because the selected client routes lose their legacy fallback
  material and must remain Engine-only.
- **Consumes:** catalogued dispositions, migrated test/REPL routes, and Engine executor boundary.
- **Produces:** deleted Rust legacy execution and an evaluator-free support crate; preserved Lean
  material with a separate-project handoff.
- **Downstream owner:** TASK-2041 validates the manifest reaches zero current/executable Rust
  legacy entries and owns documentation/traceability closure.
- **Does not own:** Lean implementation or deletion, replacement semantics, a new feature domain,
  or a new proof system.
- **Integration/proof responsibility:** TASK-2041 owns final cross-client evidence; this task owns
  deletion/API-absence tests and manifest reconciliation.

## TDD and activation steps

1. Activate semantic removal records with canonical rules and update coverage/traceability before
   deleting semantic Rust material.
2. Add compile/API-negative tests that imports and calls of AST evaluation, differential harness,
   and external CPS execution are unavailable; add a deferred-Lean-label control.
3. Delete only manifest-owned Rust entries; relocate required non-evaluator metadata to its named
   owner before removing a crate or module.
4. Run affected workspace tests and the re-entry guard with the deletion manifest.

## Completion checklist

- [ ] Every Rust `delete` entry in AUDIT-204 is absent.
- [ ] No Rust direct AST interpreter, differential oracle/corpus, or independent CPS evaluator is
      compiled or reachable.
- [ ] Lean sources/docs remain present, labeled deferred separate-project work, and excluded from
      current Ash execution/conformance authority.
- [ ] Historical retained material is prose-only, labeled, and excluded from current read paths.
- [ ] The task records exact removal evidence; it does not claim all target features implemented.
