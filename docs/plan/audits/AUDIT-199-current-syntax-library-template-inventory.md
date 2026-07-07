# AUDIT-199: Current-Syntax Library/Template Inventory

**Status:** Complete for TASK-1943 audit/remediation gate
**Task:** [TASK-1943](../tasks/TASK-1943-current-syntax-library-template-audit-remediation.md)

## Scope

This audit classifies every `.ash` file under `std/src`, `examples`, `tests/std`, and
`tests/workflows` before Phase 199 accepts new productive app templates. Classifications are:

- `current executable`: current target syntax accepted by `ash check` through an existing corpus or
  focused CLI gate.
- `current reference`: current library/reference surface that is intentionally not standalone
  executable through the generic corpus checker yet, but is covered by an artifact or focused test.
- `historical/reference-only`: excluded from productive template and tutorial paths until rewritten.
- `removed from productive path`: no current entries in this audit.

## Documentation Asset Notes

Productive docs that describe current examples remain tied to the corpus gates:
`examples/README.md`, `examples/06-capability-implementations/README.md`,
`examples/07-phase105/README.md`, `examples/09-phase108/README.md`, and `std/src/llm/README.md`.
Historical docs such as `examples/03-policies/README.md`, `examples/04-real-world/README.md`, and
`examples/workflows/40_tdd_README.md` are reference-only and must not feed Phase 199 templates until
they are rewritten and gated.

## Inventory

