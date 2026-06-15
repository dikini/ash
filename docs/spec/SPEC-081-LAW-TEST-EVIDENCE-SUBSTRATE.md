# SPEC-081: Law Test Evidence Substrate

**Status:** Planned
**Date:** 2026-06-15
**Builds on:** [SPEC-077](SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md), [SPEC-078](SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md), [SPEC-080](SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
**Plan:** [PLAN-145](../plan/PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Implementation Tasks:** [TASK-1446](../plan/tasks/TASK-1446-law-test-evidence-no-rust-baseline.md) through [TASK-1455](../plan/tasks/TASK-1455-law-test-evidence-closeout.md)

## 1. Summary

SPEC-081 defines the user-facing law test evidence substrate for `proof ... { by test ... }` declarations. The core rule is that `by test` is an empirical evidence family with three separate submodes:

1. **authored/manual tests** — a proof delegates to a named Ash-authored test or assertion file;
2. **property tests** — the law proposition itself is treated as a property and executed over generated law-parameter bindings;
3. **small-world tests** — the law proposition is executed exhaustively over explicitly finite data/world domains.

This spec deliberately keeps these modes classified under `by test`, leaving future non-test evidence families such as symbolic proof search, solver-backed proofs, and checked proof terms outside this empirical-test family.

The user-facing constraint is normative: a law author, test author, proof author, and proof executor must be able to run the supported evidence modes with an Ash toolchain binary, not with Rust developer tooling. During Ash tooling development, the globally installed `ash` binary may lag behind the worktree. Therefore acceptance is defined against an explicit **Ash-under-test** executable (`$ASH_UNDER_TEST`) produced or selected by the implementation task. The executor invokes only `$ASH_UNDER_TEST test ...`; they must not need `cargo`, `cargo run`, `rustc`, or a Rust toolchain to validate Ash laws and Ash-authored tests. Phase closeout must then prove release/install parity by updating or staging the normal `ash` entrypoint.

## 2. Motivation

Phase 136 added `law`/`proof` syntax. Phase 144 added generated algebra law rows for selected built-in algebra profiles. That implementation is useful infrastructure, but it does not yet satisfy the ordinary library-author workflow:

```text
write an Ash library -> add Ash laws -> add Ash tests/proof evidence -> run ash test -> get fail-closed law evidence
```

The current `ProofBody::ByTest { test_name }` representation is too weak. It records a string label but does not require that the named Ash test exists, passed, or is relevant to the law. It also conflates manual test delegation, property generation, and small-world enumeration.

SPEC-081 splits the concept into structured test evidence modes and defines fail-closed behavior for each mode.

## 3. Non-Rust execution constraint

All acceptance tests for this spec must include final-surface commands that invoke an Ash executable directly, not Cargo. During development, tasks must set or record `$ASH_UNDER_TEST` and use that executable consistently:

```bash
${ASH_UNDER_TEST:?set Ash candidate binary} test path/to/library.ash --include-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test path/to/tests/ash/property --format json
```

`$ASH_UNDER_TEST` may be a staged candidate produced from the current worktree by the implementer, an Ashgrove-installed local toolchain, or the ordinary `ash` entrypoint once it has been updated. It must not be `cargo run -p ash-cli -- ...`, and acceptance output must come from the executable itself. Implementation tasks may still use Rust tooling to build that candidate and to verify compiler/runner internals, but the feature is not complete until an Ash executable can run both sides of the workflow:

- the **law side**: parse/check/extract law and proof declarations from `.ash` source;
- the **test side**: discover/execute Ash-authored tests, generated property cases, or finite small-world cases;
- the **linkage side**: connect `by test ...` evidence to the law proof result fail-closed.

## 4. Evidence taxonomy

The intended internal model is:

```text
ProofEvidence
├── ByDefinition
├── ByTest(TestEvidence)
├── BySymbolicProof(...)      -- future, not part of SPEC-081
├── BySolver(...)             -- future, not part of SPEC-081
└── ByProofTerm(...)          -- future, not part of SPEC-081

TestEvidence
├── Authored { test_name }
├── Property { generators, seed, max_cases }
└── SmallWorld { domains, max_worlds }
```

The MVP may represent this as runner-facing metadata before fully replacing parser/typechecker public enums, but the runner output must distinguish the three modes.

## 5. Surface syntax

### 5.1 Backward-compatible authored shorthand

Existing syntax remains accepted and is classified as authored/manual test evidence:

```ash
proof identity(value: Option<Int>) {
    by test "option_functor_identity"
}
```

Equivalent structured meaning:

```text
ByTest(Authored { test_name: "option_functor_identity" })
```

### 5.2 Explicit authored spelling

The implementation may add an explicit spelling for clarity:

```ash
proof identity(value: Option<Int>) {
    by test authored "option_functor_identity"
}
```

If the explicit spelling is deferred, implementations must still treat `by test "..."` as authored evidence rather than generic string metadata.

### 5.3 Property evidence

Property evidence treats the law proposition as the test oracle and generated law-parameter bindings as test cases.

Example direction:

```ash
proof identity(value: Option<Int>) {
    by test property {
        value: option(int(-3, 3))
        cases: 100
        seed: 42
    }
}
```

MVP syntax may initially use metadata descriptors rather than rich generator expressions, but the result must be the same: generated values are bound to law parameters and the law proposition is evaluated for each case.

### 5.4 Small-world evidence

Small-world evidence exhaustively enumerates explicitly finite domains or worlds.

Example direction:

```ash
proof associativity(a: BoolOp, b: BoolOp, c: BoolOp) {
    by test small_world {
        a: values [Zero, One]
        b: values [Zero, One]
        c: values [Zero, One]
    }
}
```

Open domains, uncapped domains, unsupported value constructors, and unsupported runtime worlds must defer or fail closed; they must not pass by omission.

## 6. Authored test evidence semantics

For authored evidence:

1. The runner builds a stable registry of Ash-authored test names.
2. `by test "name"` resolves against that registry.
3. Duplicate authored test names are an error.
4. Missing delegated tests are invalid evidence.
5. Skipped/deferred delegated tests do not satisfy the proof.
6. Failing delegated tests make the proof broken.
7. Passing delegated tests satisfy the empirical proof evidence.

A minimal authored test may be declared with current metadata:

```ash
-- @test name: option_functor_identity
-- @test kind: property
-- @test proves: std::option.Functor.identity
workflow main() -> Bool {
    let value = Some { value: 1 };
    ret map(value, |x| -> x) == value
}
```

The `proves` metadata is optional in the first resolver slice but should become mandatory before accepting arbitrary authored tests as proof-relevant.

## 7. Property evidence semantics

For property evidence:

1. The law proposition is the oracle.
2. Law parameters are bound from generated case data.
3. Each generated case evaluates the proposition in the same checked context used for the law.
4. The first counterexample fails the proof and records a repro artifact.
5. If any parameter lacks a supported generator, the proof evidence is deferred or invalid, not satisfied.
6. Shrinking is not required for the first implementation, but result models must leave room for a future minimal/shrunk counterexample.

Supported first-slice domains should be small and honest:

- `Bool`
- bounded `Int`
- small `String`
- explicit finite `values [...]`
- `Option<T>` where `T` has a supported finite/bounded generator
- `Result<T, E>` where both sides have supported finite/bounded generators
- small `List<T>` with bounded length, if the list substrate is available

## 8. Small-world evidence semantics

For small-world evidence:

1. The law proposition is the oracle.
2. Every value/world in the declared finite domain is enumerated up to the configured `max_worlds` cap.
3. Exhaustive means exhaustive only relative to the declared finite domain.
4. If the finite domain cannot be materialized safely, the evidence is deferred or invalid.
5. Repro artifacts record the failing world index and full binding/world snapshot.

Small-world mode is least practical for broad data, but useful for finite ADTs, tiny carriers, bounded protocol/state-machine worlds, policy contexts, and explicitly enumerated examples.

## 9. Result model

The proof evidence layer should preserve more precise statuses than the existing CLI outcome enum:

| Evidence status | Meaning | CLI row mapping |
|---|---|---|
| `satisfied` | Evidence ran and supports the proof | `pass` |
| `broken` | Evidence ran and found a failure/counterexample | `fail` |
| `invalid_evidence` | Referenced test/generator/domain is missing or malformed | `error` or `fail` |
| `deferred` | Evidence mode is recognized but unsupported metadata/substrate is missing | `skip` |
| `untested` | Evidence did not run | `skip` |

JSON output must include at least:

```json
{
  "law": "identity",
  "proof": "identity",
  "evidence_family": "test",
  "test_mode": "authored|property|small_world",
  "evidence_status": "satisfied|broken|invalid_evidence|deferred|untested"
}
```

## 10. CLI behavior

Synthesized law evidence must remain explicit and opt-in. Existing source selectors may be preserved, but they must include laws in the Ash-under-test binary and, by closeout, in the installed/staged user entrypoint:

```bash
${ASH_UNDER_TEST:?set Ash candidate binary} test library.ash --include-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test library.ash --only-synthesized laws --format json
```

Compatibility aliases such as `--include-law-tests` may continue to exist, but documentation should prefer the general synthesized-source selector once `laws` is supported in the candidate and installed entrypoints.

## 11. Deferred features

The following are explicitly out of scope for SPEC-081:

- symbolic proof search;
- SMT/solver-backed proofs;
- checked proof terms beyond existing proof-body scaffolding;
- full shrinking/minimization;
- arbitrary runtime `Act`/`Proc`/`Workflow` equivalence without bounded equivalence metadata;
- automatic proof relevance checking beyond initial `proves` metadata or explicit law IDs.

## 12. Acceptance criteria

SPEC-081 is implemented when:

1. `by test "name"` resolves to an Ash-authored test and fails closed when missing.
2. Authored, property, and small-world test evidence are represented as distinct modes in runner metadata and JSON output.
3. At least one authored/manual law proof is satisfied by an Ash test using only `ash test`.
4. At least one property-mode law proof evaluates a law proposition over generated bindings using only `ash test`.
5. At least one small-world law proof evaluates a law proposition over all values in an explicit finite domain using only `ash test`.
6. Unsupported generators/domains/proposition forms report explicit `invalid_evidence` or `deferred` statuses rather than pass.
7. `$ASH_UNDER_TEST` supports `--include-synthesized laws` and `--only-synthesized laws`, and closeout demonstrates release/install parity for the ordinary `ash` entrypoint; no user-facing acceptance command uses `cargo run` or a Rust test harness.
8. Reference documentation clearly distinguishes empirical test evidence from future symbolic/solver/proof-term evidence.
