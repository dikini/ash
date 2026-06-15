# PLAN-145: Law Test Evidence Substrate

> **For Hermes/orchestrators:** This phase is about making `by test` real for Ash users, not adding more Rust-only integration assertions. Because Ash tooling is itself under development, do not assume the globally installed `ash` is current. Every implementation task must identify an Ash-under-test executable (`$ASH_UNDER_TEST`) and end with at least one final-surface `$ASH_UNDER_TEST test ...` command that does not invoke Cargo or Rust tooling. Use Rust/Cargo gates as implementation health checks, but do not count the feature complete without no-Rust Ash CLI evidence from the candidate executable and closeout parity for the normal `ash` entrypoint.

## Phase: 145

## Status: ✅ Complete

## Spec

- [SPEC-081: Law Test Evidence Substrate](../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)

## Goal

Turn `proof ... { by test ... }` from string metadata into fail-closed empirical evidence that can be produced and checked entirely from Ash source with an Ash toolchain binary, using an explicit `$ASH_UNDER_TEST` candidate during development and proving normal `ash` entrypoint parity at closeout.

The phase accounts for three related but separate `by test` use types:

1. **authored/manual** — the user writes expected tests/assertions manually and a proof delegates to a named Ash-authored test;
2. **property** — the law specification itself is the property and the runner generates bindings for law parameters;
3. **small-world** — the law specification is checked exhaustively over explicit finite data/world permutations.

Symbolic/solver-backed proofs are intentionally not part of this phase. They remain future non-test proof evidence modes.

## Non-Rust execution contract

The final user-facing workflow must work with an Ash executable, not a Rust developer command. During implementation, each task must set or record `$ASH_UNDER_TEST`:

```bash
${ASH_UNDER_TEST:?set Ash candidate binary} test examples/laws/my_library.ash --include-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test tests/ash/property --format json
```

`$ASH_UNDER_TEST` may point to a candidate binary produced from the current worktree or installed through Ashgrove/toolchain staging. The closeout task must additionally prove that the ordinary `ash` entrypoint has been updated or explicitly document the remaining release/install handoff. The law/test/proof author and executor must not need:

- `cargo run`
- `cargo test`
- `rustc`
- a Rust source checkout
- a local debug binary hidden in `target/debug` unless it is explicitly promoted to `$ASH_UNDER_TEST` with provenance, path, version, and release/install-parity handoff recorded

Implementation agents may use Rust tooling internally to build and test the candidate, but each task with user-facing behavior must include no-Cargo smoke evidence from `$ASH_UNDER_TEST` itself. `cargo run -p ash-cli -- test ...` is never final-surface evidence.

## Scope

### In scope

- Structured `ProofEvidence::ByTest(TestEvidence)` metadata.
- Backward-compatible classification of `by test "name"` as authored test evidence.
- Authored test registry and duplicate-name detection.
- Fail-closed resolver from proofs to authored Ash tests.
- Runner result model for `satisfied`, `broken`, `invalid_evidence`, `deferred`, and `untested`.
- Minimal `by test property` metadata/syntax and generated law-parameter bindings.
- Minimal `by test small_world` metadata/syntax and finite explicit domain enumeration.
- Final-surface fixtures proving laws and tests run through `ash test` without Cargo.
- Documentation/reference updates that distinguish authored, property, small-world, and future symbolic proof evidence.

### Out of scope

- SMT/symbolic proof search.
- Checked proof terms beyond existing proof scaffolding.
- Full shrinking/minimization.
- Broad `Act`/`Proc`/`Workflow` law equivalence without bounded equivalence metadata.
- Arbitrary generator inference for all Ash types.
- Hidden implicit synthesized tests enabled by default.

## Architecture

```text
Ash source
  ├─ law declarations
  ├─ proof declarations
  │    └─ by test <mode>
  └─ authored tests / generator metadata / small-world metadata
        │
        ▼
Parser + Typechecker
  ├─ preserve structured proof evidence mode
  ├─ validate proof names against laws
  └─ keep unsupported evidence explicit
        │
        ▼
Test runner
  ├─ authored test registry
  ├─ law proof resolver
  ├─ authored test executor
  ├─ law-as-property executor
  └─ small-world executor
        │
        ▼
LawProofResult
  ├─ evidence_family = test
  ├─ test_mode = authored | property | small_world
  ├─ evidence_status = satisfied | broken | invalid_evidence | deferred | untested
  └─ repro artifacts
```

## Workstreams

