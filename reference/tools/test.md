---
id: ref.tools.test
title: Ash Test
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: cli
last_verified: 2026-06-16
verified_against:
  git_commit: 7cf576d
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-005-CLI.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
    - docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md
    - docs/spec/SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md
    - docs/spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md
  tasks:
    - docs/plan/tasks/TASK-509-ash-test-runner-substrate.md
    - docs/plan/tasks/TASK-512-authored-test-metadata-and-execution-model.md
    - docs/plan/tasks/TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md
    - docs/plan/tasks/TASK-514-property-and-smallworld-execution.md
    - docs/plan/tasks/TASK-515-ash-test-runner-docs-and-phase-verification.md
    - docs/plan/tasks/TASK-1010-phase-76b-rescope-spec-hardening-packet.md
    - docs/plan/tasks/TASK-1011-phase-76b-final-remediation-and-design022-023-planning.md
    - docs/plan/tasks/TASK-1019-reference-ash-test-daily-use.md
    - docs/plan/tasks/TASK-1505-quickcheck-v1-final-surface-fixtures-and-docs.md
  code:
    - crates/ash-cli/src/commands/test.rs
    - crates/ash-cli/src/main.rs
    - crates/ash-cli/src/test_runner/discovery.rs
    - crates/ash-cli/src/test_runner/executor.rs
    - crates/ash-cli/src/test_runner/metadata.rs
    - crates/ash-cli/src/test_runner/output.rs
    - crates/ash-cli/src/test_runner/coverage_mutation.rs
    - crates/ash-cli/src/test_runner/property.rs
    - crates/ash-cli/src/test_runner/quickcheck.rs
    - crates/ash-cli/src/test_runner/evidence_cache.rs
    - crates/ash-cli/src/test_runner/synthesized.rs
    - crates/ash-cli/src/test_runner/types.rs
  tests:
    - crates/ash-cli/tests/phase150_quickcheck_metadata.rs
    - crates/ash-engine/tests/phase151_quickcheck_stdlib.rs
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

`ash test` runs Ash-authored test files and, when explicitly requested, runs bounded synthesized rows from contract, policy, obligation, generated-property, and small-world metadata. Unsupported raw-source or open-domain cases are reported as deferred skips rather than passes.

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

For authored small-world tests, `--max-worlds` bounds repeated execution and reports the failing world index. For synthesized small-world rows, `--max-worlds` bounds actual materialized finite worlds from the checked/structured metadata path described below.

## Synthesized Controls

Synthesized tests are never enabled by default.

```bash
ash test --include-synthesized contracts
ash test --include-synthesized contracts,policies,obligations
ash test --only-synthesized policies --format json
```

Sources are a comma-separated list of `contracts`, `policies`, `obligations`, and `laws`.

`--include-synthesized` runs authored tests and then selected synthesized sources. `--only-synthesized` skips authored tests and implies synthesized inclusion.

### Current Execution Boundary

Phase 132 supports a bounded executable MVP for structured synthesized rows:

- live checked/lowered `RunnerIntrospectionSnapshot` production from ordinary CLI source files for supported pure-function contract metadata;
- contract `requires` boundary cases and supported pure `Int` function postconditions over checked/lowered target and `ensures` expressions;
- policy terminal-oracle rows over exact finite metadata, with unsupported approval/transform slices deferred unless a stable oracle is available;
- obligation lifecycle rows over narrow typed transition metadata;
- metadata-backed generated property rows with exact finite values;
- deterministic small-world execution over explicit finite worlds, including explicit states/values, bool, safely capped bounded integers, bounded products/lists, role/capability inclusion sets, policy-context descriptors, and obligation-lifecycle descriptors.

Raw-source compatibility scans, unsupported setup, open or uncapped domains, arbitrary capability/Act/workflow execution, symbolic exploration, and full runtime policy/capability semantics remain deferred skips rather than passes.

## Law Test Evidence

`proof ... { by test ... }` is empirical test evidence. Phase 145 classifies it into three submodes:

```ash
proof p(x: Int) { by test authored "authored_test_name" }
proof p(x: Int) { by test property }
proof p(x: Bool) { by test small_world }
```

Authored evidence resolves the named Ash-authored test from the discovered test registry. Missing, duplicate, skipped, errored, or failing authored tests do not satisfy the proof; they report `invalid_evidence` or `broken` in JSON output. Property evidence treats the law proposition as the property oracle and evaluates supported primitive and container law parameters (`Int`, `Bool`, `String`, `List<T>`, `Option<T>`, and `Result<T, E>` for supported nested `T`/`E`) over bounded generated bindings. Small-world evidence evaluates the law proposition over finite primitive domains and reports `world_index`/world snapshots.

