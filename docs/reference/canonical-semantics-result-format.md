# Canonical Semantics Result Format

## Status

TASK-438 result-format definition.

## Purpose

This reference freezes one machine-readable expected-result format for Ash differential conformance
work.

It exists so implementations are compared against one explicit result schema aligned with
[SPEC-026: Implementation Conformance Contract](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md)
instead of multiple ad hoc golden-file shapes.

This format is implementation-neutral. It defines comparison artifacts, not one required runtime API.

## 1. Authority and Role

This result format is downstream from:

- [Canonical IR Semantics Corpus](canonical-ir-semantics-corpus.md), which defines the corpus cases
  and file layout;
- [SPEC-026](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md), which defines the conformance surfaces
  and comparison rules;
- [SPEC-004](../spec/SPEC-004-SEMANTICS.md), [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md), and
  [SPEC-021](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), which own the meaning of the projected
  semantic/observable fields;
- [Semantic Execution Record Contract](semantic-execution-record-contract.md) and
  [Retained Completion Parity Contract](retained-completion-parity-contract.md), which define exact,
  conservative, and subset-only projection boundaries for runtime-facing carriers.

Normatively:

1. each expected-result artifact must declare the surface it compares;
2. each artifact must declare whether it is deterministic or an allowed set;
3. each artifact must encode only the fields relevant to that surface/case;
4. omitted fields are out of scope for that case unless explicitly declared otherwise;
5. implementation-private incidental data must not become failure conditions.

## 2. Top-Level Envelope

Each `expected.json` file must use the following top-level shape:

```json
{
  "schema_version": "ash-expected-result/v1",
  "case_id": "seq-bind-return",
  "surface": "big-step",
  "authorities": ["SPEC-001", "SPEC-004", "SPEC-026"],
  "expectation": {
    "kind": "exact",
    "result": { }
  }
}
```

Required fields:

- `schema_version`
- `case_id`
- `surface`
- `authorities`
- `expectation`

### 2.1 Surface enumeration

`surface` must be exactly one of:

- `big-step`
- `small-step`
- `runtime-observable`

Multi-surface corpus cases should use one expected-result artifact per surface rather than one merged
artifact that blurs the comparison object.

## 3. Deterministic vs Allowed-Set Expectations

### 3.1 Exact deterministic expectation

```json
{
  "expectation": {
    "kind": "exact",
    "result": { ... }
  }
}
```

### 3.2 Allowed-set expectation

```json
{
  "expectation": {
    "kind": "allowed_set",
    "results": [
      { ... },
      { ... }
    ]
  }
}
```

Rules:

1. `exact` means exactly one admitted result projection;
2. `allowed_set` means any one listed result is conformant;
3. each member of an `allowed_set` must already be normalized into the same result shape;
4. a harness must fail only if the actual normalized result matches none of the listed members.

## 4. Canonical Result Object

Each result object may contain the following top-level fields, depending on surface and case:

```json
{
  "outcome_class": "return",
  "payload": { ... },
  "effects": { ... },
  "obligations": { ... },
  "provenance": { ... },
  "trace": { ... },
  "blocked": { ... },
  "retained_completion": { ... },
  "control_observation": { ... },
  "notes": { ... }
}
```

This is a sparse schema. Fields may be omitted when they are out of scope for the selected surface
or case.

## 5. Required Normalized Fields by Surface

### 5.1 Big-step result shape

Big-step result objects must include:

- `outcome_class`

They may include, as applicable:

- `payload`
- `effects`
- `obligations`
- `provenance`
- `trace`
- `retained_completion`

#### 5.1.1 Outcome-class enumeration

For big-step cases, `outcome_class` must be one of:

- `return`
- `reject`

#### 5.1.2 Payload shape

```json
{
  "payload": {
    "kind": "value",
    "value": { ...canonical-value... }
  }
}
```

or

```json
{
  "payload": {
    "kind": "error",
    "error": { ...normalized-error... }
  }
}
```

### 5.2 Small-step result shape

Small-step result objects must include:

- `outcome_class`

