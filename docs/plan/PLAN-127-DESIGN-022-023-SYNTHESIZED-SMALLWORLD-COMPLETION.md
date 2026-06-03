# PLAN-127: DESIGN-022/023 Synthesized and Small-World Completion

**Status:** Draft
**Spec:** [SPEC-077](../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
**Designs:** [DESIGN-022](../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md), [DESIGN-023](../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)
**Depends on:** Phase 76B narrow structured-snapshot runner substrate
**Task range:** TASK-1012 through TASK-1018

## Goal

Complete DESIGN-022 and DESIGN-023 beyond Phase 76B by adding live checked/lowered snapshot production from ordinary CLI files, end-to-end synthesized contract/policy/obligation execution, real small-world execution against Ash targets, richer finite domains, CLI integration hardening, and broad verification.

## Current Baseline

Phase 76B is complete only for the narrow structured-snapshot slice: injected `RunnerIntrospectionSnapshot` values can produce executable finite metadata-backed cases, raw-source scans defer, obligation lifecycle pass rows evaluate explicit finite lifecycle world-state metadata, uncapped bounded-int worlds defer, and synthesized filters/fail-fast apply to structured results.

TASK-1012 and TASK-1013 extend that baseline so ordinary `ash test` source files can produce live checked runner snapshots, and supported pure `Int` function contract postconditions execute checked/lowered core target and `ensures` expressions through `ash_interp`. String-only raw-source or display metadata remains deferred-skip only.

The following remain future work:
- policy execution is limited to exact `TerminalEquals` metadata
- obligation execution is limited to finite lifecycle metadata equality
- small-world execution does not yet run Ash targets across richer role/capability/policy/obligation worlds

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-1012](tasks/TASK-1012-live-runner-introspection-snapshot-production.md) | Produce checked/lowered runner snapshots from ordinary CLI source files and suite roots | CLI/Typeck/Substrate | 12 | Complete |
| [TASK-1013](tasks/TASK-1013-contract-target-and-postcondition-synthesized-execution.md) | Execute synthesized contract targets and postcondition oracles end to end for supported metadata | Runner/Runtime | 14 | Complete |
| [TASK-1014](tasks/TASK-1014-policy-domain-and-terminal-oracle-execution.md) | Execute policy domains and terminal oracles beyond the narrow metadata equality slice | Runner/Policy | 12 | Complete |
| [TASK-1015](tasks/TASK-1015-runtime-backed-obligation-lifecycle-execution.md) | Execute obligation lifecycle transitions through lowered/runtime-backed semantics | Runner/Runtime | 14 | Planned |
| [TASK-1016](tasks/TASK-1016-smallworld-target-execution.md) | Materialize deterministic worlds and execute Ash targets against each world | Runner/SmallWorld | 16 | Planned |
| [TASK-1017](tasks/TASK-1017-richer-domains-and-cli-integration-hardening.md) | Add richer finite domains and harden synthesized/small-world CLI controls | Runner/CLI | 12 | Planned |
| [TASK-1018](tasks/TASK-1018-design022-023-completion-closeout.md) | Run broad verification and promote DESIGN-022/DESIGN-023 completion status | Closeout | 8 | Planned |

Total estimate: 88h.

## Execution Order

1. TASK-1012 must land first so ordinary CLI inputs can produce structured snapshots.
2. TASK-1013, TASK-1014, and TASK-1015 add source-specific executable synthesized slices over that snapshot.
3. TASK-1016 adds actual world execution against Ash targets.
4. TASK-1017 expands finite domain families and hardens CLI behavior.
5. TASK-1018 reconciles docs/status and runs broad verification before design promotion.

## Decision Gates

- D1: Raw-source compatibility scans never produce pass rows.
- D2: Every pass row must be backed by evaluated target/oracle metadata or real world execution.
- D3: Unbounded generated domains defer before materialization.
- D4: Repro artifacts are mandatory for every executed synthesized/generated/small-world row.
- D5: CLI filtering, source selection, fail-fast, seed, max-cases, max-worlds, timeout, human output, and JSON output stay consistent across authored and synthesized paths.

## Verification Strategy

Each implementation task must follow strict TDD, record RED/GREEN evidence in its task file, and run focused non-zero tests plus:

```bash
cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace
git diff --check
```

Closeout additionally runs:

```bash
CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture
CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture
CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= cargo test --workspace
```

## Completion Checklist

- [ ] Ordinary CLI files produce checked/lowered runner snapshots.
- [x] Contract target/postcondition synthesized execution is implemented for supported cases.
- [x] Policy domain and terminal oracle execution is implemented for supported cases.
- [ ] Obligation lifecycle execution is runtime-backed for supported cases.
- [ ] Small-world execution runs Ash targets against deterministic worlds.
- [ ] Richer finite domains and CLI controls are hardened.
- [ ] DESIGN-022 and DESIGN-023 status and acceptance criteria are promoted only after broad verification.
