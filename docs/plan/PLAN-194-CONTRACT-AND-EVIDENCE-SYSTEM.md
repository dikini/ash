# PLAN-194: Contract And Evidence System

**Status:** ✅ Complete; all Phase 194 tasks finished and gates passing
**Depends on:** Phase 184 Handler / Provider Semantics and Phase 193 Surface Tuple ADT Expressions.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`,
`SPEC-100`, `PLAN-165`, `PLAN-183`, `PLAN-184`, `NOTE-027`, `NOTE-029`, `NOTE-031`,
`NOTE-032`, `NOTE-033`, `NOTE-034`, and `NOTE-035`.

## Goal

Add correctness obligations on top of the now-firm target computation model: surface
`requires`/`ensures`, predicate well-formedness, authority-free contract predicate evaluation,
evidence rows for tests/laws/proofs/runtime monitors, and structured blame diagnostics.

## Architecture

This phase treats contracts as correctness obligations over direct-style row-bearing `fn`
computations, not as authority grants or separate workflow boxes. Surface `requires` and `ensures`
lower into Core predicate sidecars, runtime check plans, discharge metadata, and evidence row
requirements. Predicate evaluation is deliberately authority-free: predicates may inspect captured
values and policy-governed observation evidence, but cannot perform operations, install handlers,
or acquire provider/resource/role authority.

## Scope

- Parse and preserve target-surface `requires` and `ensures` clauses on ordinary `fn`
  declarations.
- Validate contract predicates for stable, total, authority-free observer behavior.
- Lower contract predicates into structured Core artifacts, not source-text re-evaluation.
- Represent evidence rows for tests, laws, proofs, runtime monitors, and observation evidence.
- Integrate contract discharge with row admission without letting contract rows grant authority.
- Execute dynamic checks with distinct predicate-false and predicate-fault outcomes.
- Produce structured diagnostics with blame polarity, boundary identity, predicate identity,
  evidence references, snapshots, and redacted observation details.

## Non-Goals

- No new authority vocabulary beyond the operation/resource/role/policy/evidence/failure discharge
  families already established by Phases 183 and 184.
- No proof-assistant or SMT implementation beyond carrier/discharge hooks needed for fail-closed
  planning.
- No optimizer that consumes contract evidence.
- No generalized temporal logic surface syntax beyond monitor evidence carriers and diagnostics.
- No revival of workflow-specific contract semantics as a separate runtime path.

## Design Locks

1. `requires` failures blame the caller or negative position; `ensures` failures blame the callee,
   implementation, or positive position.
2. Predicate well-formedness requires both empty computation rows and a stable-observer
   classification; row-empty but unstable expressions such as time, randomness, pointer identity,
   or forced lazy state are rejected in contract position.
3. Contract predicates cannot perform operations, acquire authority, install handlers, admit roles,
   select resources, or discharge rows.
4. Operation-produced values may carry observation evidence into diagnostics, but that evidence is
   metadata, not a predicate capability.
5. Evidence rows are requirements and records. They can require or record tests, laws, proofs,
   runtime monitors, and observation evidence, but cannot prove authority by being mentioned.
6. Dynamic predicate falsehood and predicate evaluator faults are distinct outcomes:
   `ContractViolation(ContractDiagnostic)` versus
   `ContractPredicateFault(PredicateFaultDiagnostic)`.
7. Recoverable contract behavior must use an explicit `fail` or compensation operation row. Contract
   violation traps are not resumable operation effects by default.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1891](tasks/TASK-1891-contract-evidence-plan-packet.md) | Create the Phase 194 plan and task packet | 2h | Phase 193 | Planned |
| [TASK-1892](tasks/TASK-1892-contract-evidence-seam-audit.md) | Audit live contract/evidence carriers, row admission, and diagnostics boundaries | 4h | TASK-1891 | Planned |
||| [TASK-1893](tasks/TASK-1893-requires-ensures-surface-carriers.md) | Parse and preserve `requires`/`ensures` clauses on target `fn` declarations | 8h | TASK-1892 | ✅ Complete |
||| [TASK-1894](tasks/TASK-1894-contract-predicate-well-formedness.md) | Enforce predicate well-formedness and authority-free observer rules | 10h | TASK-1893 | ✅ Complete |
||| [TASK-1895](tasks/TASK-1895-surface-contract-lowering.md) | Lower surface contracts to Core predicate sidecars, snapshots, and check plans | 10h | TASK-1894 | ✅ Complete |
||| [TASK-1896](tasks/TASK-1896-evidence-row-substrate.md) | Add evidence row records for tests, laws, proofs, runtime monitors, and observations | 8h | TASK-1895 | ✅ Complete |
||| [TASK-1897](tasks/TASK-1897-contract-discharge-integration.md) | Integrate static/evidence/dynamic contract discharge with row admission | 8h | TASK-1896 | ✅ Complete |
||| [TASK-1898](tasks/TASK-1898-dynamic-contract-runtime-checks.md) | Execute dynamic contract checks with distinct violation and predicate-fault traps | 10h | TASK-1897 | ✅ Complete |
||| [TASK-1899](tasks/TASK-1899-contract-blame-diagnostics.md) | Emit structured blame diagnostics with snapshots, evidence, and redaction metadata | 8h | TASK-1898 | ✅ Complete |
||| [TASK-1900](tasks/TASK-1900-runtime-monitor-evidence.md) | Wire runtime monitor evidence rows and temporal monitor diagnostics | 8h | TASK-1896, TASK-1899 | ✅ Complete |
||| [TASK-1901](tasks/TASK-1901-contract-evidence-closeout.md) | Close out Phase 194 with fixtures, docs, gates, and review remediation | 6h | TASK-1893 through TASK-1900 | ✅ Complete |

Estimated implementation effort after the plan packet: 80 hours.

## Required Test Families

### Surface Contract Tests

Add parser/typechecker/engine tests that prove:

1. `fn f(x: Int) requires { x > 0 } -> Int { ... }` preserves the precondition;
2. `fn f(x: Int) ensures { result >= x } -> Int { ... }` binds `result` only in postconditions;
3. multiple `requires` and `ensures` clauses preserve source order and stable identities;
4. imported public callable summaries carry contract metadata without exposing private predicate
   helpers as ordinary exports.

### Predicate Well-Formedness Tests

Add fail-closed tests for:

1. operation calls in predicates, such as `PosixFs::exists(path)`;
2. handler/provider installation inside predicates;
3. role/resource/policy admission inside predicates;
4. unstable row-empty observations such as time, randomness, pointer identity, and unsafe force;
5. invalid `old(...)` roots, cross-boundary snapshots, and `result` in `requires`;
6. allowed pure helper predicates with checked public summaries.

### Evidence Row Tests

Add tests for:

1. `by test`, law, proof, runtime monitor, and observation evidence rows remaining requirements;
2. invalid or stale evidence failing closed without converting to authority;
3. statistical/test evidence remaining advisory unless the contract strategy explicitly permits a
   dynamic check or evidence discharge;
4. evidence rows crossing module boundaries with stable evidence identities.

### Runtime And Diagnostics Tests

Add runtime tests for:

1. precondition false traps with caller-side blame;
2. postcondition false traps with callee/impl-side blame;
3. predicate evaluator faults distinct from false predicates;
4. diagnostics preserving boundary id, predicate id, snapshot refs, evidence refs, and redacted
   observation evidence;
5. recoverable behavior only when an explicit failure or compensation row is present.

## Verification Gates

Each implementation task must run focused tests for the touched crates plus:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-Claim Sweep

Before closeout, search live normative docs and code comments for stale claims:

```text
contract predicates may call providers
predicate evaluator receives authority
requires.*operation call
ensures.*operation call
ContractViolation.*row item
ContractViolation.*resumable operation
evidence row grants authority
test evidence proves optimizer rewrite
workflow-only contract system
source text predicate re-evaluation
```

Changelog/history may mention old wording as historical context. Live guidance must route through
row requirements, authority-free predicates, evidence discharge, and structured diagnostics.

## Acceptance Criteria

- [x] Phase 194 plan and task files exist and are indexed.
- [x] Target `fn` contracts parse, preserve, summarize, and lower without workflow syntax.
- [x] Predicate well-formedness rejects authority acquisition and unstable observers.
- [x] Surface contracts lower to Core predicate sidecars, snapshots, runtime check plans, and
  discharge metadata.
- [x] Evidence rows cover tests, laws, proofs, runtime monitors, and observations as requirements.
- [x] Dynamic checks distinguish contract violations from predicate faults.
- [x] Structured diagnostics preserve blame, boundary, predicate, snapshot, evidence, and redaction
  details.
- [x] Changelog and orientation indexes route future contract/evidence work through this phase.