For small-step cases, `outcome_class` must be one of:

- `running`
- `blocked`
- `terminal_return`
- `terminal_reject`
- `invalid`

They may additionally include:

- `blocked`
- `payload`
- `effects`
- `obligations`
- `provenance`
- `trace`

### 5.3 Runtime-observable result shape

Runtime-observable result objects must include:

- `outcome_class`

For runtime-observable cases, `outcome_class` must be one of the visible classes relevant to the
case, such as:

- `success`
- `warning`
- `denied`
- `error`
- `blocked`
- `completion_observed`
- `control_tombstone`

The exact visible classes are case-dependent but must remain aligned with the observable boundary the
cited authority documents actually own.

## 6. Field Schemas

### 6.1 Effects field

```json
{
  "effects": {
    "classification": "exact",
    "terminal": "Operational",
    "reached": ["Epistemic", "Operational"]
  }
}
```

Rules:

1. `classification` must be one of:
   - `exact`
   - `conservative`
2. if `classification` is `exact`, the harness compares exact normalized terminal and reached sets;
3. if `classification` is `conservative`, the harness must compare according to the weaker contract
   declared by the owning reference, not by pretending the field is exact;
4. for TASK-438 v1 corpus work, canonical expected results should prefer exact projections whenever
   the chosen surface owns one exact projection.

### 6.2 Obligations field

```json
{
  "obligations": {
    "classification": "exact",
    "summary": { ... }
  }
}
```

or

```json
{
  "obligations": {
    "classification": "subset_only",
    "summary": { ... }
  }
}
```

Allowed `classification` values:

- `exact`
- `subset_only`

### 6.3 Provenance field

```json
{
  "provenance": {
    "classification": "exact",
    "summary": { ... }
  }
}
```

or

```json
{
  "provenance": {
    "classification": "conservative",
    "summary": { ... }
  }
}
```

Allowed `classification` values:

- `exact`
- `conservative`

### 6.4 Trace field

```json
{
  "trace": {
    "policy": "exact_payload",
    "events": [ ... ]
  }
}
```

or

```json
{
  "trace": {
    "policy": "out_of_scope"
  }
}
```

Allowed `policy` values:

- `exact_payload`
- `summary_only`
- `out_of_scope`

For TASK-438 v1:

- big-step cases may use `exact_payload` where trace is semantically owned and expected;
- retained-completion-oriented comparisons may use `out_of_scope`, since retained parity does not own
  `T`.

### 6.5 Blocked field

```json
{
  "blocked": {
    "family": "ReceiveWait"
  }
}
```

Allowed `family` values must align with the execution-record / small-step contracts, including:

- `ReceiveWait`
- `CompletionObservationWait`
- `ControlWait`
- `HelperWait`

If `family` is `HelperWait`, an optional `name` field may refine it.

### 6.6 Retained-completion field

```json
{
  "retained_completion": {
    "kind": "child_completion",
    "result": { ... },
    "effects": { ... },
    "obligations": { ... },
    "provenance": { ... }
  }
}
```

or

```json
{
  "retained_completion": {
    "kind": "control_tombstone"
  }
}
```

Rules:

1. `child_completion` represents child-owned terminal retained completion payloads;
2. `control_tombstone` represents terminal control observations that are explicitly not child-owned
   completion payloads;
3. comparison of `effects`, `obligations`, and `provenance` must respect the exact / conservative /
   subset-only classifications frozen by the retained-completion parity contract.

### 6.7 Control-observation field

```json
{
  "control_observation": {
    "state": "terminated"
  }
}
```

This field is for runtime-observable control cases where the visible control state matters apart from
retained child completion payload.

## 7. Value and Error Normalization

The canonical result format does not freeze one universal serialized value language beyond requiring
stable normalized JSON.

Normatively:

1. value payloads must be encoded in a stable implementation-neutral normalized JSON representation;
2. error payloads must preserve semantic category and payload meaning, not implementation-private
   display strings only;
3. implementations may keep richer private detail, but comparison uses the normalized payload only.

A v1 normalized value/error policy is:

