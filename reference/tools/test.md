---
id: ref.tools.test
title: Ash Test
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: cli
last_verified: 2026-06-03
verified_against:
  git_commit: 7cf576d
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-005-CLI.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
    - docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md
  tasks:
    - docs/plan/tasks/TASK-509-ash-test-runner-substrate.md
    - docs/plan/tasks/TASK-512-authored-test-metadata-and-execution-model.md
    - docs/plan/tasks/TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md
    - docs/plan/tasks/TASK-514-property-and-smallworld-execution.md
    - docs/plan/tasks/TASK-515-ash-test-runner-docs-and-phase-verification.md
    - docs/plan/tasks/TASK-1010-phase-76b-rescope-spec-hardening-packet.md
    - docs/plan/tasks/TASK-1011-phase-76b-final-remediation-and-design022-023-planning.md
    - docs/plan/tasks/TASK-1019-reference-ash-test-daily-use.md
  code:
    - crates/ash-cli/src/commands/test.rs
    - crates/ash-cli/src/main.rs
    - crates/ash-cli/src/test_runner/discovery.rs
    - crates/ash-cli/src/test_runner/executor.rs
    - crates/ash-cli/src/test_runner/metadata.rs
    - crates/ash-cli/src/test_runner/output.rs
    - crates/ash-cli/src/test_runner/property.rs
    - crates/ash-cli/src/test_runner/synthesized.rs
    - crates/ash-cli/src/test_runner/types.rs
  tests:
    - cargo run -p ash-cli -- test --help
    - check_frontmatter pilot validation
    - check_frontmatter full reference validation
    - git diff --check
  examples:
    []
related:
  depends_on:
    - ref.tools.cli
  explains:
    - ref.status.alpha_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md
    - docs/design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md
