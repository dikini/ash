---
status: draft
created: 2026-04-03
last-revised: 2026-04-06
tags: [analysis, roadmap, implementation, assessment]
---

# Ash Ideas Implementability Report

**Scope:** docs/ideas/minimal-core/, docs/ideas/type-system/, docs/ideas/otp/  
**Assessment Date:** 2026-04-06
**Assessor:** Hermes Agent (Software Engineer/Analyst)

---

## Executive Summary

This report assesses the implementability of 17 design explorations across three directories. The goal is to identify which ideas are ready for specification work, which need further exploration, and which have blocking dependencies.

### Overall Assessment

| Area | Documents | Readiness | Primary Blockers |
|------|-----------|-----------|------------------|
| minimal-core | 9 | Medium-High | MCE-003 design uncertainty, frozen MCE-007 true residual drift set (blocked-state classification, cumulative carriers, completion retention, `Par` aggregation), runtime cleanup |
| type-system | 6 | Low-Medium | Closed-world-interface MVP still needs normative spec and later implementation work |
| otp | 2 | Low | Generic/concrete split for behaviors |

### Key Findings

1. **MCE-001 (Entry Point)** is the most mature exploration—ready for SPEC-first implementation path
2. **Type system explorations** still have significant theoretical depth, but `TYPES-002` now has a narrowed closed-world-interface MVP follow-on target instead of only a broad `v1`/`v2` exploration pair
3. **OTP explorations** identified a fundamental architectural uncertainty: the generic/concrete split without type classes
4. **Cross-cutting concern:** Ad-hoc polymorphism still blocks OTP progress and influences minimal-core, but the active target is now the closed-world-interface MVP boundary rather than the whole exploration space

---

## 1. Minimal Core Explorations (MCE-*)

### 1.1 MCE-001: Entry Point — READY FOR SPEC PHASE

**Status:** Candidate (most mature)  
**Implementability:** HIGH — Ready for Phase 57A (SPEC updates)

**Summary:**
This exploration resolved all major entry point questions through systematic design deliberation. Decisions reached include:
- Hidden system supervisor model
- Static lexical scoping with `use` imports
- Capability parameter injection for main workflow
- Args as capability from standard library
- No library vs executable distinction (CLI determines entry)

**Implementation Path:**
Document explicitly calls for SPEC-first approach:
- TASK-S57-1 through TASK-S57-6 update normative specs
- Only then proceed to implementation (TASK-359 through TASK-369)

**Assessment:** This is ready to proceed. The exploration itself states "All implementation blocked on SPEC updates (57A tasks)." This is correct prioritization.

**Risk:** Low. Design is mature, dependencies are well-understood.

---

### 1.2 MCE-002: IR Core Forms Audit — COMPLETE

**Status:** Accepted
**Implementability:** COMPLETE — Closed via TASK-370 and the audit report

**Summary:**
The IR audit is complete. The repository now has a formal audit report documenting the current canonical core carriers, the active duplication layers, and conservative recommendations for future consolidation work.

**Key Results:**
- Identified `crates/ash-core/src/ast.rs` as the de facto primary core-AST carrier.
- Confirmed `Workflow::Seq` is primitive and rejected `Seq` → `Let`.
- Confirmed `Expr::IfLet` is sugar over `Expr::Match`.
- Deferred deeper eliminations until semantics and lowering are clearer.

**Assessment:** MCE-002 is no longer a missing-inventory blocker. It now serves as an accepted input to later work such as MCE-004 closeout and MCE-007 full-stack alignment.

**Risk:** Medium. Form elimination affects all downstream semantics.

---

### 1.3 MCE-003: Functions vs Capabilities — OPEN DESIGN QUESTION

**Status:** Drafting  
**Implementability:** LOW — Fundamental language design unresolved

**Summary:**
Asks whether Ash needs standalone functions or if workflows + capabilities are sufficient. Three approaches: Pure Workflows, Functions as Distinct, Capabilities as Interface.

