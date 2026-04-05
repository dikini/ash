---
status: draft
created: 2026-04-03
last-revised: 2026-04-05
tags: [analysis, roadmap, implementation, assessment]
---

# Ash Ideas Implementability Report

**Scope:** docs/ideas/minimal-core/, docs/ideas/type-system/, docs/ideas/otp/  
**Assessment Date:** 2026-04-05
**Assessor:** Hermes Agent (Software Engineer/Analyst)

---

## Executive Summary

This report assesses the implementability of 16 design explorations across three directories. The goal is to identify which ideas are ready for specification work, which need further exploration, and which have blocking dependencies.

### Overall Assessment

| Area | Documents | Readiness | Primary Blockers |
|------|-----------|-----------|------------------|
| minimal-core | 9 | Medium-High | MCE-003 design uncertainty, frozen MCE-007 true residual drift set (blocked-state classification, cumulative carriers, completion retention, `Par` aggregation), runtime cleanup |
| type-system | 5 | Low-Medium | Ad-hoc polymorphism design decisions |
| otp | 2 | Low | Generic/concrete split for behaviors |

### Key Findings

1. **MCE-001 (Entry Point)** is the most mature exploration—ready for SPEC-first implementation path
2. **Type system explorations** have significant theoretical depth but lack convergent design direction
3. **OTP explorations** identified a fundamental architectural uncertainty: the generic/concrete split without type classes
4. **Cross-cutting concern:** Ad-hoc polymorphism decisions in TYPES-002 block OTP progress and influence minimal-core

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
3. Spawn completion seals the child's own authoritative terminal obligation/provenance/effect state in `CompletionPayload`.
4. `Expr::Match` remains a primitive core expression, and `if let` lowers to `Expr::Match` with a wildcard fallback arm.

**Assessment:** MCE-004 is no longer an open research gap. It is completed documentation/planning alignment work captured by TASK-393.

---

### 1.5 MCE-005: Small-Step Semantics — PHASE 61 COMPLETE

**Status:** Accepted
**Implementability:** HIGH as planning/design corpus — backbone fixed, runtime realization still downstream

**Summary:**
Phase 61 converted MCE-005 from an exploratory note into an accepted small-step planning/design artifact. The corpus now fixes:

- workflow-first canonical configurations over `SPEC-001` workflows;
- ambient context as `(C, P)` and dynamic state in `Γ`, `Ω`, `π`, cumulative trace, cumulative effect summary, and residual workflow terms;
- a deliberate split between configuration-carried cumulative state and label-carried local step deltas;
- explicit blocked/suspended versus stuck distinction;
- a canonical rule inventory for workflow forms, with pure expressions/patterns remaining atomic in v1.

**Assessment:** MCE-005 is no longer blocked on foundational ambiguity. It is resolved enough to unblock MCE-006 while still remaining documentation/spec-planning work rather than interpreter implementation.

---

### 1.6 MCE-006: Small-Step ↔ IR Execution Alignment — PHASE 63 CLOSEOUT COMPLETE

**Status:** Drafting  
**Implementability:** MEDIUM-HIGH as planning/design corpus — semantic target fixed and Phase 63 runtime correspondence now packaged conservatively for downstream MCE-007 use

**Summary:**
MCE-006 now consumes the accepted MCE-005 backbone and, after Phase 63 / TASK-401 through TASK-404, includes a canonical semantic-carrier → runtime mapping table, one explicit operational correspondence section for residual control / blocked-suspended realization / completion-control authority, one explicit `Par` correspondence section, and one explicit closeout section for observable preservation, divergence taxonomy, and MCE-007 handoff packaging. The frozen runtime story is conservative: ordinary residual execution is primarily direct AST recursion, blocking receive is realized implicitly through mailbox/stream wait loops, yield suspension is explicit through `YieldState` plus `ExecError::YieldSuspended`, `ControlLinkRegistry` directly realizes reusable-versus-terminal control authority, `Workflow::Par` is documented as bulk async child execution with cloned branch-local `Context` state plus shared runtime registries and list-value terminal collation, and authoritative cumulative carriers for `π`, `T`, `ε̂`, and stronger terminal `Ω` / completion-payload packaging remain partial or missing.