Phase 146 also supports generated bindings for Ash-authored property tests via metadata directives:

```ash
-- @test name: authored_generated_reflexive
-- @test kind: property
-- @test params: x: Int, xs: List<Int>, opt: Option<Int>
-- @test property: x == x
pub fn main() -> Bool { true }
```

For generated property rows, JSON `repro_artifact.generated_input_snapshot` includes the original `bindings`, generator descriptors, and `shrunk_counterexample` / `shrink_trace` when a counterexample is found. Unsupported parameter domains or malformed property oracles fail closed as `error` for authored property tests, and defer rather than pass for synthesized law evidence.

## QuickCheck Strategies, Seeds, Shrinking, and Evidence

Phase 151 updates the QuickCheck model toward ordinary Ash strategy values while
keeping the metadata bridge as the current runnable alpha surface. Conceptually:

```ash
pub type Strategy<T> = Strategy {
    gen: GenContext -> T,
    shrink: T -> List<T>,
}

pub interface Arbitrary<T> {
    arbitrary() -> Strategy<T>
}
```

`Arbitrary<T>` is the default generated-domain evidence for a type. An explicit
strategy override selects a `Strategy<T>` for one generated parameter. The
target v1 parser form is `by test quickcheck with { x <- strategy expr }`, but
the current runnable alpha path still encodes overrides in `@test strategy`
metadata and canonicalizes Phase 150 aliases to the v1 namespace.

Canonical strategy descriptor paths use submodules in runner metadata. These
names are accepted in `@test strategy` today; they are not yet source-visible
stdlib function paths until the deferred module split/parser tasks land:

```ash
-- @test name: sorted_binary_search_domain
-- @test kind: property
-- @test max_cases: 4
-- @test params: xs: List<Int>, x: Int
-- @test strategy xs: test::quickcheck::list::sorted_ints
-- @test strategy x: test::quickcheck::int::positive
-- @test property: x >= 1
fn main() -> Bool { true }
```

Alpha metadata aliases such as `test::quickcheck::positive_ints` and
`test::quickcheck::sorted_int_lists` are accepted for migration, but reference
material should prefer `test::quickcheck::int::positive` and
`test::quickcheck::list::sorted_ints`. If no override is supplied, the runner
uses the bounded default `Arbitrary<T>` representative domain for supported
primitive/container types only when the source explicitly imports
`test::quickcheck::{Arbitrary}` or `test::quickcheck::prelude`. Missing
in-scope default evidence, unsupported domains, unknown override bindings,
duplicate overrides, and strategy/type mismatches fail closed as `error` rows;
they never count as passing evidence.

The target v1 strategy library includes `qc::combinator::one_of`,
`one_of_weighted`, `map`, `map_project`, `map2`, `recursive_with`,
`with_shrink`, `append_shrink`, and `prepend_shrink`. In the current runnable
alpha, these semantics are represented in the runner-side strategy descriptor
and shrink/evidence model while full source-visible stdlib module splitting and
parser/typechecker support for ordinary strategy expressions remain Phase 151
planned work. Treat recursive/weighted source examples as design examples until
TASK-1498/TASK-1501 are reopened and completed.

QuickCheck v1 records the RNG contract as `ash-quickcheck-rng-v1`. Seeds are
random by default and always recorded in JSON repro artifacts. A CLI/replay seed
overrides source `@test seed` metadata. Source-pinned seeds are allowed for
compatibility but produce a warning because they reduce exploration. Source
`@test max_cases` is exact for the authored property; CLI `--max-cases` only
fills in when the source is silent.

Generated-property pass rows include a `generated_input_snapshot` with
`schema_version: "ash-quickcheck-run-v1"`, executed/requested cases, effective
seed, `rng_algorithm`, generator descriptors, and
`aggregate_summary: "empirical_pass_history"`. Counterexample repro artifacts
record `failure_class: "property_false"`, `shrunk_counterexample`,
`shrink_trace`, and `shrink_order_policy: "preserve_order_no_dedup"`.

Strategy shrinking is domain-preserving: explicit strategy overrides shrink via
the strategy's own representative domain before using generic structural
shrinking. For example, `test::quickcheck::int::positive` does not shrink a
failing positive value to `0`, because `0` is outside that strategy domain. The
runner preserves candidate order exactly and does not deduplicate.