**Assessment:**
This is a fundamental design question with far-reaching implications:
- Affects MCE-002 (Call form semantics)
- Affects TYPES-002 (ad-hoc polymorphism—are methods functions?)
- Affects OTP-002 (Task trait—are tasks functions or workflows?)

**Risk:** HIGH. Decision needed before many other features can stabilize.

**Recommendation:** This needs user ergonomics research and performance analysis. Suggest prototyping examples in all three approaches to gather data.

---

### 1.4 MCE-004: Big-Step Semantics Alignment — RESOLVED

**Status:** Accepted
**Implementability:** COMPLETE — Alignment recorded in current corpus via TASK-393

**Summary:**
This exploration started as a gap-analysis note for surface syntax, IR, and big-step semantics alignment, but the relevant questions are now settled by the existing corpus. `SPEC-001` defines the canonical IR, the parser-to-core lowering contract defines the surface-to-core handoff, and `SPEC-004` now provides explicit workflow/expression/pattern judgments plus helper contracts after TASK-350.

**Resolved Decisions:**
1. `Workflow::Seq` stays primitive; MCE-002 rejected any `Seq` → `Let` rewrite because `Seq` composes workflows while `Let` binds an `Expr`.
2. `Par` effect aggregation is defined in `SPEC-004` as branch-effect join in the all-success case, with helper-backed obligation/provenance aggregation.
3. Spawn completion seals the child's own authoritative terminal obligation/provenance/effect state in `CompletionPayload`; the current runtime now partially realizes this through retained `result` plus a conservative retained effect summary, while exact obligation/provenance/effect parity remains open.
4. `Expr::Match` remains a primitive core expression, and `if let` lowers to `Expr::Match` with a wildcard fallback arm.

**Assessment:** MCE-004 is no longer an open research gap. It is completed documentation/planning alignment work captured by TASK-393.

---

### 1.5 MCE-005: Small-Step Semantics — PHASE 61 COMPLETE

**Status:** Accepted
**Implementability:** HIGH as planning/design corpus — backbone fixed, runtime realization still downstream

**Summary:**
Phase 61 converted MCE-005 from an exploratory note into an accepted small-step planning/design artifact, TASK-427 closes out [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) as the docs/spec home for that accepted contract, and TASK-430 now packages the remaining helper-owned boundaries plus the frozen proof-usable state taxonomy. The corpus now fixes:

- workflow-first canonical configurations over `SPEC-001` workflows;
- ambient context as `(C, P)` and dynamic state in `Γ`, `Ω`, `π`, cumulative trace, cumulative effect summary, and residual workflow terms;
- a deliberate split between configuration-carried cumulative state and label-carried local step deltas;
- explicit blocked/suspended versus stuck distinction;
- one frozen state taxonomy distinguishing progress, blocked/suspended waiting, terminal success, terminal rejection/failure, and invalid/inadmissible/runtime-failure boundaries;
- explicit helper-owned contracts for receive selection, parallel aggregation, policy decision ownership, obligation transition/discharge ownership, and spawned-child completion/control observation;
- a canonical rule inventory for workflow forms, with pure expressions/patterns remaining atomic in v1.

**Assessment:** MCE-005 is no longer blocked on foundational ambiguity. Its accepted semantic backbone is now also packaged in SPEC-025 as the stable docs/spec surface, while MCE-005 remains the accepted design reasoning backplane and MCE-006 remains the runtime-evidence backplane. This is resolved enough to unblock MCE-006 while still remaining documentation/spec-planning work rather than interpreter implementation.

---

### 1.6 MCE-006: Small-Step ↔ IR Execution Alignment — PHASE 63 CLOSEOUT COMPLETE

**Status:** Accepted  
**Implementability:** MEDIUM-HIGH as planning/design corpus — semantic target fixed and Phase 63 runtime correspondence now packaged conservatively for downstream MCE-007 use