| Path | Classification | Gate or reason |
|------|----------------|----------------|
| `examples/01-basics/01-hello-world.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/01-basics/02-variables.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/01-basics/03-expressions.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/01-basics/04-observe.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/02-control-flow/01-conditionals.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/02-control-flow/02-foreach.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/02-control-flow/03-sequential.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/02-control-flow/04-sequential.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/03-io/directory_listing.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/03-io/file_read_write.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/03-io/path_operations.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/03-policies/01-role-based.ash` | historical/reference-only | historical policy sketch uses stale role and workflow-era forms and is excluded from productive templates |
| `examples/03-policies/02-time-based.ash` | historical/reference-only | historical policy sketch uses stale temporal policy and workflow-era forms and is excluded from productive templates |
| `examples/04-real-world/code-review.ash` | historical/reference-only | older real-world sketch uses historical workflow and policy syntax and is excluded from productive templates |
| `examples/04-real-world/customer-support.ash` | historical/reference-only | older real-world sketch uses historical workflow and policy syntax and is excluded from productive templates |
| `examples/05-phase98/01-fail-with-error.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and run/trace via `task_717_phase98_examples_conformance` |
| `examples/05-phase98/02-proc-par-await-join.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and run/trace via `task_717_phase98_examples_conformance` |
| `examples/05-phase98/03-proc-scatter-gather.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and run/trace via `task_717_phase98_examples_conformance` |
| `examples/05-phase98/04-workflow-boundary-reporting.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and run via `task_717_phase98_examples_conformance` |
| `examples/06-capability-implementations/01-mock-internal-kv.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and `task_742_capability_examples_conformance` |
| `examples/06-capability-implementations/02-caching-kv-adapter.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and `task_742_capability_examples_conformance` |
| `examples/06-capability-implementations/03-recording-replay-sketch.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and `task_742_capability_examples_conformance` |
| `examples/07-phase105/01-do-act.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/07-phase105/02-act-sugar.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/07-phase105/03-do-proc-from-act.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/08-phase106/01-act-comprehension.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/08-phase106/02-proc-comprehension-from-act.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/08-phase106/03-deferred-pure-targets.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/09-phase108/01-do-workflow-unit.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/09-phase108/02-do-workflow-contract-statements.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/09-phase108/03-workflow-algebra-intrinsics.reference.ash` | historical/reference-only | reference-only workflow algebra intrinsic spelling until full source-file elaboration is available |
| `examples/09-phase108/04-workflow-explicit-lifts.reference.ash` | historical/reference-only | reference-only explicit lower-tower lift spelling until full source-file elaboration is available |
| `examples/09-phase108/05-workflow-comprehension.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/09-phase108/06-legacy-workflow-migration-warning.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/10-testing-helpers/testing_helpers.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and `phase199_testing_helpers` |
| `examples/11-process-channel-helpers/process_channel_helpers.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and `phase199_process_channel_helpers` |
| `examples/code_review.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/entrypoint_args.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` and `stdlib_corpus_check` repair target |
| `examples/entrypoint_minimal.ash` | current executable | ash check via `cargo test -p ash-cli --test example_corpus_check` |
| `examples/multi_agent_research.ash` | historical/reference-only | older workflow sketch uses stale `observe ... with` syntax and is excluded from productive templates |
| `examples/simple_workflow.ash` | historical/reference-only | older workflow sketch uses stale `observe ... with` syntax and is excluded from productive templates |
| `examples/support_ticket.ash` | historical/reference-only | older workflow sketch uses stale `observe ... with` syntax and is excluded from productive templates |
| `examples/workflows/40_tdd_workflow.ash` | historical/reference-only | older TDD workflow sketch uses stale observe and act forms and is excluded from productive templates |
| `examples/workflows/40a_tdd_concrete_example.ash` | historical/reference-only | older concrete TDD workflow sketch uses stale observe and act forms and is excluded from productive templates |
| `std/src/act.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/applicative.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/comonad.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/eq.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/functor.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/kleisli.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/mod.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/monad.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/monoid.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/algebra/semigroup.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/evidence.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/http.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/buf.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/dir.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/fs.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/meta.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/mod.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/path.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/io/stdio.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/json.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/lib.ash` | current reference | artifact assertion: classified expected-fail in `cargo test -p ash-cli --test stdlib_corpus_check` because generic checker does not yet parse multi-line pub-use imports |
| `std/src/list.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/conversation.ash` | current reference | artifact assertion: classified expected-fail in `cargo test -p ash-cli --test stdlib_corpus_check` until workflow export visibility resolves `dispatch::complete` |
| `std/src/llm/dispatch.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/loading.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/mod.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/openai.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/prompt.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/router.ash` | current reference | artifact assertion: classified expected-fail in `cargo test -p ash-cli --test stdlib_corpus_check` until workflow export visibility resolves `dispatch::complete` |
| `std/src/llm/supervised.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/llm/tool_agent.ash` | current reference | artifact assertion: classified expected-fail in `cargo test -p ash-cli --test stdlib_corpus_check` until workflow export visibility resolves `dispatch::complete_with_tools` |
| `std/src/llm/types.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/logging.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/map.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/markdown.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/ooda.ash` | current reference | artifact assertion: classified expected-fail in `cargo test -p ash-cli --test stdlib_corpus_check` as library and template compatibility source rather than executable stdlib corpus file |
| `std/src/option.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/predicate.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/prelude.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/proc.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/process.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/record.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/regex.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/result.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/runtime/args.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/runtime/error.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/runtime/mod.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/runtime/supervisor.ash` | current reference | artifact assertion: classified expected-fail in `cargo test -p ash-cli --test stdlib_corpus_check` because relative supervisor imports are not resolved by the generic checker |
| `std/src/string.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/artifact.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` and imported by `phase199_testing_helpers` |
| `std/src/test/fixtures.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` and imported by `phase199_testing_helpers` |
| `std/src/test/quickcheck/arbitrary.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/bool.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/combinator.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/context.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/int.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/list.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/mod.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/prelude.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/strategy.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/test/quickcheck/string.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/time.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `std/src/workflow.ash` | current executable | ash check via `cargo test -p ash-cli --test stdlib_corpus_check` |
| `tests/std/io_buf.ash` | historical/reference-only | legacy std fixture directory is excluded from productive template paths and superseded by stdlib corpus and focused engine tests |
| `tests/std/io_dir.ash` | historical/reference-only | legacy std fixture directory is excluded from productive template paths and superseded by stdlib corpus and focused engine tests |
| `tests/std/io_fs.ash` | historical/reference-only | legacy std fixture directory is excluded from productive template paths and superseded by stdlib corpus and focused engine tests |
| `tests/std/io_meta.ash` | historical/reference-only | legacy std fixture uses stale `observe ... with` syntax and is excluded from productive templates |
| `tests/std/io_path.ash` | historical/reference-only | legacy std fixture uses stale `observe ... with` syntax and is excluded from productive templates |
| `tests/std/io_stdio.ash` | historical/reference-only | legacy std fixture directory is excluded from productive template paths and superseded by stdlib corpus and focused engine tests |
| `tests/std/option.ash` | historical/reference-only | legacy std fixture uses stale `observe ... with` syntax and is excluded from productive templates |
| `tests/std/result.ash` | historical/reference-only | legacy std fixture uses stale `observe ... with` syntax and is excluded from productive templates |
| `tests/workflows/code_review.ash` | historical/reference-only | legacy workflow fixture uses historical workflow-era forms and is excluded from productive templates |
| `tests/workflows/multi_agent_research.ash` | historical/reference-only | legacy workflow fixture uses stale `observe ... with` syntax and is excluded from productive templates |
| `tests/workflows/support_ticket.ash` | historical/reference-only | legacy workflow fixture uses stale `observe ... with` syntax and is excluded from productive templates |

## Findings

- Existing `example_corpus_check` and `stdlib_corpus_check` already provide the primary executable
  gates for current productive examples and stdlib modules.
- Productive Phase 199 template inputs must draw only from `current executable` or explicitly
  `current reference` rows above. `historical/reference-only` rows are excluded until rewritten and
  promoted through an executable or artifact gate.
- The only required syntax remediation in this slice was the productive `std/README.md` usage
  snippet, which no longer advertises stale `act ... with` spelling.
