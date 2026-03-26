# Phase 44-46 Roadmap: Audit Convergence and Unified Capability-Role Synthesis

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Address all audit findings and selectively implement unified capability-role-workflow features with syntax discipline.

**Architecture:** Two-track approach: (1) Fix audit issues as immediate required work, (2) Implement brainstorm features only after syntax reduction review to avoid surface syntax bloat.

**Status:** Draft - Pending user review of syntax reduction decisions

---

## Executive Summary

This roadmap synthesizes two work sources:
1. **Audit Report (REQUIRED):** Concrete bugs and gaps that must be fixed
2. **Brainstorm Document (SELECTIVE):** Forward-looking capability-role-workflow design that needs syntax reduction before implementation

**Key Decision:** The brainstorm document introduces significant new syntax. Before implementing, we will conduct a syntax reduction review to eliminate redundant constructs and minimize surface expansion.

---

## Part 1: Feasibility Analysis

### Audit Report Analysis

**Source:** `docs/audit/codex-comprehensive-review.md`

**Nature of work:** Bug fixes, stub completion, quality gate remediation

**Risk level:** Low - well-defined, localized changes

**Dependencies:** None beyond existing codebase

**Must-fix categories:**
| Priority | Category | Count | Example |
|----------|----------|-------|---------|
| Critical | Unimplemented runtime | 2 | `Oblige`/`CheckObligation` execution |
| Critical | Placeholder lowering | 1 | `Yield` lowered to `Done` |
| High | Unsafe code | 1 | `SmtContext` Send/Sync |
| High | API gaps | 2 | `EngineBuilder` no-ops, stub providers |
| High | Quality gates | 3 | clippy, fmt, doc warnings |
| Medium | Silent failures | 2 | float truncation, null coercion |

### Brainstorm Document Analysis

**Source:** `todo-examples/definitions/hermes-conversations/unified-capability-role-workflow-synthesis.md`

**Nature of work:** Language extension with new constructs

**Risk level:** High - significant syntax expansion, semantic complexity

**Concerns identified:**
| Syntax | Concern | Current Status |
|--------|---------|----------------|
| `capability http = network + tls` | Composition operator adds complexity | Proposed |
| `file @ { paths: [...] }` | Constraint refinement at 3 levels | Proposed |
| `yield role(X_default)` | Implicit role reference leak | Proposed |
| `plays role(R)` + `capabilities: [...]` | Two ways to assign capabilities | Proposed |
| `yield workflow(X)` sugar | Multiple yield target syntaxes | Proposed |

**Syntax proliferation risk:** The brainstorm introduces ~5 new syntactic constructs that overlap with existing capability/role syntax. Without reduction, this could create multiple ways to express the same concept.

---

## Part 2: Syntax Reduction Decisions (Phase 45 Gate)

Before implementing brainstorm features, we apply these reduction principles:

### Decision 1: Keep `capabilities:` Direct Declaration

**What:** Workflows can declare capabilities directly without explicit role syntax.

**Rationale:** Brevity for common case; roles are for reusable bundles.

**Desugaring:** `capabilities: [network]` creates implicit role at lowering time.

### Decision 2: Omit `+` and `|` Composition Operators (Phase 45)

**What:** Remove capability composition operators from surface syntax.

**Rationale:** Can be achieved via role inclusion (`plays role(http_client)`) or explicit capability lists. Composition adds parser/type complexity for marginal expressiveness gain.

**Replacement pattern:**
```ash
-- Instead of: capability http = network + tls
-- Use: Define a role
role http_client {
    capabilities: [network, tls]
}

-- Workflows use the role
workflow api_client plays role(http_client) { ... }
```

### Decision 3: Simplify Constraint Syntax

**What:** Keep `@ { ... }` syntax but limit to one-level refinement at declaration site.

**Rationale:** Three-level refinement (definition → role → use) is powerful but complex. Start with declaration-site constraints only.

**Supported:**
```ash
capability file {
    permissions: { read: bool, write: bool }
}

workflow processor capabilities: [
    file @ { paths: ["/tmp/*"], read: true, write: false }
] { ... }
```

**Deferred:** Use-site constraint narrowing (can be added later without breaking change).

### Decision 4: No Implicit Role Syntax Leak

**What:** Do not expose `_default` role names in surface syntax.

**Rationale:** Implementation detail should not be user-visible.

**Instead:** Keep yield targeting explicit:
```ash
-- Keep: yield role(ai_assistant) Request { ... }
-- Omit: yield role(specialized_analyzer_default) Request { ... }
-- Omit: yield workflow(specialized_analyzer) Request { ... } (sugar)
```

**Procedure call pattern:** Use named roles as interfaces:
```ash
role analyzer { capabilities: [ml_model] }

workflow specialized_analyzer plays role(analyzer) { ... }

-- Caller yields to the role, not the workflow
workflow caller plays role(executor) {
    yield role(analyzer) Request { ... }  -- Routed to handler
}
```