**Summary:**
MCE-006 now consumes the accepted MCE-005 backbone and, after Phase 63 / TASK-401 through TASK-404, includes a canonical semantic-carrier → runtime mapping table, one explicit operational correspondence section for residual control / blocked-suspended realization / completion-control authority, one explicit `Par` correspondence section, and one explicit closeout section for observable preservation, divergence taxonomy, and MCE-007 handoff packaging. TASK-427 keeps [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) aligned with that evidence packet without promoting it into stronger implementation claims. The frozen runtime story is conservative: ordinary residual execution is primarily direct AST recursion, blocking receive is realized implicitly through mailbox/stream wait loops, yield suspension is explicit through `YieldState` plus `ExecError::YieldSuspended`, `ControlLinkRegistry` directly realizes reusable-versus-terminal control authority, `Workflow::Par` is documented as bulk async child execution with cloned branch-local `Context` state plus shared runtime registries and list-value terminal collation, and authoritative cumulative carriers for `π`, `T`, `ε̂`, and stronger terminal `Ω` / completion-payload packaging remain partial or missing.

**Assessment:** MCE-006 is now complete as a documentation/planning/runtime-correspondence phase. The resulting verdict is intentionally conservative: the current interpreter partially realizes the accepted small-step backbone for observable purposes, but MCE-007 should still keep cumulative-carrier and retained-completion rows marked partial/follow-up rather than closed.

**Recommendation:** Treat TASK-401 through TASK-404 as the frozen MCE-006 evidence packet. MCE-007 should consume that packet directly and avoid reopening carrier/control/`Par` correspondence stories unless new runtime evidence appears.

---

### 1.7 MCE-007: Full Layer Alignment — CLOSEOUT ARTIFACT PUBLISHED, TRUE RESIDUAL DRIFT STILL OPEN

**Status:** Closeout artifact published  
**Implementability:** MEDIUM-HIGH as planning/closeout corpus — frozen runtime evidence is ingested, residual gaps are classified, and the final checklist/signoff artifact is now published; remaining work is runtime-side follow-on for the true residual drift set

**Summary:**
Consolidates all five layers: Surface → IR → Big-step → Small-step → Interpreter. TASK-398 consumes the frozen MCE-006 Phase 63 packet directly into the MCE-007 verification matrix, TASK-399 adds a dedicated residual-gap layer that classifies the remaining partial rows into packaging-only work, accepted partiality, and true residual drift with explicit owners, and TASK-400 publishes the final closeout/signoff/checklist section that freezes the accepted matrix state and current residual register.

**Assessment:** This remains a tracking/consolidation document, but it is no longer blocked on un-ingested MCE-006 evidence, on re-deriving which residuals are real drift, or on missing closeout packaging. The corpus now explicitly treats packaged big-step ↔ small-step correspondence as closed by the TASK-400 closeout artifact, keeps rejected-vs-runtime-failure subtype cleanup as accepted partiality, freezes the true residual drift set to blocked/terminal/invalid runtime classification, authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging, retained completion-payload observation, and full helper-backed `Par` aggregation, and keeps sequencing / binding / branching explicit as a mixed case: locally aligned execution with one remaining cumulative-carrier drift dependency.

**Recommendation:** Use as the living closeout matrix, frozen residual register, and signoff/checklist artifact. Future work should focus on runtime/interpreter follow-on for the true residual drift set, not on re-deriving the Phase 63 packet or rebuilding the closeout prose. TASK-405 now provides the first such runtime-side follow-on by introducing a conservative authoritative runtime outcome/state classification in `ash-interp` for blocked/suspended, invalid/terminated, execution-failure, and terminal-success distinctions. TASK-406 is now complete for its scoped goal of introducing one explicit sealed/write-once retained completion-observation carrier in `ash-interp`. TASK-407 then supplies the runtime-owned spawned-child execution substrate and honest automatic completion sealing path. TASK-408 adds the next honest retained payload slice by preserving direct child terminal `Result<Value, ExecError>` data in `RetainedCompletionRecord.result` and exposing it via `terminal_result()`. TASK-428 now adds [SPEC-026](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) as the explicit cross-implementation conformance anchor for the accepted big-step, small-step, and runtime-observable surfaces without overclaiming current Rust runtime closure. The still-open follow-ons are obligations/provenance/effects parity, cumulative carriers, and helper-backed `Par` aggregation.

