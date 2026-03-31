# S57-2 CLI Exit-Immediately Policy Design

## Context

TASK-S57-2 updates SPEC-005 so the CLI contract explicitly defines process lifetime for `ash run`. The missing point is whether the OS process remains alive until all spawned descendants complete or exits once the entry workflow `main` completes.

This task adopts the architectural decision already recorded in the task: the OS process exits immediately when `main` completes. Descendant workflows are not part of CLI process-liveness semantics.

## Goals

- Define the `ash run` process lifecycle in SPEC-005.
- Tie process exit to entry-workflow (`main`) completion.
- Define the CLI exit-code source in terms of `main` completion.
- State that spawned descendants do not extend process lifetime.
- Cross-reference SPEC-004 for completion semantics and SPEC-021 for observability boundaries.

## Non-Goals

- No runtime implementation changes.
- No new detached-process or background-service feature.
- No attempt to fully define descendant cleanup semantics after process exit.
- No expansion of the full observable-exit contract beyond what SPEC-005 needs; that remains for follow-on work such as S57-3.

## Design Decisions

### 1. Exit Immediately on `main` Completion

SPEC-005 will state that `ash run <file.ash>` creates one OS process, resolves and executes the entry workflow `main`, and exits immediately when `main` completes.

The process lifetime is therefore tied to the entry workflow only, not to the total runtime graph.

### 2. Descendants Are Outside Process-Liveness Semantics

Spawned descendants do not extend process lifetime. Once `main` completes, the CLI contract is satisfied and the process exits.

The fate of descendants after that point is intentionally left outside the CLI contract and described as implementation-defined. This avoids over-specifying runtime behavior in a CLI document.

### 3. Abstract Exit-Code Mapping

This task uses an intentionally small exit-code contract:

- exit code `0`: `main` completes successfully with its required obligations discharged;
- exit code `N`: `main` completes with a runtime error carrying exit code `N`;
- exit code `1`: bootstrap, loading, or verification failure that prevents successful entry execution.

This is enough for S57-2 without prematurely freezing a larger exit-code taxonomy.

### 4. `ash run` Syntax Tightening

SPEC-005 will update the `ash run` syntax to:

```text
ash run [options] <file.ash> [-- <args>...]
```

It will clarify that:

- `<file.ash>` is the entry source file;
- `--` separates CLI flags from program arguments;
- trailing arguments are passed to the program through the `Args` capability;
- bare `ash file.ash` is not part of the minimal-core CLI contract.

### 5. Cross-Spec Alignment

- SPEC-004 remains the source of workflow-completion semantics for `main`.
- SPEC-021 remains the source of what failures, outputs, and CLI-visible execution outcomes are observable.

SPEC-005 will reference those specs rather than restating their full behavior.

## Planned Edits

1. Update the `ash run` synopsis in SPEC-005.
2. Add a process-exit-policy subsection under `ash run`.
3. Add `ash run`-specific exit-code bullets tied to `main` completion.
4. Add cross-references to SPEC-004 and SPEC-021.
5. Update the S57-2 task file, plan index, and changelog after the spec text lands.

## Risks and Mitigations

- **Risk:** SPEC-005 overreaches into runtime descendant behavior.
  - **Mitigation:** keep descendant fate explicitly implementation-defined and outside the CLI contract.
- **Risk:** conflict with future observable-exit work in S57-3.
  - **Mitigation:** keep this task scoped to process lifetime and abstract exit-code sourcing only.
- **Risk:** ambiguity about successful completion requirements.
  - **Mitigation:** tie successful exit to successful `main` completion with obligations discharged.

## Validation

The task is complete when:

- SPEC-005 explicitly states exit-immediately policy for `ash run`.
- SPEC-005 defines exit-code derivation from `main`.
- SPEC-005 states that descendants do not extend process lifetime.
- SPEC-005 updates command syntax to include `-- <args>...`.
- SPEC-005 cross-references SPEC-004 and SPEC-021.
- task/changelog metadata reflect the completed work.
