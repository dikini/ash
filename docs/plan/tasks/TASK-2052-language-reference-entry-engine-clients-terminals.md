# TASK-2052: Language Reference for Entry, Engine Admission, Clients, and Terminals

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-016.

## Description

Document the bounded `fn main` source-entry route, checked lowering/admission, Engine-only
execution fragments, CLI/REPL/test/daemon client boundaries, and normalized terminal behaviour.

## Requirements

- Create `docs/reference/language/execution/index.md`, `entry-lowering-and-admission.md`, and
  `clients-terminals-and-diagnostics.md`.
- Explain that generic Engine execution APIs fail closed outside their admitted source/checked CPS
  boundaries and that no direct-evaluator fallback is selected.
- Distinguish one selected client/terminal parity witness from full client or target-spec parity.
- Link authority and row requirements to TASK-2050/2051 pages without treating rows as admission.

## Handoffs and dependencies

- **Consumes:** `ash-engine/src/lib.rs`, private CPS/admission paths, CLI run/test/daemon, and
  REPL route implementations.
- **Evidence:** `cargo test -p ash-engine --test task_1865_surface_fn_main_entry`, `--test
  entry_verification`; `cargo test -p ash-cli --test task_2008_runtime_terminal_envelope`, `--test
  task_2042_daemon_admitted_request_terminal_envelope_parity`; task records 2032/2038/2039/2042.
- **Produces:** runtime status and diagnostic links for TASK-2053/2054.
- **Non-goals:** workflow source declarations, a claim all functions are executable, a shared
  daemon evaluator, or general four-client/target parity.

## TDD and verification steps

1. Write an entry-route matrix that names source, checked Core/CPS, admission, client, and
   terminal evidence before writing examples.
2. Run positive terminal and fail-closed admission tests; label unsupported samples accordingly.
3. Render only transitions witnessed by implementation and validate links/fences.

## Completion checklist

- [x] Entry, admission, client locality, and terminal statuses are independent and evidenced.
- [x] Bounded parity is not described as a general guarantee.
- [x] Removed forms never appear as current examples.
- [x] Indexes, changelog, and PLAN-INDEX are updated.

## Completion evidence

**Semantic task classification:** non-semantic-workflow-enforcement

- Entry route evidence: `task_1865_surface_fn_main_entry` proves the exact `fn main() -> Int {
  do { return 42; } }` positive control and its rich checked-but-unadmitted counterpart;
  `entry_verification` proves the distinct canonical `Result<(), RuntimeError>` and
  `capability Args` verification contract.
- Closed-boundary evidence: `Engine::execute` and `Engine::execute_with_input` return the
  checked-Core/CPS closed-admission error. `run` and `run_file` instead use the Engine-issued
  admitted-program request and the shared dispatcher.
- Client/terminal evidence: TASK-2032 covers the in-process opaque-request seam; TASK-2038 and
  TASK-2039 cover their declared `ash test` and REPL catalogues; TASK-2042 compares the exact
  `fn main() -> Int { 42 }` descriptor with `ash run` over a Unix socket. The latter is a
  selected parity witness, not a general client theorem.
- Documentation evidence: the execution index and both child pages link from the manual status
  map; the EBNF fence, orientation index self-test, documentation gate, and diff check pass.