- algebraic values encoded by constructor/tag plus normalized payload array/object;
- primitive values encoded by obvious JSON scalars where lossless;
- errors encoded by normalized semantic category plus structured payload fields where the owning
  authority fixes them.

Future harness work may freeze the exact normalized value schema in a support file as long as it
remains implementation-neutral and consistent with this document.

## 8. Omission and Out-of-Scope Rules

The result format is intentionally sparse.

Normatively:

1. omitted fields are not compared for that case;
2. omission means the field is out of scope for the declared surface/case, not "anything goes"
   globally;
3. if a field matters to the conformance claim for a case, it must be present;
4. implementations must not fail a case because they expose extra internal details not named by the
   normalized format.

## 9. Comparison Rules

A harness comparing actual vs expected results must follow these rules.

### 9.1 Exact comparison

For `expectation.kind = exact`, the actual normalized result must match the single expected result on
all present fields.

### 9.2 Allowed-set comparison

For `expectation.kind = allowed_set`, the actual normalized result is conformant if it matches any
member of the set on all present fields.

### 9.3 Classification-aware comparison

When a field declares:

- `classification = exact` — compare exact normalized contents;
- `classification = conservative` — compare only against the conservative contract that the case
  explicitly declares;
- `classification = subset_only` — compare only the declared subset, not absent hidden state.

A harness must not silently upgrade conservative or subset-only fields into exact requirements.

### 9.4 Surface discipline

A harness must compare only the projection relevant to the declared `surface`.

For example:

- runtime-observable tests do not fail because hidden trace payload differs;
- small-step blocked cases do not require terminal payloads;
- big-step terminal cases do not require raw internal step segmentation.

## 10. Example Artifacts

### 10.1 Deterministic big-step case

```json
{
  "schema_version": "ash-expected-result/v1",
  "case_id": "seq-bind-return",
  "surface": "big-step",
  "authorities": ["SPEC-001", "SPEC-004", "SPEC-026"],
  "expectation": {
    "kind": "exact",
    "result": {
      "outcome_class": "return",
      "payload": {
        "kind": "value",
        "value": { "type": "int", "value": 42 }
      },
      "effects": {
        "classification": "exact",
        "terminal": "Epistemic",
        "reached": ["Epistemic"]
      }
    }
  }
}
```

### 10.2 Allowed-set `Par` case

```json
{
  "schema_version": "ash-expected-result/v1",
  "case_id": "par-all-success-aggregates",
  "surface": "small-step",
  "authorities": ["SPEC-001", "SPEC-004", "SPEC-025", "SPEC-026"],
  "expectation": {
    "kind": "allowed_set",
    "results": [
      {
        "outcome_class": "terminal_return",
        "payload": {
          "kind": "value",
          "value": { "type": "list", "value": [1, 2] }
        }
      },
      {
        "outcome_class": "terminal_return",
        "payload": {
          "kind": "value",
          "value": { "type": "list", "value": [2, 1] }
        }
      }
    ]
  }
}
```

### 10.3 Blocked receive case

```json
{
  "schema_version": "ash-expected-result/v1",
  "case_id": "receive-empty-blocks",
  "surface": "small-step",
  "authorities": ["SPEC-025", "SPEC-026"],
  "expectation": {
    "kind": "exact",
    "result": {
      "outcome_class": "blocked",
      "blocked": {
        "family": "ReceiveWait"
      }
    }
  }
}
```

## 11. Relationship to TASK-439 and TASK-440

This format is the direct input for:

- [TASK-439: Differential Conformance Harness (Rust First)](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md)
- [TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus](../plan/tasks/TASK-440-lean-reference-refresh-plan-against-current-semantic-corpus.md)

Normatively:

1. TASK-439 must normalize Rust results into this schema before comparison;
2. TASK-439 must honor exact vs allowed-set vs classification-aware comparison instead of using
   fixture-specific ad hoc assertions;
3. TASK-440 must treat this format as the canonical result artifact future Lean/reference work should
   emit or consume when doing cross-implementation comparison.
