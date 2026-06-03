# Canonical IR Semantics Corpus

## Status

TASK-438 reference corpus definition.

## Purpose

This reference freezes the canonical semantics corpus that future differential-conformance and
reference-implementation work must target.

It exists so Rust, Lean, and future Ash implementations are compared against one shared corpus of
canonical IR cases instead of assembling ad hoc examples from surface syntax, runtime tests, or
phase-local notes.

This document defines the corpus, not the harness.

## 1. Authority and Role

This corpus must be read under the authority split already frozen elsewhere:

- [SPEC-001: Intermediate Representation](../spec/SPEC-001-IR.md) owns the canonical workflow and
  expression forms used as corpus inputs.
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md) owns terminal big-step meaning.
- [SPEC-025: Small-Step Operational Semantics](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
  owns the workflow-first state-taxonomy and helper-boundary story.
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) owns the
  surfaced runtime-observable layer.
- [SPEC-026: Implementation Conformance Contract](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md)
  owns the conformance surfaces and comparison rules.
- [Semantic Execution Record Contract](semantic-execution-record-contract.md) and
  [Retained Completion Parity Contract](retained-completion-parity-contract.md) own the runtime-side
  projection boundaries the corpus may reference.

Normatively:

1. corpus inputs are canonical IR first;
2. every case must declare the conformance surface(s) it targets;
3. every case must name the authoritative documents that own its expected meaning;
4. every case must say whether it is deterministic or set-valued under bounded nondeterminism;
5. the corpus must not smuggle in extra requirements from current Rust implementation details.

## 2. Corpus Scope

The canonical v1 corpus covers the minimum semantic families required by TASK-438:

1. sequencing / binding / branching;
2. pattern-driven control;
3. capability / policy / obligation workflows;
4. receive / blocking / fallback behavior;
5. spawn / control / completion observation;
6. representative failure paths.

This corpus is canonical-IR-first rather than surface-syntax-first. Surface syntax may be used to
explain a case informally, but the authoritative executable input for the corpus is the canonical IR
encoding of that workflow.

## 3. File-Backed Corpus Layout

Future harness work should treat the following repo-relative layout as the canonical corpus layout:

```text
tests/differential/corpus/<case-id>/
  case.json
  input.ir.json
  expected.json
  setup.json        # optional
  notes.md          # optional, informative only
```

### 3.1 Required files

- `case.json` — machine-readable case manifest
- `input.ir.json` — canonical IR input for the case
- `expected.json` — expected-result artifact using
  [canonical-semantics-result-format.md](canonical-semantics-result-format.md)

### 3.2 Optional files

- `setup.json` — machine-readable pre-state/setup requirements when the case needs mailbox contents,
  child workflow registration, provider availability, or other externalized harness setup
- `notes.md` — informative human notes only; never the authoritative expected result

## 4. Case Manifest Contract

Each `case.json` file must have at least:

```json
{
  "schema_version": "ash-corpus-case/v1",
  "case_id": "seq-bind-return",
  "title": "Sequencing binds then returns the bound value",
  "surfaces": ["big-step"],
  "authorities": [
    "SPEC-001",
    "SPEC-004",
    "SPEC-026"
  ],
  "determinism": {
    "kind": "deterministic"
  },
  "tags": ["seq", "binding", "deterministic"],
  "input_file": "input.ir.json",
  "expected_file": "expected.json",
  "setup_file": null
}
```

Rules:

1. `surfaces` must be drawn from the SPEC-026 conformance surfaces:
   - `big-step`
   - `small-step`
   - `runtime-observable`
2. `authorities` must cite the owning documents for the case.
3. `determinism.kind` must be either:
   - `deterministic`
   - `allowed_set`
4. if `setup_file` is omitted or `null`, the case must be runnable from the harness default empty
   setup for its declared surface.

## 5. Canonical Input Rules

### 5.1 Canonical IR only

`input.ir.json` must encode canonical workflow/input terms corresponding to [SPEC-001](../spec/SPEC-001-IR.md)
(and [SPEC-020](../spec/SPEC-020-ADT-TYPES.md) where constructor/pattern values matter).

The corpus must not use surface-only sugar as its authoritative input.

### 5.2 Stable normalization

The canonical input artifact must be normalized so independent implementations can consume the same
input file without re-parsing informal prose.

At minimum the normalization policy is:

1. canonical workflow form names, not parser-only aliases;
2. canonical constructor and pattern names, not display-specific renderings;
3. deterministic key ordering for JSON objects when fixtures are generated or refreshed;
4. no implementation-private enum names, Rust debug dumps, or Lean pretty-printed terms.

## 6. Setup Profile Rules

Some corpus cases need setup outside the workflow term itself.

Examples:

- receive/mailbox cases need preloaded messages or timeout configuration;
- spawn/completion cases need registered child workflow definitions;
- capability or policy cases may need provider availability or denied capability context;
- runtime-observable control cases may need child/workflow registration plus explicit control-link
  observation steps.

Normatively:

1. any required setup must be externalized in `setup.json`, not hidden in harness code;
2. setup files must describe only the preconditions needed to run the canonical case;
3. setup files must not silently encode expected outcomes that belong in `expected.json`.

## 7. Minimum v1 Corpus Catalog

