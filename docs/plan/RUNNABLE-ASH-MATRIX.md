---
id: docs.plan.runnable-ash-matrix
title: Runnable Ash Matrix
kind: integration-route-matrix
status: active
authority: planning-and-review
owner: plan-203
last_verified: 2026-07-28
---

# Runnable Ash Matrix

This is TASK-2032's bounded integration ledger. **Adapter parity** means the two in-process client
adapters submit the same Engine-issued admitted-program request and receive the same normalized
terminal envelope. The Engine derives a fresh deadline for each submission from the request's
retained timeout configuration, while cancellation is shared and sticky. This is not evidence that
the daemon service accepts the corresponding source or has its provider/binding configuration. The
actual daemon service is called out separately so no adapter result is mislabeled as a daemon-active
route. Every unsupported service route has an explicit rejection; no row authorizes direct
evaluation.

| Rule/source family | Layer owner and artifact | In-process adapter seam | Actual daemon service | Evidence and explicit boundary |
| --- | --- | --- | --- | --- |
| Selected noncanonical pure `fn main() -> Int` | TASK-2004 checked Core/CPS admission | same-request return parity | rejected before worker startup: daemon indexes only `Result<(), RuntimeError>` entries | `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY`; `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`. Unsupported pure lowering also rejects at Engine admission. |
| Canonical pure `fn main() -> Result<(), RuntimeError>` | TASK-2004 checked Core/CPS entry admission | no opaque request crosses the daemon transport | daemon-active for the bounded success/status route; the alpha protocol exposes lifecycle status, not a V1 terminal envelope | `daemon_start_execute_uses_hashed_source_bytes_after_drift_check`. The daemon still uses the shared Engine seam; a transport V1-terminal parity contract is unimplemented. |
| Selected `time::sleep` | TASK-2014 sealed checked-CPS/provider-frame admission | same-request timeout/cancellation parity | rejected before worker startup by the canonical-entry gate | Engine timeout/cancellation in `TEST-TASK-2032-SHARED-ENGINE-SEAM-POSITIVE`; adapter parity and per-submission deadline reuse in `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY` and `TEST-TASK-2032-CLIENT-ADAPTER-DEADLINE-REUSE-PARITY`; `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`. A future daemon-active route also requires separately owned profile/binding configuration. |
| Selected `trap_sleep` handler body | TASK-2013 facts plus TASK-2014 admission | same-request trap parity | rejected before worker startup by the canonical-entry gate | Engine trap and adapter parity in `TEST-TASK-2032-SHARED-ENGINE-SEAM-POSITIVE` and `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY`; `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`. Client handler-name selection is not authority. |
| Selected `deep_affine_clock` handler | TASK-2013 checked handler facts and sealed two-frame Engine admission | same-request `Int(107)` parity | rejected before worker startup by the canonical-entry gate | `TEST-TASK-2032-SHARED-ENGINE-SEAM-POSITIVE`, `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY`, and `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`. No direct evaluator or row-derived frame is allowed. |
| Selected `forward_sleep` chain | TASK-2014 ordered frame instructions | same-request `Int(73)` parity with the explicit test provider/binding | rejected before worker startup by the canonical-entry gate | TASK-2026 sealed production-admission controls, `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY`, and `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`. A future daemon-active route additionally needs separately owned wake-provider/binding configuration. |
| `ash trace` client over selected pure admission | TASK-2032 trace lifecycle adapter and TASK-2004 checked Core/CPS admission | CLI-only admitted-program adapter | not a daemon route | `trace_output::trace_stdout_emits_a_document_for_an_admitted_pure_return`. Missing admission produces no partial trace document; trace projects only the Engine terminal envelope. |
| Missing/foreign admission | TASK-2004/TASK-2014 admission boundary | explicit rejection | `TEST-TASK-2032-SHARED-ENGINE-SEAM-NEGATIVE` | No direct-evaluator fallback. |
| Forged/malformed checked artifact | TASK-2014 provenance/frame authority | explicit rejection before dispatch | `TEST-TASK-2032-SHARED-ENGINE-SEAM-MUTATION` | No public summary or client can manufacture frames. |
| Unowned generic expressions, handlers, providers, and lowering gaps | Feature-owning task | explicit source-to-client rejection | matrix review by feature owner | No incomplete lowering selects a fallback. |

## TASK-2032 handoff boundary

TASK-2032 consumes TASK-2004/TASK-2014 checked admissions, TASK-2008 terminal projection, and
TASK-2031 correspondence. It produces only shared execution/parity integration and this matrix.
Parser, Core/CPS lowering, provider realization, handler semantics, terminal taxonomy, and daemon
transport remain separately owned.