Law and QuickCheck evidence schemas are version-moderated. Missing or stale
empirical law evidence is distinct from refuted law/property evidence. Phase 151
adds aggregate QuickCheck run records with compatible pass history, sticky
counterexample/error findings, exact case-count buckets, and same-seed
nondeterminism detection; a later compatible pass does not clear an active
counterexample or generator/error finding.

Law evidence rows include:

- `evidence_family: "test"`
- `test_mode: "authored" | "property" | "small_world"`
- `evidence_status: "satisfied" | "broken" | "invalid_evidence" | "deferred" | "untested"`

The final user-facing command is an Ash executable invocation, for example:

```bash
${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/phase145-law-test-evidence --only-synthesized laws --format json
```

`cargo run -p ash-cli -- test ...` is not final-surface law/test/proof evidence.

## Law Coverage and Mutation Reports

Phase 147 adds opt-in law/test coverage and bounded mutation reporting. These reports are suite-level additions to the existing `ash-test-v1.0` envelope; they do not change ordinary pass/fail classification and are not emitted unless requested.

```bash
ash test fixtures/phase147-coverage --coverage --format json
ash test fixtures/phase147-coverage --mutation --mutation-limit 20 --format json
```

`--coverage` scans Ash source files under the requested path for runner-visible law metadata and reports:

- `coverage.schema_version: "ash-law-coverage-v1.0"`
- `coverage.totals.laws`, `covered_laws`, and `uncovered_laws`
- per-law rows with `id`, `name`, `scope`, `proposition`, `evidence_kind`, `evidence_status`, and optional `evidence_target`
- an `uncovered_laws` subset for downstream agents

`--mutation` emits a bounded first-slice mutation report over discovered law propositions:

- `mutation.schema_version: "ash-mutation-v1.0"`
- `mutation.limit` and aggregate `generated`, `killed`, `survived`, `deferred`, and `errored` totals
- per-mutant rows with source law, operator, original/replacement proposition summaries, status, and an Ash replay command hint

Current Alpha boundary: mutation status is based on the Phase 145/146 empirical law-evidence substrate for the selected law proposition. Covered laws kill their bounded law-proposition mutants; laws without satisfied evidence report survived mutants rather than being counted as passing evidence. This is not unrestricted expression mutation, distributed execution, symbolic proof, or automatic open-domain generator synthesis.

## Flaky Tests, Quarantine, Shards, and Merge

Phase 148 adds local orchestration features for `ash test` while keeping evidence explicit and machine-readable.

```bash
ash test fixtures/phase148-flakes --retries 2 --format json
ash test fixtures/phase148-shards --shard 1/2 --format json > shard-1.json
ash test fixtures/phase148-shards --shard 2/2 --format json > shard-2.json
ash test --merge-results shard-1.json shard-2.json --format json
```

`--retries N` retries failing authored test rows up to `N` times. A row that fails before eventually passing remains a visible pass row with:

- `attempts`: one row per attempt, including outcome/message/duration
- `flake.schema_version: "ash-flake-v1.0"`
- `flake.status: "flaky"`
- suite-level `flake_summary` totals

Test metadata supports quarantine with a required reason:

```ash
-- @test quarantine: known flaky runtime fixture
```

A quarantined row is still emitted, but it is remapped to `skip` with `quarantine.original_outcome` and the human reason. Empty quarantine metadata fails closed as an `error` row.

`--shard INDEX/TOTAL` uses one-based deterministic local shard selection over the sorted discovered authored-test list and emits:

- `shard.schema_version: "ash-shard-v1.0"`
- `shard.index`, `shard.total`, `candidate_count`, `selected_count`, `skipped_count`
- per-row `shard.index`, `shard.total`, and sorted-list ordinal

`--merge-results FILE...` reads shard JSON files without rerunning tests. It rejects invalid shard ranges, failed shard envelopes, missing tests arrays, duplicate shard IDs, missing shard IDs, and duplicate `(path, name)` test rows before producing aggregate success, then emits `merge.schema_version: "ash-merge-v1.0"`.

Current Alpha boundary: these are local orchestration primitives, not remote worker lifecycle management, queueing, artifact upload, or hosted distributed execution. `--shard` currently applies only to authored test discovery and fails closed when combined with synthesized-test selection.

## Non-Goals

`ash test` does not currently provide remote distributed worker lifecycle management, queueing, artifact upload, proof-producing synthesis, unrestricted mutation execution, or unrestricted automatic value/world generation from ordinary source files. Local retries/flake classification, quarantine metadata, local deterministic sharding, shard JSON merge, generation, shrinking, coverage, and mutation reporting are bounded to the implemented law/test/property metadata substrates and supported primitive/container domains.
