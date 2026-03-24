# Phases 38-43 Roadmap: Governance and Collaboration

## Overview

This roadmap addresses the gaps identified between the idea space documents and current implementation. The focus is on:

1. **Capability definitions** in source files (Phases 38-39)
2. **Role runtime semantics** for authority and obligation enforcement (Phases 39-40)
3. **Proxy workflows** for human-AI collaboration (Phases 41-42)

No release is currently planned - this is a development stream for future capabilities.

---

## Phase 38: Capability Definition Specification

**Task:** TASK-233
**Type:** Specification
**Status:** ✅ Ready
**Dependencies:** None

**Goal:** Revise SPEC-017 to add capability definition parsing requirements.

**Deliverable:** SPEC-017 revision with complete BNF grammar for capability definitions.

---

## Phase 39: Capability Definition Implementation

**Task:** TASK-234
**Type:** Implementation
**Status:** ⏳ Blocked on TASK-233
**Dependencies:** TASK-233
**Estimated:** 2-3 weeks

**Goal:** Implement parser support for capability definitions in `.ash` files.

**Components:**
- Surface AST: `CapabilityDef`, `EffectType`
- Parser: `capability_def` in `parse_module.rs`
- Lowering: Surface → Core
- Tests: Parse, lower, integrate

---

## Phase 40: Role Runtime Semantics Specification

**Task:** TASK-235
**Type:** Specification
**Status:** ✅ Ready
**Dependencies:** None (can proceed in parallel)

**Goal:** Create SPEC-019 defining how role `authority` and `obligations` are enforced.

**Key Decisions:**
- Authority check before capability access
- Role obligation tracking separate from local obligations
- Role assignment at spawn time

---

## Phase 41: Role Runtime Implementation

**Task:** TASK-236
**Type:** Implementation
**Status:** ⏳ Blocked on TASK-235
**Dependencies:** TASK-235 (SPEC-019)
**Estimated:** 3-4 weeks

**Goal:** Implement runtime enforcement of role authority and obligations.

**Components:**
- `RoleContext` with authority/obligations tracking
- Authority checks in capability policy evaluator
- Role obligation verification at workflow completion
- Integration with existing obligation system

---

## Decision Point: Obligation Syntax

**Task:** DECISION-237
**Type:** Decision
**Status:** ✅ Ready for discussion
**Dependencies:** None

**Question:** Continue with local obligations only, or add role-bound syntax?

**Recommendation:** Option C - Both (local + role-bound)
- Phase 1: Keep local (status quo)
- Phase 2: Add role-bound after proxy workflows

This decision should be made before starting proxy workflow specification.

---

## Phase 42: Proxy Workflows Specification

**Task:** TASK-238
**Type:** Specification
**Status:** ✅ Ready
**Dependencies:** DECISION-237

**Goal:** Create SPEC-023 defining proxy workflows for external persona representation.

**Key Concepts:**
- `proxy` keyword vs `workflow`
- `handles role(name)` declaration
- Role-to-proxy registry
- `yield role(name)` routing
- Quorum/consensus patterns

---

## Phase 43: Proxy Workflows Implementation

**Task:** TASK-239
**Type:** Implementation
**Status:** ⏳ Blocked on TASK-238
**Dependencies:** TASK-238 (SPEC-023), TASK-236 recommended
**Estimated:** 6-8 weeks

**Goal:** Implement proxy workflow runtime with message routing.

**Components:**
- `ProxyDef` AST and parser
- Role-to-proxy registry
- `yield` routing to proxies
- `resume` correlation handling
- Quorum pattern support

---

## Dependency Graph

```
TASK-233 (SPEC-017 rev)
    │
    ▼
TASK-234 (Capability parser) ────────┐
    │                                │
    │                                ▼
    │                           [Capability system complete]
    │                                │
TASK-235 (SPEC-019) ────────────────┤
    │                                │
    ▼                                │
TASK-236 (Role runtime) ────────────┤
    │                                │
    │                                ▼
    │                           [Role system complete]
    │                                │
DECISION-237 (Obligation syntax)     │
    │                                │
    ▼                                │
TASK-238 (SPEC-023) ────────────────┤
    │                                │
    ▼                                │
TASK-239 (Proxy workflows) ◄────────┘
    │
    ▼
[Human-AI collaboration enabled]
```

## Parallel Work Streams

### Stream A: Capabilities
- TASK-233 → TASK-234
- Can start immediately

### Stream B: Roles
- TASK-235 → TASK-236
- Can start immediately
- Informed by DECISION-237

### Stream C: Proxy Workflows
- DECISION-237 → TASK-238 → TASK-239
- Should wait for Stream B (roles) to complete or progress significantly

## Task Files

| Task | File |
|------|------|
| TASK-233 | `docs/plan/tasks/TASK-233-SPEC-017-CAPABILITY-PARSING.md` |
| TASK-234 | `docs/plan/tasks/TASK-234-CAPABILITY-PARSER-IMPL.md` |
| TASK-235 | `docs/plan/tasks/TASK-235-SPEC-019-ROLE-SEMANTICS.md` |
| TASK-236 | `docs/plan/tasks/TASK-236-ROLE-RUNTIME-IMPL.md` |
| DECISION-237 | `docs/plan/tasks/TASK-237-OBLIGATION-SYNTAX-DECISION.md` |
| TASK-238 | `docs/plan/tasks/TASK-238-SPEC-023-PROXY-WORKFLOWS.md` |
| TASK-239 | `docs/plan/tasks/TASK-239-PROXY-WORKFLOW-IMPL.md` |

## Notes

- No release date is planned for these features
- Tasks can be worked on semi-independently within dependency constraints
- Specifications (TASK-233, TASK-235, TASK-238) can all proceed in parallel
- DECISION-237 should be made before TASK-238 begins
- TASK-234 and TASK-236 can proceed in parallel after their specs complete
