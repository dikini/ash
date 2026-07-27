---
id: docs.plan.semantic-rule-coverage
title: Semantic Rule Coverage Map
kind: implementation-coverage-map
status: active
authority: planning-and-review
last_verified: 2026-07-27
---

# Semantic Rule Coverage Map

This is the human-review surface for target semantic-rule coverage. Canonical specs own
semantics; `docs/spec/SEMANTIC-TRACEABILITY.json` owns machine-validated links. This map makes a
rule's declared domain and gaps explicit before implementation.

For semantic work, link a task to one or more rows below and update the row before writing a
fixture. A source example is evidence only. A general rule is complete only when its declared
domain has Type, Core, CPS, admission, runtime, diagnostics, and evidence coverage. Status values
are **general**, **bounded**, **planned**, **deferred**, or **not applicable**.

| Rule family / canonical owner | Type | Core | CPS | Admission / runtime | Evidence | Declared domain and next obligation |
| --- | --- | --- | --- | --- | --- |
| Surface forms and source-to-Core (`SPEC-095b`, `SPEC-098c`) | bounded | bounded | — | — | bounded | Pure entries, declared operation facts, and selected handler facts; general expression, call, closure, pattern, and import lowering is planned. |
| Calls and continuations (`SEM-CPS-CALL-001`, `SEM-CPS-JUMP-001`) | bounded | bounded | bounded | bounded | bounded | Exact local call and pure forms only; general calls, parameters, closures, recursion, and imports are planned. |
| Core control and terminals (`SEM-CPS-LETVAL-001`, `SEM-CPS-IF-001`, `SEM-CPS-RETURN-001`, `SEM-CPS-TRAP-001`) | bounded | bounded | bounded | bounded | bounded | Approved pure ANF and selected control forms only; general source control lowering is planned. |
| Operations and lookup (`SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`) | bounded | bounded | bounded | bounded | bounded | Concrete declared/built-in operations only; arbitrary operations, arguments, imports, and chains are planned. |
| Handlers and deep affine resume (`SEM-EFFECT-HANDLE-001`, `SEM-EFFECT-DEEP-AFFINE-HANDLE-001`) | bounded | bounded | bounded | bounded | bounded | Exact closed-row witnesses only; general multi-clause, open-row, imported, and multi-shot behavior is planned. |
| Rows and imported summaries (`SPEC-097b`, `TYPE-TARGET-ROW-001`) | bounded | bounded metadata | n/a | non-authorizing | bounded | V8 structural summaries and selected closed rows; general row polymorphism, expansion, and discharge is planned. |
| Production admission and frames (`TASK-2004`, `TASK-2014`) | bounded | bounded | bounded | bounded | bounded | Path-B selected artifacts only; general artifacts and all route coverage are planned. |
| Terminal envelopes and async control (`TASK-2008`, `TASK-2014`) | bounded | n/a | bounded | bounded | bounded | Selected return/rejection/trap/timeout/cancellation routes; full route matrix is planned. |
| Differential parity (`TASK-2005`, `TASK-439`) | n/a | bounded private targets | bounded private targets | non-production | bounded | Trusted case- and fingerprint-locked corpus controls only; general parity is planned. |
| Contracts, predicates, and proofs (`SPEC-098b`, `SPEC-100`) | bounded | bounded sidecars | — | — | bounded | Predicate provenance only; general discharge, proof, and runtime contract semantics are planned. |

## Required task record

Each linked task records: canonical rule/spec section; declared domain; layer status changed;
positive, negative, mutation, and parity evidence where applicable; non-goals; and the next gap.
Reviewers reject a claim that a passing fixture implements a general rule without this row update.