---

### 1.8 MCE-008: Runtime Cleanup — AUDIT NEEDED

**Status:** Drafting  
**Implementability:** MEDIUM — Needs runtime inventory

**Summary:**
Identifies minimal runtime surface: capability registry, library loader, FFI boundary, boot capability set. Proposes static linking initially.

**What's Missing:**
- Inventory of current runtime components
- Definition of minimal boot capability set
- FFI boundary design
- Capability registration mechanism

**Assessment:** Similar to MCE-002—needs codebase understanding before design can finalize.

**Recommendation:** Delegate sub-agent to inventory current runtime implementation in `ash-core`.

---

### 1.9 MCE-009: Test and Example Workflows — VALIDATION FRAMEWORK

**Status:** Drafting  
**Implementability:** LOW — Depends on all other explorations

**Summary:**
Defines test workflow categories, example programs, and success criteria for minimal core validation.

**Assessment:** This is a validation framework that cannot execute until the features it tests exist. However, drafting example programs (hello.ash, echo.ash, etc.) could inform other designs.

**Recommendation:** Proceed with drafting example programs—they serve as design probes. Full implementation waits for MCE-001 completion.

---

## 2. Type System Explorations (TYPES-*)

### 2.1 TYPES-001: Tuple Variant Syntax — DECISION MADE, READY FOR SPEC PROMOTION

**Status:** Candidate
**Implementability:** MEDIUM-HIGH — Source-contract work first, implementation next

**Summary:**
`TYPES-001` now selects one canonical tuple-variant syntax: explicit parenthesized payloads such as `RuntimeError(Int, String)` and matching positional patterns such as `RuntimeError(code, msg)`.

**Assessment:**
The syntax decision itself is now settled and low-risk. The remaining work is to align the normative ADT/surface/type/runtime docs first, then implement the feature across parser, ADT metadata, pattern matching, and runtime representation.

**Recommendation:** TASK-413 now completes the contract-promotion step for tuple variants. The next follow-on should be parser/typechecker/runtime implementation work against the frozen parenthesized tuple-variant contract.

---

### 2.2 TYPES-002 V1/V2/MVP: Ad-Hoc Polymorphism — NARROWED DOCS/SPEC TARGET NOW FROZEN

**Status:** V1=Drafting, V2=Reviewing, MVP Cut=Candidate
**Implementability:** LOW for implementation, MEDIUM-HIGH for follow-on normative spec work against the frozen MVP boundary

**Summary:**
V1 remains the preserved non-normative reasoning trace. V2 remains the broader polished exploration. The MVP cut is now the narrowed follow-on target: closed-world interfaces with explicit `interface`/`impl` declarations, constrained generic parameters, one canonical bound form (`T: Interface`), one canonical method-call form (`Interface::method(value)`), strong coherence, capability/interface separation, and effect-conservative methods in the first pass.

**Key Insight from V2:**
> "Capabilities and interfaces are not the same thing. Capabilities represent runtime authority and governed access to external resources. Interface constraints describe what operations are available for a type or family of types."

**Assessment:**
This remains the deepest type-system design space, but the repository now has a concrete next step rather than an open-ended one: future work can target the frozen closed-world-interface MVP boundary instead of reopening open-world typeclasses, associated items, dynamic dispatch, and capability/interface unification all at once.