**Assessment:** MCE-006 is now complete as a documentation/planning/runtime-correspondence phase. The resulting verdict is intentionally conservative: the current interpreter partially realizes the accepted small-step backbone for observable purposes, but MCE-007 should still keep cumulative-carrier and retained-completion rows marked partial/follow-up rather than closed.

**Recommendation:** Treat TASK-401 through TASK-404 as the frozen MCE-006 evidence packet. MCE-007 should consume that packet directly and avoid reopening carrier/control/`Par` correspondence stories unless new runtime evidence appears.

---

### 1.7 MCE-007: Full Layer Alignment — CLOSEOUT ARTIFACT PUBLISHED, TRUE RESIDUAL DRIFT STILL OPEN

**Status:** Closeout artifact published  
**Implementability:** MEDIUM-HIGH as planning/closeout corpus — frozen runtime evidence is ingested, residual gaps are classified, and the final checklist/signoff artifact is now published; remaining work is runtime-side follow-on for the true residual drift set

**Summary:**
Consolidates all five layers: Surface → IR → Big-step → Small-step → Interpreter. TASK-398 consumes the frozen MCE-006 Phase 63 packet directly into the MCE-007 verification matrix, TASK-399 adds a dedicated residual-gap layer that classifies the remaining partial rows into packaging-only work, accepted partiality, and true residual drift with explicit owners, and TASK-400 publishes the final closeout/signoff/checklist section that freezes the accepted matrix state and current residual register.

**Assessment:** This remains a tracking/consolidation document, but it is no longer blocked on un-ingested MCE-006 evidence, on re-deriving which residuals are real drift, or on missing closeout packaging. The corpus now explicitly treats packaged big-step ↔ small-step correspondence as closed by the TASK-400 closeout artifact, keeps rejected-vs-runtime-failure subtype cleanup as accepted partiality, freezes the true residual drift set to blocked/terminal/invalid runtime classification, authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging, retained completion-payload observation, and full helper-backed `Par` aggregation, and keeps sequencing / binding / branching explicit as a mixed case: locally aligned execution with one remaining cumulative-carrier drift dependency.

**Recommendation:** Use as the living closeout matrix, frozen residual register, and signoff/checklist artifact. Future work should focus on runtime/interpreter follow-on for the true residual drift set, not on re-deriving the Phase 63 packet or rebuilding the closeout prose.

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

### 2.1 TYPES-001: Tuple Variant Syntax — SYNTAX DECISION NEEDED

**Status:** Drafting  
**Implementability:** HIGH — Pure syntax choice

**Summary:**
Proposes syntax for tuple-style enum variants (needed for `RuntimeError Int String`). Options: space-separated (ambiguous), explicit tuple `()` (recommended), or hybrid.

**Assessment:**
This is a straightforward syntax decision. Option B (explicit tuple `()`) is recommended and unambiguous. No deep semantic issues.

**Recommendation:** Make decision, update SPEC-002 grammar, implement. Low risk.

---

### 2.2 TYPES-002 V1/V2: Ad-Hoc Polymorphism — DEEP DESIGN SPACE

**Status:** V1=Drafting, V2=Reviewing  
**Implementability:** LOW — Fundamental type system extension

**Summary:**
V1 explores four approaches: Haskell-style typeclasses, Rust-style traits, capability-inspired interfaces, minimal constraints. V2 synthesizes, identifies **closed-world interfaces** as most promising direction.

**Key Insight from V2:**
> "Capabilities and interfaces are not the same thing. Capabilities represent runtime authority and governed access to external resources. Interface constraints describe what operations are available for a type or family of types."

