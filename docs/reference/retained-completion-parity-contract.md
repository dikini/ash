# Retained Completion Parity Contract

> **TASK-2041 status:** This is a reference contract. It does not authorize a legacy evaluator,
> a non-Engine CPS executor, or a client fallback route.

## Status

TASK-436 reference contract.

## Purpose

This reference freezes one explicit contract for how retained completion observation relates to the
full semantic `CompletionPayload` contract from [SPEC-004](../spec/SPEC-004-SEMANTICS.md).

It exists to stop the Phase 67 retained-completion work from accreting payload slices ad hoc.

The corpus already has:

- the canonical semantic execution-record contract in
  [docs/reference/semantic-execution-record-contract.md](semantic-execution-record-contract.md);
- real retained-completion runtime slices from TASK-406 through TASK-412; and
- an explicit residual-gap classification in
  [MCE-007](../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md) that says retained completion remains
  useful but partial.

This document now freezes the missing boundary between those pieces.

This is a semantic/reference contract, not a concrete Rust API or promise that the current runtime
 already provides full parity.

## Scope

This contract defines:

1. what full retained-completion parity means;
2. how retained completion differs from the canonical live/sealed execution record;
3. which retained dimensions must be exact for full parity;
4. which current retained slices are conservative, subset-only, or exact today;
5. how retained completion relates to conformance versus optional runtime enrichment.

This contract does not define:

- one mandatory Rust struct layout for retained completion;
- one public API shape for completion waiting or lookup;
- one requirement that retained completion transport full execution trace `T`;
- one claim that current `ash-interp` already satisfies full parity.

## 1. Canonical Semantic Target

Per [SPEC-004](../spec/SPEC-004-SEMANTICS.md), the semantic completion payload for a terminal child
completion is:

```text
CompletionPayload ::= {
  result: Result<Value, Error>,
  obligations: ObligationState,
  provenance: Provenance,
  effects: EffectTrace,
}
```

where:

- `result` is the child's exact terminal `Ok(v)` or `Err(err)` outcome;
- `obligations` is the child's authoritative terminal obligation state;
- `provenance` is the child's authoritative terminal provenance state;
- `effects` is the child terminal `EffectTrace { terminal, reached }` summary.

This `CompletionPayload` is the semantic parity target for retained completion observation.

## 2. Relationship to the Semantic Execution Record

Retained completion is not the same thing as the canonical execution record.

The canonical execution record from
[semantic-execution-record-contract.md](semantic-execution-record-contract.md) carries:

- `phase`,
- exact cumulative `Ω`,
- exact cumulative `π`,
- exact cumulative trace `T`,
- exact cumulative effect summary `ε̂`.

Retained completion is instead a downstream terminal observation surface over a sealed terminal child
completion.

Normatively:

1. retained completion is derived from a terminal child execution state;
2. retained completion may project a subset of the canonical execution record;
3. retained completion never replaces the canonical execution record as the source of full semantic
   state;
4. absence of full retained-completion parity does not redefine the canonical execution-record
   contract.

In particular:

- exact trace `T` remains owned by the canonical execution record and workflow-outcome projection;
- retained completion is about parity with `CompletionPayload`, not parity with the full execution
  record;
- control tombstones remain distinct from child-owned retained completion payloads.

## 3. Retained Completion Observation Classes

This contract distinguishes four classes.

### 3.1 Full `CompletionPayload` parity

A retained completion surface has full parity when it preserves exactly the same semantic contents as
the semantic `CompletionPayload` target:

```text
RetainedCompletion ≃ CompletionPayload
```

for the dimensions named by `CompletionPayload`:

- exact `result`
- exact terminal `obligations`
- exact terminal `provenance`
- exact terminal `effects.terminal`
- exact terminal `effects.reached`

### 3.2 Conservative retained slice

A retained field is conservative when it preserves an honest upper bound, approximation, or weaker
projection that is still useful but not exact parity.

Examples:

- conservative effect upper bounds;
- conservative provenance lineage/identity summaries.

### 3.3 Terminal-visible subset only

A retained field is subset-only when it preserves only the portion of the terminal state the current
runtime can honestly observe at the retained-completion boundary.

Examples:

- terminal-visible obligations slices that do not claim full hidden/cumulative parity.

### 3.4 Out of scope for retained completion parity

Some semantic dimensions remain intentionally outside retained-completion parity itself.

Most importantly:

- full execution trace `T`;
- live/nonterminal execution-state exposure;
- scheduler/interleaving realization details.