| Stream | Tasks | Can run in parallel? | Notes |
|---|---|---|---|
| A. Baseline + evidence model | TASK-1446, TASK-1447 | sequential | Establish no-Rust gates and shared metadata first. |
| B. Surface + authored mode | TASK-1448, TASK-1449, TASK-1450 | mostly sequential | Parser/AST before resolver; registry can start after model. |
| C. Property mode | TASK-1451, TASK-1452 | sequential | Proposition executor before generator binding integration. |
| D. Small-world mode | TASK-1453 | after TASK-1451 | Reuses proposition executor. |
| E. Final-surface fixtures + closeout | TASK-1454, TASK-1455 | sequential | Proves both law side and test side without Cargo. |

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1446](tasks/TASK-1446-law-test-evidence-no-rust-baseline.md) | Audit and freeze no-Rust CLI baseline for law/test evidence | 4 | Phase 144 | ✅ Complete |
| [TASK-1447](tasks/TASK-1447-structured-law-test-evidence-model.md) | Add structured test evidence metadata and result statuses | 5 | TASK-1446 | ✅ Complete |
| [TASK-1448](tasks/TASK-1448-by-test-submode-parser-ast.md) | Parse/preserve authored/property/small-world `by test` submodes | 5 | TASK-1447 | ✅ Complete |
| [TASK-1449](tasks/TASK-1449-authored-test-registry.md) | Build stable authored Ash test registry with duplicate detection | 5 | TASK-1447 | ✅ Complete |
| [TASK-1450](tasks/TASK-1450-authored-by-test-resolver.md) | Resolve `by test "name"` to authored tests fail-closed | 6 | TASK-1448, TASK-1449 | ✅ Complete |
| [TASK-1451](tasks/TASK-1451-law-proposition-executor.md) | Execute supported law propositions over explicit bindings | 6 | TASK-1447 | ✅ Complete |
| [TASK-1452](tasks/TASK-1452-by-test-property-generators.md) | Implement minimal `by test property` generators and binding injection | 6 | TASK-1448, TASK-1451 | ✅ Complete |
| [TASK-1453](tasks/TASK-1453-by-test-small-world-domains.md) | Implement minimal `by test small_world` finite domain enumeration | 6 | TASK-1448, TASK-1451 | ✅ Complete |
| [TASK-1454](tasks/TASK-1454-no-rust-final-surface-law-fixtures.md) | Add final-surface Ash law/test fixtures and no-Cargo smoke gates | 5 | TASK-1450, TASK-1452, TASK-1453 | ✅ Complete |
| [TASK-1455](tasks/TASK-1455-law-test-evidence-closeout.md) | Closeout: docs, reference, PLAN-INDEX, changelog, broad verification | 4 | TASK-1454 | ✅ Complete |

**Total estimated effort:** 52 hours.

**Implementation status:** Complete in this worktree. Final-surface evidence used `$ASH_UNDER_TEST=$PWD/target/debug/ash` produced by `cargo build -p ash-cli`; ordinary installed `ash` parity remains a release/install handoff because the globally installed binary may lag active toolchain development.

## Orchestrator guidance

1. **Do not start with property generators.** First make `by test "name"` fail closed against authored Ash tests. This gives immediate practical value and prevents string-label false positives.
2. **Preserve authored vs synthesized separation.** Synthesized law rows remain opt-in. Authored tests remain authored. Proof results may reference both, but output must label source/mode clearly.
3. **Run no-Cargo smoke commands in every user-facing task.** If an implementation only passes `cargo test`, it is not complete. Use `$ASH_UNDER_TEST test ...`; do not assume `/home/dikini/.local/bin/ash` is current until TASK-1446 proves it.
4. **Keep tasks small and sequential at semantic seams.** Avoid one mega-agent changing parser, typechecker, runner, fixtures, and docs in a single task.
5. **Defer honestly.** Unsupported generator/domain/proposition shapes must return `invalid_evidence` or `deferred`, never pass.
6. **Check installed/staged binary drift.** If `/home/dikini/.local/bin/ash` lags behind the current worktree, record the drift and set `$ASH_UNDER_TEST` to the candidate executable. The task closeout must record how that candidate was produced, its version/path, and how release/install parity will be restored.

## Verification gates

### Required no-Rust user gates

Each implementation task that changes runner behavior must include a final-surface gate using `$ASH_UNDER_TEST` directly:

```bash
${ASH_UNDER_TEST:?set Ash candidate binary} test <fixture>.ash --include-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test <fixture>.ash --only-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test <fixture-tests-dir> --format json
```

The command must not be `cargo run -p ash-cli -- test ...`. The globally installed `ash` is a parity target, not an assumption.

### Implementation health gates

Implementation tasks should also run focused Rust health gates while developing:

```bash
cargo fmt --check
cargo test -p ash-cli <focused_filter> -- --nocapture
cargo clippy -p ash-cli --all-targets -- -D warnings
```

The closeout task owns broad workspace verification.

## Acceptance criteria

- [ ] `by test "name"` resolves to an authored Ash test and fails closed when missing.
- [ ] Duplicate authored test names are rejected or reported as invalid evidence.
- [ ] Authored test pass/fail/skip propagates to law proof evidence status.
- [ ] `by test property` executes a law proposition over generated bindings.
- [ ] `by test small_world` executes a law proposition over explicit finite domains.
- [ ] JSON output distinguishes `test_mode` and `evidence_status`.
- [ ] User-facing law/test/proof examples run with `$ASH_UNDER_TEST test` and no Cargo; closeout proves or explicitly hands off ordinary `ash` entrypoint parity.
- [ ] Docs clearly reserve symbolic/solver proofs as future non-test evidence families.

## References

- [SPEC-081](../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
- [SPEC-077](../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [SPEC-080](../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [reference/tools/test.md](../../reference/tools/test.md)
- [reference/stdlib/algebra.md](../../reference/stdlib/algebra.md)