**Risk:** VERY HIGH for direct implementation of the whole design space. Medium for work that stays inside the frozen MVP boundary.

**Recommendation:** [TASK-415](../plan/tasks/TASK-415-closed-world-interfaces-mvp-spec-cut.md) now completes the narrowing pass. Keep v1 as background only, use v2 plus the MVP cut as the serious discussion surfaces, and do not begin parser/typechecker/runtime work until normative specs are written against that frozen MVP boundary.

---

### 2.3 TYPES-003: Capability and Effect Vocabulary — READY FOR CORPUS CLEANUP

**Status:** Candidate
**Implementability:** HIGH — Documentation/spec cleanup only

**Summary:**
`TYPES-003` now acts as the reasoning record behind promoted vocabulary guidance. The repository now has a reusable prose target in `docs/reference/type-system-vocabulary-guidance.md` covering capability declarations, identities, witnesses, providers, effect classifications, policy context, obligation context, and provenance context.

**Assessment:**
This remains documentation/spec cleanup rather than implementation, but it is now actionable planning work rather than loose drafting.

**Recommendation:** TASK-414 now completes the main vocabulary-promotion pass. Follow-on work should focus on remaining corpus cleanup and any later implementation work that relies on the promoted terminology.

---

### 2.4 TYPES-004: Effect Typing Foundations — READY FOR NARROW CONTRACT PROMOTION

**Status:** Candidate
**Implementability:** MEDIUM-HIGH — Docs/spec convergence now, selective implementation later

**Summary:**
`TYPES-004` now serves as the basis for a narrow promoted task: freeze the current coarse workflow-form effect classifications, make provider effect metadata explicitly secondary to source-level effect typing, and stage the `Pure` question as a deliberate follow-up instead of mixing it silently into the current normative story.

**Assessment:**
This remains one of the strongest and most implementable type-system explorations. The current best next step is contract promotion and cleanup, not immediate whole-system redesign.

**Recommendation:** TASK-414 now freezes the narrow coarse effect-typing contract. Any later work on `Pure` should remain an explicit follow-on rather than an implicit normative change.

---

## 3. OTP Explorations (OTP-*)

### 3.1 OTP-001: Erlang/OTP Analysis — RESEARCH COMPLETE

**Status:** Drafting  
**Implementability:** N/A — Research document

**Summary:**
Comprehensive analysis of Erlang/OTP gen_server and supervisor. Documents message protocols, restart strategies, state management. Key finding: Ash/Erlang mapping is stronger than initially assessed.

**Revised Assessment (from document):**
| Aspect | Erlang/OTP | Ash |
|--------|-----------|-----|
| State isolation | Per-process | Per-workflow |
| Coordination | Message passing | Message passing via effects |
| Process identity | Pid | Workflow address |

**Critical Uncertainty Identified:**
The generic/concrete split for OTP behaviors—how to separate generic framework code from user-specific callbacks without type classes.

**Assessment:**
This is a research document, not implementation. Its value is identifying the architectural uncertainty that blocks OTP implementation.

---

### 3.2 OTP-002: Ash OTP Design — BLOCKED ON CLOSED-WORLD INTERFACES FOLLOW-ON

**Status:** Drafting  
**Implementability:** LOW — Blocked on the narrowed closed-world interfaces MVP follow-on after TASK-415

**Summary:**
Explores four options: Direct Erlang Port (rejected), Capability-Based Supervision (recommended), Runtime-Based Isolation (out of scope), Structured Concurrency Integration.

**Recommended Approach:** Capability-Based Supervision with:
- Task trait for units of work
- Child specifications with typed errors
- Supervisor capabilities with restart policies
- Event streaming

**Assessment:**
The document correctly identifies that the primary blocker is the generic/concrete split (OTP-001 Section 9). In the current repo state, that should be read through the narrowed closed-world interfaces MVP follow-on after TASK-415 rather than through the entire unconstrained TYPES-002 design space.

