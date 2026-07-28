# TASK-2040: Remove Rust Direct AST and Differential Execution

**Status:** Complete
**Semantic task classification:** semantic-runtime-removal
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2034, TASK-2036, TASK-2037, TASK-2038, TASK-2039, and TASK-2042
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Rule coverage:** [Engine-only removal coverage](../SEMANTIC-RULE-COVERAGE.md#task-2040-engine-only-removal)

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
- Add enumerated-domain property tests showing manifest-supported admitted programs still terminalize
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

## Semantic workflow record

**Canonical rules:** `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-TARGET-CORE-CPS-001`,
`OBS-TARGET-PROJECTION-001`, and `CONF-IMPLEMENTATION-001`.

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The target Core/CPS domains and TASK-2041's four-client comparison remain incomplete.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.

**Run-route impact:** active.

**Consumes:** the frozen `AUDIT-204` dispositions, TASK-2035's selected source contracts,
TASK-2036's re-entry guard, and the Engine boundary delivered by TASK-2037 through TASK-2042.

**Produces:** removal of every owned Rust legacy-execution entry, the evaluator-free
`ash-runtime` support crate, and retained Lean material with its external separate-project handoff.

**Downstream owner:** TASK-2041 validates the zero-use state, closes the documentation and
traceability reports, and owns the four-client normalized-terminal comparison.

**Does not own:** Lean implementation or deletion, a direct-evaluator compatibility route, source synthesis, a new execution domain, or TASK-2041's four-client parity proof.

**Integration/proof responsibility:** This task owns manifest reconciliation and API-absence
controls. TASK-2041 owns the final cross-client comparison and any later production refinement
claim.

**Next obligation:** TASK-2041 validates the zero-use state, documentation and traceability, and four-client parity.

## Task-owned evidence plan

The following controls passed against the removal change.

- `TEST-TASK-2040-ENGINE-TERMINAL-POSITIVE` (**Positive**): the selected TASK-2035 source
  contract terminalizes through Engine without a legacy evaluator.
- `TEST-TASK-2040-MANIFEST-REMOVAL` (**Negative**): parses the frozen manifest, requires each
  owned delete path to disappear, and permits only `AUDIT-204-AST-010` as an entry-level shared
  Engine-file exception whose exact `mod differential;` declaration must disappear.
- `TEST-TASK-2040-EXTERNAL-API-ABSENCE` (**Negative**): static external clients cannot import or
  call direct AST evaluation or non-Engine CPS execution, and `ash_engine` has no differential
  module.
- `TEST-TASK-2040-REPLACEMENT-LEAN-CONTROLS` (**Negative**): the four named runtime replacements
  have no old source path, survive only under the renamed `ash-runtime` root, and each deferred
  Lean entry retains its separate-project authority boundary.
- `TEST-TASK-2040-DECLARED-CONTRACT-ENGINE-PROPERTY` (**Mutation**): ranges only over the exact
  TASK-2035 source identities and verifies their Engine terminal envelopes without constructing
  another source form.

## Completion checklist

- [x] Every Rust `delete` entry in AUDIT-204 is absent.
- [x] No Rust direct AST interpreter, differential oracle/corpus, or independent CPS evaluator is
      compiled or reachable.
- [x] Lean sources/docs remain present, labeled deferred separate-project work, and excluded from
      current Ash execution/conformance authority.
- [ ] Historical retained material is prose-only, labeled, and excluded from current read paths.
- [x] The task records exact removal evidence; it does not claim all target features implemented.
