# S57-3 Observable Exit Behavior Design

## Context

TASK-S57-3 updates SPEC-021 so Ash has a normative observable-exit contract for `ash run`. SPEC-005 now defines the CLI process-lifetime policy, and SPEC-004 now defines runtime-internal completion observation through `ControlLink`. The remaining gap is the external boundary: what an observer can rely on when the entry workflow `main` completes and the process exits.

## Goals

- Define the observable process-exit event for `ash run`.
- Tie that observable event to entry-workflow (`main`) completion.
- Define the observable exit-code source using the same abstract mapping already aligned in SPEC-005.
- State that spawned descendants are not externally observable after process exit.
- Keep control-authority completion observation runtime-internal while cross-referencing SPEC-004 and SPEC-005.
- Add testable assertions that unblock exit-code and integration-test tasks.

## Non-Goals

- No runtime or CLI implementation changes.
- No new user-visible `await`, `join`, or descendant-management syntax.
- No signal-handling contract for interrupts or OS termination signals.
- No full descendant teardown policy; descendant fate remains implementation-defined.
- No expansion of `CompletionPayload` into a user-visible typed value.

## Design Decisions

### 1. Observable Exit Boundary Is `main` Completion

SPEC-021 will define the externally observable event as OS process termination with an exit code. That event is triggered by the completion of the entry workflow `main`, not by descendant completion and not by runtime-internal supervisor bookkeeping.

### 2. Exit Code Is Sourced Only From the Entry Workflow Outcome

The observable exit-code contract will match the abstract entry-point mapping already used elsewhere:

- `0` when `main` completes successfully and required obligations are discharged;
- `N` when `main` completes with a runtime error carrying exit code `N`;
- `1` when bootstrap, loading, verification, or other pre-entry failures prevent successful execution of `main`.

SPEC-021 will frame this as an observable guarantee, while deferring the internal completion mechanics to SPEC-004 and the CLI command surface to SPEC-005.

### 3. Descendant Fate Is Explicitly Non-Observable

SPEC-021 will say that spawned descendants do not contribute additional externally testable lifecycle guarantees after process exit. Implementations may terminate, orphan, continue, or otherwise manage descendants after `main` completes, but that behavior is outside the observable contract.

### 4. Control-Authority Completion Observation Remains Internal

The `ControlLink`/completion-payload contract from SPEC-004 will be referenced only to clarify boundaries: a supervisor may observe child completion internally, but that observation is not itself a user-visible event. SPEC-021 should therefore tighten the value-display/control-authority text so readers do not mistake runtime-internal completion observation for a surface observable.

### 5. Testability Should Be Normative Enough to Unblock Future Tests

SPEC-021 will include concrete, testable assertions that future harnesses can verify, including:

- parent `main` success determines exit code even if a spawned child later fails;
- bootstrap or verification failure exits before any successful `main` completion;
- post-exit descendant behavior is intentionally not asserted by conformance tests.

## Planned Edits

1. Add a new subsection under Section 2 for process-exit observables.
2. Update the CLI-tooling observability bullets to mention `ash run` process termination and exit-code visibility.
3. Tighten Section 4.2 so control-authority completion observation is described as runtime-internal rather than directly user-observable.
4. Add testable assertions and cross-references to SPEC-004 and SPEC-005.
5. Update the S57-3 task file, PLAN-INDEX, and CHANGELOG after the spec edit lands.

## Risks and Mitigations

- **Risk:** SPEC-021 duplicates too much CLI text from SPEC-005.
  - **Mitigation:** keep SPEC-021 focused on observability and reference SPEC-005 for command contract details.
- **Risk:** readers infer descendant cleanup guarantees.
  - **Mitigation:** state explicitly that descendant fate is implementation-defined and non-observable.
- **Risk:** control-link completion semantics look user-visible.
  - **Mitigation:** add explicit runtime-internal boundary language and cross-reference SPEC-004.

## Validation

The task is complete when:

- SPEC-021 defines the observable process-exit event and trigger.
- SPEC-021 defines exit-code sourcing from `main`.
- SPEC-021 states descendant fate is non-observable and implementation-defined.
- SPEC-021 includes testable assertions for future harnesses.
- SPEC-021 cross-references SPEC-004 and SPEC-005.
- task/planning/changelog metadata reflect the completed work.