**Key Quote:**
> "The primary architectural uncertainty is the generic/concrete split (OTP-001, Section 9), not fundamental semantic mismatches."

**Risk:** HIGH. Cannot proceed meaningfully without resolving how Ash expresses "generic code parameterized by concrete implementation."

**Recommendation:**
1. Block OTP implementation on the closed-world interfaces MVP follow-on after TASK-415
2. In parallel, investigate besedarium session types for typed message protocols (noted as relevant but difficult in Ash)
3. Document that OTP is a secondary priority until type system foundations settle

---

## 4. Cross-Cutting Concerns

### 4.1 The Ad-Hoc Polymorphism Dependency

**Pattern:** The narrowed closed-world interfaces MVP follow-on after TASK-415 is now the relevant dependency surface for:
- OTP-002 (generic/concrete split)
- MCE-003 (functions vs capabilities—are methods functions?)
- future interface-constrained generic library work

**Implication:** the closed-world interfaces MVP boundary is now the practical critical path for significant future features that need interface-constrained generic code.

### 4.2 The Capability/Effect/Interface Distinction

**Progress:** TYPES-003 clarifies this vocabulary.

**Key Distinctions:**
- **Capability:** Runtime authority, governed access
- **Effect:** Classification of computation
- **Interface:** Type-indexed abstraction (proposed)

**Implication:** These should remain separate mechanisms. TYPES-002 V2 correctly warns against capability/interface unification.

### 4.3 The Pure Grade Follow-Up

**Proposal:** TYPES-004 suggests adding `Pure` below `Epistemic` in the effect lattice.

**Current status:** TASK-414 does not adopt `Pure` as already normative across the corpus. Instead,
it promotes a narrower contract and records `Pure` as explicit follow-up work until the affected
normative specs can be updated coherently together.

**Impact if promoted later:**
- SPEC-001 (effect lattice)
- workflow-form effect tables and inference examples
- diagnostic and reporting messages

**Assessment:** Valuable, but only once the corpus is updated coherently rather than piecemeal.

---

## 5. Recommendations

### 5.1 Immediate Actions (Next 2 Weeks)

| Action | Owner | Rationale |
|--------|-------|-----------|
| Proceed with MCE-001 SPEC phase (57A tasks) | Core team | Most mature, unblocks entry point |
| Adopt TYPES-003 vocabulary in existing specs | Documentation | Immediate clarity improvement |
| Use TASK-370 audit findings to scope any future IR-consolidation work | Documentation/Core team | MCE-002 is complete; future work should build on the accepted audit |
| Implement parser/typechecker/runtime support for canonical tuple variants after TASK-413 | Language/Core team | The source contract is now frozen; the remaining work is implementation against that contract |
| Keep `Pure` as explicit follow-up after TASK-414 and continue residual corpus cleanup | Documentation / Type system | The current coarse contract is now frozen without silent lattice drift |

### 5.2 Short-Term Actions (Next Month)

| Action | Owner | Rationale |
|--------|-------|-----------|
| Draft normative interface spec text directly from the frozen TASK-415 MVP boundary | Documentation / Language team | Advances the narrowed target without reopening the full design space |
| Keep any future interface sketches/prototypes inside the canonical MVP surface (`T: Interface`, `Interface::method(value)`) | Research | Tests feasibility without expanding scope |
| Inventory current runtime components | Sub-agent | Unblocks MCE-008 |
| Draft example programs (MCE-009) | Design | Design probes for other features |
| Document generic/concrete split options | Research | Clarify OTP blocker |

### 5.3 Medium-Term Actions (Next Quarter)

