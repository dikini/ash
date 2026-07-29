# TASK-2052: Language Reference for Entry, Engine Admission, Clients, and Terminals

**Status:** Planned
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

- [ ] Entry, admission, client locality, and terminal statuses are independent and evidenced.
- [ ] Bounded parity is not described as a general guarantee.
- [ ] Removed forms never appear as current examples.
- [ ] Indexes, changelog, and PLAN-INDEX are updated.