**Assessment:**
This is the deepest design space in the type system explorations. Decision affects:
- OTP (generic/concrete split for behaviors)
- MCE-003 (functions vs capabilities—are typeclass methods functions?)
- Authority elevation semantics (TYPES-002 V2 Section 4)

**Risk:** VERY HIGH. This is a major language feature that touches almost everything.

**Recommendation:** V2's approach of keeping options open while identifying design pressures is correct. Need workload-driven evaluation before committing. Suggest:
1. Expand workload table with concrete Ash examples
2. Prototype closed-world interface sketch
3. Explore explicit evidence passing as semantic model
4. Do NOT implement until design pressure is better understood

---

### 2.3 TYPES-003: Capability and Effect Vocabulary — PROSE REFINEMENT

**Status:** Drafting  
**Implementability:** HIGH — Documentation only

**Summary:**
Disambiguates "capability" as used across specs: capability declaration, capability identity, capability witness, provider, effect, policy context, obligation context, provenance context.

**Assessment:**
This is clarifying documentation, not implementation. Proposes Direction 2 (split prose vocabulary, preserve surface syntax).

**Recommendation:** Adopt Direction 2 immediately. Update existing specs to use precise terminology. No code changes required.

---

### 2.4 TYPES-004: Effect Typing Foundations — SOLID FOUNDATION

**Status:** Drafting  
**Implementability:** MEDIUM-HIGH — Build on existing specs

**Summary:**
Proposes practical starting point for effect typing:
- Add `Pure` as bottom element to existing lattice
- Enumerate effect-producing workflow forms
- Composition by join
- Capability declarations constrain but don't replace effect typing

**Assessment:**
This is well-scoped and builds on existing spec anchors (SPEC-001, SPEC-003, SPEC-004). The proposal to add `Pure` grade is sensible.

**Recommendation:** Proceed with implementation. This is tractable incremental work.

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

### 3.2 OTP-002: Ash OTP Design — BLOCKED ON AD-HOC POLYMORPHISM

**Status:** Drafting  
**Implementability:** LOW — Blocked on TYPES-002

**Summary:**
Explores four options: Direct Erlang Port (rejected), Capability-Based Supervision (recommended), Runtime-Based Isolation (out of scope), Structured Concurrency Integration.

**Recommended Approach:** Capability-Based Supervision with:
- Task trait for units of work
- Child specifications with typed errors
- Supervisor capabilities with restart policies
- Event streaming

**Assessment:**
The document correctly identifies that the primary blocker is the generic/concrete split (OTP-001 Section 9). This maps directly to TYPES-002 (ad-hoc polymorphism).

**Key Quote:**
> "The primary architectural uncertainty is the generic/concrete split (OTP-001, Section 9), not fundamental semantic mismatches."

**Risk:** HIGH. Cannot proceed meaningfully without resolving how Ash expresses "generic code parameterized by concrete implementation."

**Recommendation:**
1. Block OTP implementation on TYPES-002 resolution
2. In parallel, investigate besedarium session types for typed message protocols (noted as relevant but difficult in Ash)
3. Document that OTP is a secondary priority until type system foundations settle

---

## 4. Cross-Cutting Concerns

### 4.1 The Ad-Hoc Polymorphism Dependency

**Pattern:** TYPES-002 is a dependency for:
- OTP-002 (generic/concrete split)
- MCE-003 (functions vs capabilities—are methods functions?)
- MCE-002 (Call form semantics if methods are different from functions)

**Implication:** TYPES-002 is on the critical path for significant future features.

### 4.2 The Capability/Effect/Interface Distinction

**Progress:** TYPES-003 clarifies this vocabulary.

**Key Distinctions:**
- **Capability:** Runtime authority, governed access
- **Effect:** Classification of computation
- **Interface:** Type-indexed abstraction (proposed)

**Implication:** These should remain separate mechanisms. TYPES-002 V2 correctly warns against capability/interface unification.

