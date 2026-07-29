# Phase 202 Closeout Review

**Date:** 2026-07-24
**Scope:** PLAN-202 formal-semantics and verification-programme closeout (TASK-1994).
**Result:** Complete as a documentation, calculus, traceability, and pilot-evidence programme;
**conditional no-go** for claiming direct Rust verification or beginning Ash proof-syntax
implementation.

## Decision

The programme has produced the required semantic authority, archive routing, staged calculus,
traceability, isolated-toolchain, and pilot evidence.  It has **not** proved production Ash Rust,
nor approved an Ash `spec`/`proof` syntax.  The two Verus artifacts are model proofs with
empty declared logical escapes; their direct Rust-to-model refinement obligations remain deferred.
This is an evidence-backed closeout, not a relabelling of those gaps as complete.

## Requirement-to-evidence matrix

| PLAN-202 completion requirement | Evidence | Result and remaining owner |
|---|---|---|
| One canonical manifest resolves productive authority claims | `docs/spec/CANONICAL-CORPUS.json`, `docs/spec/CANONICAL-CORE.md`, canonical-corpus validator | Complete for the scoped canonical overlay. Realization gaps remain explicitly represented in the trace graph. |
| Agent routing selects canonical sources and excludes archive/research authority | generated context/redirect artifacts from TASK-1987 and `scripts/check-docs-gate.sh` | Complete for the frozen productive routes; this is routing evidence, not implementation conformance. |
| Superseded material is inert with redirects and no productive inbound routing | TASK-1987 packet, redirect map, documentation gate | Complete for audited routes. Historical material remains available as provenance only. |
| Deprecation/removal items have semantic ownership and behavior evidence | TASK-1988 audit packet and TASK-2000--TASK-2008 / TASK-439 task files | Ownership complete; production realization is deferred to the named tasks. None of these follow-ups is silently discharged here. |
| Core/CPS calculus and theorem ladder are frozen with exclusions | `docs/spec/ASH-CPS-CALCULUS.{json,md}` and calculus validator | Complete as the bounded semantic pivot. Kernel/effect metatheory, lowering preservation, and runtime parity remain deferred trace obligations. |
| Traceability links rules to implementation/tests/proofs or explicit gaps | `docs/spec/SEMANTIC-TRACEABILITY.json` and traceability validator | Complete as a graph and coverage mechanism. `proved` model nodes are not direct Rust proof claims; deferred bridge nodes retain the difference. |
| Verus toolchain/pilot sequence has reproducible outcomes and enumerated assumptions | `verification/verus/{README.md,tcb-report.json}`, row/frame manifests, runners, and reports | Toolchain: narrow GO. Pilots: verified-model-only, with no logical escapes reported. Conditional NO-GO for production-proof expansion until checked Rust/model adapters exist. |
| Later Ash proof design has an evidence-backed handoff without premature syntax | `docs/plan/ASH-PROOF-DESIGN-HANDOFF.md` | Complete as a design entry contract only. It authorizes no parser, typechecker, runtime, or proof-language implementation. |

## Pilot decision record

| Pilot | Checked outcome | Decision |
|---|---|---|
| TASK-1991 toolchain spike | Positive fixture: 1 verified / 0 errors; false fixture rejected; TCB logical-escape lists empty | **GO, narrow:** isolated verification experiments may use the pinned Verus release and Rust 1.96.0. Cargo remains independent. |
| TASK-1992 row algebra | 15 model proof items / 0 errors under `--no-cheating`; 18 focused Rust tests | **NO-GO for direct production proof:** no verified `CoreRow`/`CoreRowItem` view or refinement. `REQ-CORE-ROW-DIRECT-BRIDGE-001` remains deferred. |
| TASK-1993 frame lookup | 8 model proof items / 0 errors; deliberate false lemma rejected; focused reverse-scan tests | **NO-GO for direct production proof:** no verified mapping for Rust frame/operation/payload carriers. `REQ-CPS-FRAME-LOOKUP-DIRECT-BRIDGE-001` remains deferred. The checked benchmark is not LLM-repair evidence. |

## Remaining gaps and ownership

- TASK-2000 through TASK-2008 own the audited vocabulary, grammar, lowering, Core/CPS boundary,
  runtime parity, public-API, and observable-projection decisions.
- TASK-439 remains the sole owner of reusable canonical differential conformance.
- `REQ-SEM-CPS-KERNEL-DEFERRED-001` and `REQ-SEM-EFFECT-DEFERRED-001` retain the theorem-ladder
  work outside the two pilot models.
- `REQ-CORE-ROW-DIRECT-BRIDGE-001` and
  `REQ-CPS-FRAME-LOOKUP-DIRECT-BRIDGE-001` retain direct Rust-refinement work.
- No evidence measures LLM lemma synthesis or repair.  The only checked broken-to-repaired case is
  a provenance-limited benchmark, so it cannot support a claim about an LLM, prompt, or model.

## Verification record

The closeout must be rechecked with the repository documentation gate and the three specialised
validators: canonical corpus, CPS calculus, and semantic traceability.  The toolchain and both
pilot runners are the authoritative reproductions for their separate JSON reports; their commands
and pinned versions are recorded in the associated README files.
