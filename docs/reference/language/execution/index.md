# Entry, Admission, Clients, and Terminal Results

[Language reference](../index.md) · [Status and coverage](../status.md) ·
[Source of truth](../source-of-truth.md)

## Page status

**Reviewed revision:** `423f603c`.

**Implementation:** partial. The current Engine admits a small, sealed set of checked source
artifacts and runs those artifacts only through its checked-CPS dispatcher. It does not make every
parsed or checked `fn main` executable.
**Evidence:** tested. The entry, Engine, CLI, REPL, and daemon tests named below exercise the
admitted positive controls and their closed counterparts.
**Parity:** below_spec. The selected client comparison establishes normalized terminal agreement
only for its declared source descriptor and control envelope.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `fn main` source candidate | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Canonical zero-parameter entry metadata control | accepted | checked | bounded-only | fixture-bounded | partial | tested | below_spec |
| Engine-issued admitted program | not-applicable | checked | bounded-only | admitted-executed | partial | tested | below_spec |
| `Engine::execute` and `execute_with_input` | not-applicable | not-applicable | not-applicable | closed | partial | tested | below_spec |
| CLI, test runner, REPL, and daemon selected routes | not-applicable | not-applicable | not-applicable | fixture-bounded | partial | tested | below_spec |
| V1 terminal envelope | not-applicable | not-applicable | not-applicable | admitted-executed | partial | tested | below_spec |

`not-applicable` means that an item is an Engine or client API rather than source syntax. It does
not mean that a client may select another evaluator.

## In this chapter

- [Entry candidates, checked lowering, and admission](entry-lowering-and-admission.md) — the
  parser-visible `fn main` shape, the separate canonical-entry contract, and the sealed
  Engine-issued request route.
- [Clients and terminal results](clients-terminals-and-diagnostics.md) — the client-local Engine
  boundary, the deliberately narrow parity witness, and the six normalized V1 terminal outcomes.

## Boundary with forms, types, and effects

The [forms chapter](../forms/declarations-and-functions.md#functions-and-bounded-entry-execution)
documents ordinary function syntax and the `42` source fixture. The [types chapter](../types/data-newtypes-and-callables.md#capability-name-is-a-source-type-not-a-declaration)
documents `capability Name` as a type spelling, not an authority declaration. The
[effects chapter](../effects/index.md#scope-boundary) documents that computation rows, resources,
roles, and handler facts carry requirements or metadata but do not mint an admitted program,
provider binding, or handler frame.

Accordingly, this chapter is the only manual chapter that describes how a checked source artifact
becomes an executable request. A declaration, type, row, imported module, or client flag cannot
skip that boundary.

## Current-example boundary

No removed workflow/tower entry syntax is a current route. The pages below use only `fn main`
source candidates and label whether an example is parser/checker evidence, canonical-entry
verification evidence, or an exact admitted execution fixture.

## Related evidence

- [AUDIT-206 LANG-016](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2052](../../../plan/tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md)
- `crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`
- `crates/ash-engine/tests/entry_verification.rs`
- `crates/ash-cli/tests/task_2042_daemon_admitted_request_terminal_envelope_parity.rs`