### Decision 5: Unified `workflow` Keyword (Keep)

**What:** All workflows use `workflow` keyword; proxiness determined by `handles` clause.

**Rationale:** Already implemented (Phase 42). This aligns with brainstorm goal.

```ash
-- Internal workflow
workflow compute_hash { ... }

-- Proxy workflow (has 'handles' clause)
workflow llm_harness plays role(ai_agent) handles requests:agent_tasks { ... }
```

### Syntax Reduction Summary

| Brainstorm Proposal | Decision | Rationale |
|---------------------|----------|-----------|
| `capability X = A + B` | **Defer** | Use role inclusion for now; composition may return if needed |
| `capability X = A \| B` | **Defer** | Union types deferred; evaluate need after initial usage |
| `file @ { constraints }` at use site | **Defer** | Start with declaration-site only |
| `yield role(X_default)` | **Defer** | Avoid implementation leak; revisit if direct workflow calls needed |
| `yield workflow(X)` sugar | **Defer** | Use named roles as interfaces for now |
| `plays role(R)` | **Keep** | Explicit, clear |
| `capabilities: [...]` | **Keep** | Brevity for common case |
| `workflow` for all | **Keep** | Already implemented |

**Result:** Deferred complex features for Phase 46+; implementing foundational `plays role` and `capabilities:` in Phase 46.

---

## Part 3: Roadmap

### Phase 44: Audit Convergence (REQUIRED)

**Goal:** Fix all audit findings. This is blocking work.

**Duration:** 4-6 weeks
**Dependencies:** None
**Status:** Ready to start

#### 44.1: Critical Runtime Fixes

| Task | Description | Est. Hours | File Locations |
|------|-------------|------------|----------------|
| TASK-240 | Implement `Workflow::Oblige` execution | 6 | `ash-interp/src/execute.rs:856` |
| TASK-241 | Implement `Workflow::CheckObligation` execution | 6 | `ash-interp/src/execute.rs:867` |
| TASK-242 | Replace `Yield` placeholder lowering | 8 | `ash-parser/src/lower.rs:388` |
| TASK-243 | Implement `YIELD` runtime execution | 10 | `ash-interp/src/execute.rs:874` |
| TASK-244 | Implement `PROXY_RESUME` runtime | 8 | `ash-interp/src/execute.rs` |

#### 44.2: Safety and API Hardening

| Task | Description | Est. Hours | File Locations |
|------|-------------|------------|----------------|
| TASK-245 | Redesign `SmtContext` threading | 8 | `ash-typeck/src/smt.rs:118` |
| TASK-246 | Make `EngineBuilder` methods real | 10 | `ash-engine/src/lib.rs:306,318,327` |
| TASK-247 | Implement stub providers | 12 | `ash-engine/src/providers.rs` |
| TASK-248 | Fix role obligation discharge | 6 | `ash-interp/src/role_context.rs:86` |

#### 44.3: Quality Gate Remediation

| Task | Description | Est. Hours | File Locations |
|------|-------------|------------|----------------|
| TASK-249 | Fix clippy warnings | 4 | workspace |
| TASK-250 | Run cargo fmt | 2 | workspace |
| TASK-251 | Fix rustdoc warnings | 6 | workspace |
| TASK-252 | Fix `unexpected_cfgs` | 2 | `ash-typeck/Cargo.toml` |

#### 44.4: Numeric and CLI Fixes

| Task | Description | Est. Hours | File Locations |
|------|-------------|------------|----------------|
| TASK-253 | Fix float handling | 6 | `ash-parser/src/lower.rs:531`, `ash-cli/src/commands/run.rs:90` |
| TASK-254 | Implement trace flags or remove | 4 | `ash-cli/src/commands/trace.rs` |
| TASK-255 | Update stale documentation | 8 | `README.md`, `docs/API.md`, `docs/spec/README.md` |

**Phase 44 Deliverable:** All audit issues resolved, quality gates passing.

---

### Phase 45: Syntax Reduction Specification

**Goal:** Produce canonicalized reduced syntax specification.

**Duration:** 1 week
**Dependencies:** Phase 44 complete
**Status:** Blocked pending Phase 44

| Task | Description | Est. Hours | Output |
|------|-------------|------------|--------|
| TASK-256 | Write reduced syntax spec | 8 | `docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md` |
| TASK-257 | Create syntax decision record | 4 | `docs/design/DESIGN-014-SYNTAX-REDUCTION.md` |
| TASK-258 | Update SPEC-017 with constraint syntax | 4 | `docs/spec/SPEC-017-CAPABILITY.md` |

**Phase 45 Deliverable:** Approved reduced syntax specification ready for implementation.

---

### Phase 46: Unified Capability-Role Implementation

**Goal:** Implement reduced syntax features.

**Duration:** 6-8 weeks
**Dependencies:** Phase 45 complete
**Status:** Blocked pending syntax approval

#### 46.1: Parser Extensions