refresh_trigger:
  - docs/spec/SPEC-005-CLI.md changes
  - docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md changes
  - crates/ash-cli/src/commands/test.rs changes
  - crates/ash-cli/src/test_runner/** changes
  - reference/tools/test.md changes
---

# Ash Test

`ash test` runs Ash-authored test files and, when explicitly requested, reports synthesized compatibility rows from contract, policy, or obligation sources.

Live help checked:

```bash
cargo run -p ash-cli -- test --help
```

Help-derived installed form:

```bash
ash test [OPTIONS] [PATH]
```

`PATH` defaults to the current directory. Command snippets below use placeholders and are reference forms, not guaranteed copy/paste examples.

## Daily Forms

```bash
ash test
ash test tests/ash/unit/example.ash
ash test tests/ash/property --seed 42 --max-cases 100
ash test --tag stdlib --kind unit --format json
ash test --include-synthesized contracts,policies,obligations
```

Use `cargo run -p ash-cli -- test ...` from a checkout before installing Ash. After installation, use the same shapes as `ash test ...`.

## Authored Test Discovery

If `PATH` is a `.ash` file, the runner runs that file. If `PATH` is a directory, the runner discovers `.ash` files under conventional roots:

- `tests/ash/unit`
- `tests/ash/integration`
- `tests/ash/e2e`
- `tests/ash/property`
- `tests/ash/smallworld`

Passing one of the kind directories, or `tests/ash`, runs `.ash` files from that directory tree. Files directly under `tests/ash` are also discovered when running from a project root.

Authored tests parse, type check, and execute through the engine. A returned `false` is a failed test. Parse/type/runtime infrastructure problems are reported as `error`, and caught panics are reported as `panic`.

## Metadata Directives

Authored metadata is read from leading comment lines using either `//` or `--`. Each directive line currently starts with `@test`; the older block shorthand with bare `-- name:` lines is not the implemented parser shape.

```ash
-- @test name: option unwrap_or returns default
-- @test kind: unit
-- @test tags: stdlib, option
-- @test timeout_ms: 1000
-- @test seed: 42
-- @test max_cases: 25
-- @test max_worlds: 10
```

Supported keys are `name`, `kind`, `tags`, `timeout_ms`, `capabilities`, `seed`, `max_cases`, `max_worlds`, and `xfail`. `tags` and `capabilities` are comma-separated. `xfail` can be written as a flag:

```ash
// @test xfail
```

Current Alpha boundary: `capabilities` metadata is parsed but is not a full capability-admission setup surface for the test runner.

## Filtering

`--tag TAG` selects authored tests whose metadata tags contain exactly `TAG`. For authored tests, non-matching files are reported as skipped rows.

`--kind KIND` accepts the current kind labels:

- `unit`
- `integration`
- `e2e`
- `property`
- `smallworld`

Kind comes from `@test kind:` when present, otherwise from the discovery path. Unknown kind metadata falls back to unit behavior in the current runner.

For structured synthesized rows, tag and kind filters select matching generated rows. Raw-source fallback rows are unit-kind deferred skips.

## Output and Global Controls

`--format human` is the default. Human output lists each result with outcome, name, source label for synthesized rows, kind, duration, and message.

`--format json` emits a stable runner JSON envelope with `schema_version: "ash-test-v1.0"`, suite counts, durations, per-test outcome/source/kind/tags, and repro artifacts when present.

Help also exposes global controls on `ash test`:

```bash
ash test --quiet
ash test --color never
ash test -vv
```

`--color` controls colored human output. `-v` initializes CLI tracing verbosity. `--quiet` suppresses CLI error printing at the top level; the current `ash test` formatter still emits its selected human or JSON suite output on successful command execution.

## Failure Control and Timeouts

```bash
ash test --fail-fast
ash test --timeout 5000
```

`--fail-fast` stops authored execution after the first failing authored row. It also applies to structured synthesized rows.

Current Alpha boundary: when authored and synthesized rows are both requested, an authored fail-fast stop does not suppress the later synthesized phase.

`--timeout` is in milliseconds. The default is `30000`. File metadata `@test timeout_ms:` overrides the command default for that authored test.

## Property and Small-World Knobs

```bash
ash test tests/ash/property --seed 9001 --max-cases 50
ash test tests/ash/smallworld --max-worlds 20
```

For authored property tests, `--seed` is recorded for reproducibility and `--max-cases` bounds repeated execution. The current authored property path reruns the `.ash` body; it does not yet generate arbitrary input values from ordinary source.

For authored small-world tests, `--max-worlds` bounds repeated execution and reports the failing world index. True world enumeration is currently available only through the structured snapshot path described below.

## Synthesized Controls

Synthesized tests are never enabled by default.

```bash
ash test --include-synthesized contracts
ash test --include-synthesized contracts,policies,obligations
ash test --only-synthesized policies --format json
```

Sources are a comma-separated list of `contracts`, `policies`, and `obligations`.

`--include-synthesized` runs authored tests and then selected synthesized sources. `--only-synthesized` skips authored tests and implies synthesized inclusion.

### Current Execution Boundary

Phase 76B supports executable synthesized rows only for the runner-internal structured snapshot slice:

- contract `requires` boundary cases over exact finite representatives;
- policy `TerminalEquals` allow/deny metadata;
- obligation lifecycle rows with explicit finite lifecycle world state;
- metadata-backed generated property rows with exact finite values;
- explicit finite small-world states, explicit values, bool, and safely capped bounded integers.

Ordinary raw source files passed to the CLI do not yet produce live checked/lowered `RunnerIntrospectionSnapshot` values. Raw-source synthesized compatibility rows are deferred skips with messages such as missing lowered executable metadata; they are not live lowered synthesized executions and must not be treated as full contract, policy, obligation, property, or small-world passes.

Broader live snapshot production, contract postcondition target execution, richer policy and obligation execution, richer finite domains, and Ash target execution across materialized worlds are deferred to SPEC-077 / Phase 132 follow-up tasks.

## Non-Goals

`ash test` does not currently provide coverage reporting, mutation testing, shrinking, flaky-test quarantine, distributed orchestration, proof-producing synthesis, or unrestricted automatic value/world generation from ordinary source files.