The table below freezes the minimum canonical v1 case families and initial case IDs.

| Case ID | Primary surface(s) | Determinism | Canonical focus |
|---|---|---|---|
| `seq-bind-return` | big-step | deterministic | sequencing and binding preserve canonical terminal return |
| `if-branch-selects-else` | big-step | deterministic | branching selects the correct continuation |
| `match-variant-binds-payload` | big-step | deterministic | pattern-driven control with constructor payload binding |
| `guard-fallthrough-selects-next-arm` | big-step | deterministic | pattern/guard control fallthrough remains semantically owned |
| `oblige-then-check-discharges` | big-step | deterministic | obligation creation/check success path |
| `obligation-role-mismatch-rejects` | big-step | deterministic | obligation failure stays a semantic rejection |
| `capability-denied-rejects` | runtime-observable | deterministic | denied capability remains a visible non-success boundary |
| `policy-requires-approval-classifies-distinctly` | runtime-observable | deterministic | approval-required outcome stays distinct on observable surface |
| `receive-empty-blocks` | small-step | allowed_set | blocked receive remains blocked/nonterminal, not stuck/reject |
| `receive-timeout-fallback` | big-step, small-step | allowed_set | timeout/fallback behavior stays within admitted helper-owned set |
| `receive-wildcard-fallback` | big-step | deterministic | wildcard receive fallback continuation |
| `spawn-child-success-retained-completion` | big-step, runtime-observable | deterministic | spawned child success with retained completion projection |
| `spawn-child-rejection-retained-completion` | big-step, runtime-observable | deterministic | spawned child rejection with retained completion projection |
| `spawn-control-kill-tombstone` | runtime-observable | deterministic | control tombstone remains distinct from child-owned completion payload |
| `completion-observation-wait` | small-step, runtime-observable | allowed_set | completion observation wait remains blocked/helper-owned |
| `runtime-boundary-missing-provider` | runtime-observable | deterministic | representative runtime failure path |
| `pattern-failure-owned-rejection` | big-step | deterministic | unchecked-IR defensive pattern/control failure keeps the correct owning rejection boundary |

Checked source examples that contain refutable binder patterns or non-exhaustive total eliminators
belong in source/type-checking conformance suites, not in this canonical IR-first corpus. The IR
case above exists to preserve the runtime defensive boundary for host-created or unchecked IR terms;
it must not be used to justify source-level `let` pattern failures reaching runtime.

This is the minimum corpus, not the maximum corpus. Later tasks may add more cases without changing
this document's authority split.

## 8. Bounded Nondeterminism Policy for Corpus Cases

The corpus must make bounded nondeterminism explicit.

### 8.1 Deterministic cases

A deterministic case has exactly one allowed result projection.

### 8.2 Allowed-set cases

An `allowed_set` case has a finite set of admitted result projections recorded in `expected.json`.

This is the required representation for v1 corpus nondeterminism.

The corpus must not use vague prose such as "implementation-dependent" where SPEC-026 already gives
an admitted bounded set.

### 8.3 `receive` policy

For `receive` cases:

1. the corpus must compare admitted selection/fallback/timeout/blocked outcome classes;
2. it must not compare raw queue-probe counts, polling frequency, or one mailbox algorithm;
3. if more than one admitted continuation/result exists, the allowed set must list them explicitly.

## 9. Surface-Specific Expectations

The corpus must declare the surface being tested and keep projections surface-appropriate.

### 9.1 Big-step cases

Big-step cases compare terminal semantic outcomes only.

They must not require internal step counts or scheduler order unless a canonical spec explicitly owns
that ordering.

### 9.2 Small-step cases

Small-step cases compare admitted state-taxonomy behavior.

They may assert:

- running vs blocked vs terminal vs invalid class,
- blocked-family ownership,
- admitted terminal reconstruction,
- bounded nondeterministic aggregate classes.

They must not overassert on hidden internal silent-step segmentation.

### 9.3 Runtime-observable cases

Runtime-observable cases compare surfaced user/tool-visible behavior only.

They may assert:

- exit/result class,
- visible value/error distinctions,
- visible control/completion/tombstone distinctions,
- visible retained-completion observations where applicable.

They must not smuggle in hidden runtime-private state as if it were observable truth.

## 10. Relationship to the Canonical Result Format

Each corpus case's `expected.json` must use
[Canonical Semantics Result Format](canonical-semantics-result-format.md).

That companion document owns:

- the machine-readable envelope,
- exact-vs-allowed-set representation,
- projection field names,
- omission/out-of-scope rules,
- normalized payload comparison rules.

This corpus document instead owns:

- which cases exist,
- which surfaces they target,
- which authorities they cite,
- and how the file-backed corpus is organized.

## 11. Follow-On Boundary

This document is the direct input for:

- [TASK-439: Differential Conformance Harness (Rust First)](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md)
- [TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus](../plan/tasks/TASK-440-lean-reference-refresh-plan-against-current-semantic-corpus.md)

Normatively:

1. TASK-439 must consume this corpus layout and case-selection policy rather than inventing one-off
   harness fixtures;
2. TASK-440 must treat this corpus as the canonical comparison-target set for future Lean/reference
   integration;
3. later tasks may extend the corpus, but they must not contradict the authority split or surface
   rules frozen here.