### 4.3 The Pure Grade Addition

**Proposal:** TYPES-004 suggests adding `Pure` below `Epistemic` in the effect lattice.

**Impact:** Affects:
- SPEC-001 (effect lattice)
- All workflow form effect tables
- Diagnostic messages

**Assessment:** Low-risk, high-value addition.

---

## 5. Recommendations

### 5.1 Immediate Actions (Next 2 Weeks)

| Action | Owner | Rationale |
|--------|-------|-----------|
| Proceed with MCE-001 SPEC phase (57A tasks) | Core team | Most mature, unblocks entry point |
| Adopt TYPES-003 vocabulary in existing specs | Documentation | Immediate clarity improvement |
| Use TASK-370 audit findings to scope any future IR-consolidation work | Documentation/Core team | MCE-002 is complete; future work should build on the accepted audit |
| Decide TYPES-001 tuple syntax | Language team | Simple syntax decision, unblocks RuntimeError |
| Add `Pure` grade per TYPES-004 | Type system | Foundation for effect typing |

### 5.2 Short-Term Actions (Next Month)

| Action | Owner | Rationale |
|--------|-------|-----------|
| Expand TYPES-002 V2 workload table | Research | Data-driven design decision |
| Prototype closed-world interface sketch | Research | Test Direction 1 feasibility |
| Inventory current runtime components | Sub-agent | Unblocks MCE-008 |
| Draft example programs (MCE-009) | Design | Design probes for other features |
| Document generic/concrete split options | Research | Clarify OTP blocker |

### 5.3 Medium-Term Actions (Next Quarter)

| Action | Owner | Rationale |
|--------|-------|-----------|
| Resolve TYPES-002 ad-hoc polymorphism | Language team | Unblocks OTP and influences MCE-003 |
| Use the published TASK-400 closeout artifact as the baseline for any future runtime drift follow-on | Research / Core team | The matrix, residual register, signoff conditions, and drift-prevention checklist are now frozen; later work should resolve true drift rather than repackage the closeout |
| Implement MCE-001 Phase 57B | Core team | Entry point implementation |
| Decide MCE-003 functions vs capabilities | Language team | Fundamental language design |

### 5.4 Deferred Actions

| Action | Blocked On | ETA |
|--------|-----------|-----|
| OTP implementation | TYPES-002 resolution | Post-type-system |
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
8. **TYPES-004 Pure grade** — Additive change
9. **MCE-001 entry point** — Design mature, well-specified

---

## 7. Conclusion

The Ash ideas collection shows healthy exploration across multiple dimensions. The maturity gradient is clear:

- **Ready now:** MCE-001 (entry point), TYPES-001/003/004 (type system refinements)
- **Needs research:** TYPES-002 (ad-hoc polymorphism), MCE-003 (functions vs capabilities)
- **Blocked:** OTP-* (depends on type system), full runtime-side closure of the MCE-007 true residual drift set

The critical path runs through TYPES-002. Resolution of ad-hoc polymorphism will unblock OTP and inform MCE-003. Until then, work should focus on:

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
- MCE-006: Small-Step ↔ IR Execution — Drafting
- MCE-007: Full Layer Alignment — Drafting
- MCE-008: Runtime Cleanup — Drafting
- MCE-009: Test and Example Workflows — Drafting

### Type System (TYPES-*)
- TYPES-001: Tuple Variant Syntax — Drafting
- TYPES-002 V1/V2: Ad-Hoc Polymorphism — Drafting/Reviewing
- TYPES-003: Capability and Effect Vocabulary — Drafting
- TYPES-004: Effect Typing Foundations — Drafting

### OTP (OTP-*)
- OTP-001: Erlang/OTP Analysis — Drafting
- OTP-002: Ash OTP Design — Drafting

---

*Report generated by systematic review of all documents in scope. Assessment based on document maturity, identified blockers, and cross-reference analysis.*
