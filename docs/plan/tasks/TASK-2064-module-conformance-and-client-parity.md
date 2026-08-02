# TASK-2064: Module Conformance and Client Parity

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §11; PLAN-203
**Owned rules:** MOD-REAL-001 through MOD-REAL-006 integration
**Run-route impact:** active

## Semantic accounting

**Implementation:** not_implemented. **Evidence:** none. **Parity:** below_spec.
**Missing target-spec clauses:** full cross-layer conformance, mutation resistance, and CLI/daemon terminal equivalence.
**Layers:** type/Core/CPS/admission-runtime `partial`; verification `not_implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-CONFORMANCE-POSITIVE`; negative `TEST-MOD-REAL-CONFORMANCE-NEGATIVE`; mutation `TEST-MOD-REAL-CONFORMANCE-MUTATION`; parity `TEST-MOD-REAL-CLI-DAEMON-PARITY`.
**Next obligation:** transfer executed evidence and remaining gaps to TASK-2065.

## Description

Prove the complete module route with positive, negative, mutation, and client-parity evidence. This task does not invent semantics; it validates the interfaces and execution artifacts produced by TASK-2057 through TASK-2063.

## Dependencies

- 📝 TASK-2059 — source parity substrate.
- 📝 TASK-2061 — interface import/visibility.
- 📝 TASK-2063 — admitted linked module execution.

## Requirements

1. Establish a compact conformance corpus with paired file and inline module programs.
2. Cover structural graph construction, source parity, public/private interfaces, imports, every visibility form, re-exports, identity preservation, and lowering parity.
3. Cover missing child, duplicate child, structural cycle, import cycle, ambiguity, inaccessible declaration, malformed/stale interface, and forbidden fallback diagnostics.
4. Add property/mutation tests for order independence, alias identity preservation, no implicit flattening, and text-lookalike resistance.
5. Compare one identical admitted multi-module program through CLI and daemon and assert normalized terminal equality.

## TDD Steps and evidence

1. Add rule-indexed fixtures before extending the implementation, one positive and one negative case per `MOD-REAL-*` rule.
2. Add a fixture generator that can materialize equivalent inline/file trees.
3. Run the generator against parser, graph, interface, Core/CPS, and Engine snapshots; shrink any mismatch to a minimal declaration tree.
4. Add mutation controls that inject a source scan, direct evaluator fallback, interface identity rewrite, or visibility bypass and show at least one test fails.
5. Record the exact theorem scope of any proof attempt separately; tests remain `tested`, not `proved`.

## Completion checklist

- [ ] Every `MOD-REAL-*` rule has positive, negative, and mutation evidence.
- [ ] File/inline module pairs agree at interface, Core/CPS, admission, and terminal layers.
- [ ] CLI and daemon execute the same admitted artifact with equal normalized terminals.
- [ ] Focused cross-crate tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** all module artifacts and active Engine route evidence.
- **Produces:** rule-indexed implementation/evidence/parity reports, CLI/daemon terminal comparison, and closeout inputs.
- **Downstream owner:** TASK-2065 owns closeout, documentation, traceability, and review remediation.
- **Non-goals:** new language behavior, new direct evaluator, or broad package/workspace functionality.

## Files and verification

**Files:** focused parser/core/typeck/engine integration tests; CLI and daemon parity tests; `docs/plan/SEMANTIC-RULE-COVERAGE.md`; traceability records required by active semantic-task policy.

```text
cargo test -p ash-parser module
cargo test -p ash-core module
cargo test -p ash-typeck module
cargo test -p ash-engine module
cargo test -p ash-cli module
cargo fmt --check
```