| Action | Owner | Rationale |
|--------|-------|-----------|
| Promote the TASK-415 closed-world-interface MVP into normative specs, then stage implementation work against that boundary | Language team | Unblocks OTP and influences MCE-003 without reopening the full exploration space |
| Use the published TASK-400 closeout artifact as the baseline for any future runtime drift follow-on | Research / Core team | The matrix, residual register, signoff conditions, and drift-prevention checklist are now frozen; later work should resolve true drift rather than repackage the closeout |
| Implement MCE-001 Phase 57B | Core team | Entry point implementation |
| Decide MCE-003 functions vs capabilities | Language team | Fundamental language design |

### 5.4 Deferred Actions

| Action | Blocked On | ETA |
|--------|-----------|-----|
| OTP implementation | Closed-world interfaces MVP follow-on after TASK-415 | Post-type-system |
| MCE-007 true residual runtime closure | Later runtime/interpreter follow-on for blocked-state classification, cumulative carriers, completion retention, and `Par` aggregation | Late in minimal-core |

---

## 6. Risk Assessment Summary

### High-Risk Items

1. **TYPES-002 ad-hoc polymorphism** — Decision affects language core
2. **MCE-003 functions vs capabilities** — Fundamental abstraction question
3. **OTP generic/concrete split** — Blocks significant runtime feature

### Medium-Risk Items

4. **MCE-002 IR form elimination** — Could break existing semantics
5. **MCE-008 runtime cleanup** — FFI boundary is always tricky

### Low-Risk Items

6. **TYPES-001 tuple syntax** — Pure syntax choice
7. **TYPES-003 vocabulary** — Documentation only
8. **TYPES-004 effect-typing contract** — Narrow contract promoted; `Pure` remains explicit follow-up
9. **MCE-001 entry point** — Design mature, well-specified

---

## 7. Conclusion

The Ash ideas collection shows healthy exploration across multiple dimensions. The maturity gradient is clear:

- **Ready now:** MCE-001 (entry point), TYPES-001/003/004 (type system refinements)
- **Needs research:** TYPES-002 (ad-hoc polymorphism), MCE-003 (functions vs capabilities)
- **Blocked:** OTP-* (depends on type system), full runtime-side closure of the MCE-007 true residual drift set

The critical path now runs through the narrowed closed-world interfaces MVP follow-on defined by TASK-415, rather than the unconstrained TYPES-002 design space as a whole. That MVP boundary should unblock more honest follow-on work for OTP and inform MCE-003. Until then, work should focus on:

1. Completing MCE-001 (entry point) — delivers user-visible value
2. Refining type system foundations (TYPES-001/003/004) — incremental improvements
3. Building data for TYPES-002 decision — workload-driven design

The explorations demonstrate good design discipline: separating concerns, identifying blockers, and avoiding premature commitment to complex features.

---

## Appendix: Document Index

### Minimal Core (MCE-*)
- MCE-001: Entry Point — Candidate status
- MCE-002: IR Core Forms Audit — Accepted
- MCE-003: Functions vs Capabilities — Drafting
- MCE-004: Big-Step Semantics Alignment — Accepted
- MCE-005: Small-Step Semantics — Accepted
- MCE-006: Small-Step ↔ IR Execution — Accepted
- MCE-007: Full Layer Alignment — Closeout artifact published
- MCE-008: Runtime Cleanup — Drafting
- MCE-009: Test and Example Workflows — Drafting

### Type System (TYPES-*)
- TYPES-001: Tuple Variant Syntax — Candidate
- TYPES-002: Ad-Hoc Polymorphism (v1 reasoning trace) — Drafting
- TYPES-002 V2: Ad-Hoc Polymorphism — Reviewing
- TYPES-002 MVP: Closed-World Interfaces MVP Cut — Candidate
- TYPES-003: Capability and Effect Vocabulary — Candidate
- TYPES-004: Effect Typing Foundations — Candidate

### OTP (OTP-*)
- OTP-001: Erlang/OTP Analysis — Drafting
- OTP-002: Ash OTP Design — Drafting

---

*Report generated by systematic review of all documents in scope. Assessment based on document maturity, identified blockers, and cross-reference analysis.*