## 4. Exactness Matrix

The table below freezes the retained-completion parity categories.

| Dimension | Full retained-completion parity requires | Conservative retained slice allowed | Terminal-visible subset allowed | Out of scope here |
|---|---|---|---|---|
| `result` | Exact `Ok(v)` / `Err(err)` transport | No, if claiming parity | No, if claiming parity | transport/storage shape |
| `obligations` | Exact terminal child obligation state | No, if claiming parity | Yes for staged retained surfaces, if labeled subset-only | hidden runtime packaging details |
| `provenance` | Exact terminal child provenance state | Yes for staged retained surfaces, if labeled conservative | No, if claiming exact parity | storage layout |
| `effects.terminal` | Exact terminal effect | Yes for staged retained surfaces, if labeled conservative upper bound | No, if claiming exact parity | effect-summary algorithm details |
| `effects.reached` | Exact reached-effect set | Yes for staged retained surfaces, if labeled conservative upper bound | No, if claiming exact parity | concrete set encoding |
| trace `T` | Not required for retained-completion parity | omission allowed | omission allowed | yes — owned by execution record, not `CompletionPayload` |
| blocked/nonterminal state | Not part of terminal retained parity | n/a | n/a | yes |
| control tombstone state | Must remain distinguishable from child-owned completion payloads | n/a | n/a | exact tombstone API shape |

## 5. Current Runtime Classification for TASK-406 Through TASK-412

The current runtime corpus is compatible with this contract but does not yet satisfy full parity.

Current retained-completion slices classify as follows:

| Runtime surface | Current status under this contract | Notes |
|---|---|---|
| `RetainedCompletionRecord.result` | exact for the retained `result` dimension | Child-owned completions preserve the direct terminal `Result<Value, ExecError>`; control tombstones keep `None`. |
| `RetainedCompletionRecord.effects` | exact for child-owned retained completions | Child-owned completions now project the exact semantic `EffectTrace` from the sealed terminal child execution record; control tombstones still keep `None`. |
| `RetainedCompletionRecord.obligations` | terminal-visible subset only | Current runtime preserves the terminal-visible obligations slice it can honestly snapshot, not exact full parity. |
| `RetainedCompletionRecord.provenance` | conservative | Current runtime preserves runtime-owned child identity/spawn-lineage slices, not exact full terminal provenance parity. |
| retained completion wait API | compatible observation/wait surface only | Waiting for the first sealed retained record is useful, but waiting does not itself imply parity. |

So the current honest corpus verdict is:

```text
retained completion observation = materially useful, partially enriched, not full CompletionPayload parity
```

## 6. Conformance vs Optional Enrichment

This contract distinguishes semantic conformance from optional runtime enrichment.

### 6.1 Conformance-relevant statements

For conformance-sensitive retained-completion claims:

1. if an implementation claims full retained `CompletionPayload` parity, the parity dimensions above
   must be exact;
2. if a retained field is conservative or subset-only, it must be labeled as such and must not be
   compared as if it were exact parity;
3. control tombstones must remain distinguishable from child-owned retained completion payloads.

### 6.2 Optional runtime enrichment

An implementation may enrich retained completion beyond the current runtime, for example by adding:

- exact terminal obligations parity;
- exact terminal provenance parity;
- exact reached-effect transport;
- additional diagnostic metadata.

Such enrichment is allowed, but it must not blur the difference between:

- exact parity,
- conservative summary,
- terminal-visible subset,
- and non-parity diagnostic data.

## 7. Follow-On Guidance for TASK-437 and TASK-439

This contract is the direct input for:

- [TASK-437: Retained-Completion Parity Follow-On](../plan/tasks/TASK-437-retained-completion-parity-follow-on.md)
- [TASK-439: Differential Conformance Harness (Rust First)](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md)

Normatively:

1. TASK-437 should improve exactly one retained-completion dimension or slice at a time;
2. each such improvement must be labeled as exact, conservative, or subset-only;
3. TASK-439 must compare retained-completion observations using these fidelity labels rather than
   silently treating all retained fields as exact semantic parity.

## 8. Honest Current Boundary

The current corpus should now be read as saying:

- full semantic execution-record contract: frozen separately and still broader than retained
  completion;
- retained completion observation: real and useful;
- retained `result`: exact for that single dimension;
- retained `effects`: exact for child-owned retained completions;
- retained `obligations`: terminal-visible subset only;
- retained `provenance`: conservative;
- full `CompletionPayload` parity: still open follow-on work.