| Task | Description | Est. Hours | Spec |
|------|-------------|------------|------|
| TASK-259 | Parse `plays role(R)` clause | 6 | SPEC-019 |
| TASK-260 | Parse `capabilities: [...]` with `@` constraints | 10 | SPEC-024 |
| TASK-261 | Lower implicit role generation | 8 | SPEC-024 |

#### 46.2: Type System Integration

| Task | Description | Est. Hours | Spec |
|------|-------------|------------|------|
| TASK-262 | Type check role inclusion | 8 | SPEC-019 |
| TASK-263 | Validate capability constraints | 10 | SPEC-017 |
| TASK-264 | Compose effective capability sets | 8 | SPEC-024 |

#### 46.3: Runtime Integration

| Task | Description | Est. Hours | Spec |
|------|-------------|------------|------|
| TASK-265 | Runtime role resolution | 8 | SPEC-019 |
| TASK-266 | Capability constraint enforcement | 10 | SPEC-017 |
| TASK-267 | Yield routing by role | 10 | SPEC-023 |

#### 46.4: Agent Harness (Optional Sub-phase)

| Task | Description | Est. Hours | Spec |
|------|-------------|------------|------|
| TASK-268 | Define `agent_harness` capability | 4 | Design doc |
| TASK-269 | Implement harness workflow pattern | 12 | Design doc |
| TASK-270 | MCP capability provider | 10 | Design doc |

**Phase 46 Deliverable:** Unified capability-role-workflow system with reduced syntax.

---

## Part 4: Dependency Graph

```
Phase 44 (Audit Convergence)
    │
    ├──→ All audit issues resolved
    │
    └──→ Phase 45 (Syntax Reduction)
            │
            ├──→ Syntax spec approved
            │
            └──→ Phase 46 (Implementation)
                    │
                    ├──→ Parser extensions
                    ├──→ Type system
                    ├──→ Runtime
                    └──→ Optional: Agent harness
```

---

## Part 5: Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Phase 44 takes longer than estimated | High | Tasks are well-scoped; can parallelize across team |
| Syntax reduction rejected | Medium | Document alternatives in Phase 45 deliverable |
| Phase 46 conflicts with existing code | Low | Builds on existing role/proxy implementation |
| Agent harness scope creep | Medium | Marked optional; gate on PoC value |

---

## Decisions

| Question | Decision |
|----------|----------|
| Constraint solving | **Z3 required** for constraints, obligations/responsibilities, and policies. No viable alternative for equivalent guarantees. |
| Agent harness priority | **DEFER (not OMIT)** - Required as concrete real-world use-case for Ash applicability proof. Marked as Phase 46.4. |

## Open Questions

1. **Dynamic role assignment:** Are roles static at spawn time, or can they change? (Brainstorm Q1)
2. **Capability revocation:** How do we handle runtime revocation of base capabilities?
3. **Syntax details:** Review DEFER vs OMIT decisions for `+` operators and constraint refinement levels

---

## Appendix A: Audit-to-Task Mapping

| Audit Item | Priority | Task |
|------------|----------|------|
| SPEC-022 obligations not executable | Critical | TASK-240, TASK-241 |
| Proxy/yield placeholders | Critical | TASK-242, TASK-243, TASK-244 |
| SmtContext unsafe Send/Sync | High | TASK-245 |
| EngineBuilder no-ops | High | TASK-246 |
| Stub providers | High | TASK-247 |
| Role obligation discharge | High | TASK-248 |
| Clippy warnings | High | TASK-249 |
| Fmt failures | High | TASK-250 |
| Doc warnings | High | TASK-251 |
| unexpected_cfgs | Medium | TASK-252 |
| Float truncation | Medium | TASK-253 |
| Trace flags decorative | Medium | TASK-254 |
| Stale docs | Medium | TASK-255 |

---

## Appendix B: Brainstorm-to-Task Mapping

| Brainstorm Section | Reduced Syntax | Task |
|--------------------|----------------|------|
| 2. Capability-Role Model | Keep `capabilities:`, `plays role` | TASK-259, TASK-260 |
| 3. Yield as procedure | Omit `yield workflow()`, use roles | TASK-267 |
| 4. Unified workflow | Already implemented | (none) |
| 5. Constraint refinement | One-level only | TASK-261, TASK-263 |
| 6. Agent harness | Optional sub-phase | TASK-268-270 |
| 7. Complete example | Valid with reduced syntax | (validation) |

---

*Document Version: 1.0*
*Synthesized from: codex-comprehensive-review.md, unified-capability-role-workflow-synthesis.md*
*Status: Draft pending user review*

---

## Task Generation Notes

**Verification Requirement:** All task files generated from this roadmap MUST include codex sub-agent verification steps as per updated `writing-plans` skill (v1.2.0):
- End-of-task codex verification before marking complete
- End-of-phase codex audit before phase closeout
- Verification checklist: tests, clippy, fmt, doc, spec compliance
