# TASK-257: Write Reduced Syntax Specification

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Produce canonical SPEC-024 documenting reduced capability-role-workflow syntax.

**Spec Reference:** PHASE-44-46-ROADMAP.md Section 2 (Syntax Reduction Decisions)

**File Locations:**
- Create: `docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md`

---

## Background

The brainstorm document introduced significant new syntax. Phase 45 reduces this to a minimal viable set. This spec documents the reduced syntax formally.

**Kept:**
- `plays role(R)` - explicit role inclusion
- `capabilities: [...]` - direct capability declaration (desugars to implicit role)
- `capability @ { constraints }` - declaration-site constraints only

**Deferred:**
- Capability composition operators (`+`, `|`)
- Use-site constraint refinement
- Implicit role syntax leak (`_default`)
- `yield workflow()` sugar

---

## Step 1: Write Specification

Create `docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md`:

```markdown
# SPEC-024: Capability-Role-Workflow Syntax (Reduced)

**Status:** Canonical
**Replaces/Extends:** SPEC-017, SPEC-019, SPEC-022, SPEC-023

## 1. Overview

This specification defines the reduced surface syntax for:
- Capability definitions with constraints
- Role definitions with capability bundles
- Workflow definitions with role inclusion and direct capability declaration

## 2. Capability Definitions

```ebnf
capability-def ::= "capability" ident capability-body
capability-body ::= "{" field-def* "}"
field-def ::= ident ":" type-expr
```

Example:
```ash
capability file {
    effect: Operational,
    permissions: { read: bool, write: bool }
}
```

## 3. Role Definitions

```ebnf
role-def ::= "role" ident "{" role-body "}"
role-body ::= capability-decl*
capability-decl ::= "capabilities" ":" "[" capability-ref+"]"
capability-ref ::= ident constraint-refinement?
constraint-refinement ::= "@" "{" field-value* "}"
```

Example:
```ash
role ai_agent {
    capabilities: [
        file @ { paths: ["/tmp/*"], read: true, write: false }
    ]
}
```

## 4. Workflow Definitions

```ebnf
workflow-def ::= "workflow" ident workflow-header workflow-body
workflow-header ::= role-inclusion* capability-decl?
role-inclusion ::= "plays" "role" "(" ident ")"
workflow-body ::= "{" workflow-stmt* "}"
```

Example:
```ash
workflow processor
    plays role(ai_agent)
    capabilities: [network @ { hosts: ["*.example.com"] }]
{
    -- workflow body
}
```

## 5. Lowering Semantics

### 5.1 Implicit Default Role

The syntax:
```ash
workflow X capabilities: [C1, C2] { ... }
```

Lowers to:
```ash
role X_default { capabilities: [C1, C2] }
workflow X plays role(X_default) { ... }
```

The implicit role name is not user-visible.

### 5.2 Capability Composition

For Phase 46, capability composition is achieved through role inclusion:

```ash
-- Instead of: capability http = network + tls
role http_client {
    capabilities: [network, tls]
}

workflow api plays role(http_client) { ... }
```

## 6. Deferred Features

The following are deferred to future phases:

| Feature | Syntax | Rationale |
|---------|--------|-----------|
| Capability composition | `capability X = A + B` | Achievable via roles |
| Capability union | `capability X = A \| B` | Use case unclear |
| Use-site refinement | `cap "file" @ { path: "/x" }` | Start with declaration-site |
| Yield to workflow | `yield workflow(X)` | Use named roles |

## 7. References

- SPEC-017: Capability Integration
- SPEC-019: Role Semantics
- SPEC-022: Workflow Typing
- SPEC-023: Proxy Workflows
- DESIGN-014: Syntax Reduction Decisions
```

---

## Step 2: Create Design Decision Record

Create `docs/design/DESIGN-014-SYNTAX-REDUCTION.md`:

```markdown
# DESIGN-014: Syntax Reduction Decisions

## Context

The brainstorm document introduced significant new syntax for capability-role-workflow integration:
- Capability composition operators (`+`, `|`)
- Three-level constraint refinement
- Implicit role syntax exposure
- Multiple yield target syntaxes

## Decision

Apply syntax reduction to minimize surface complexity for Phase 46.

## Consequences

### Positive
- Smaller implementation surface
- Fewer ways to do the same thing
- Clearer migration path

### Negative
- Some expressiveness deferred
- May need to reintroduce features if use cases demand

## Alternatives Considered

1. **Implement all syntax** - Rejected: too complex for initial release
2. **Different reduction** - Keep `+`, defer `@` - Rejected: `@` more fundamental

## References

- PHASE-44-46-ROADMAP.md
- todo-examples/definitions/hermes-conversations/unified-capability-role-workflow-synthesis.md
```

---

## Step 3: Run Doc Tests

```bash
cargo doc --package ash-parser --no-deps
```

---

## Step 4: Commit

```bash
git add docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md
git add docs/design/DESIGN-014-SYNTAX-REDUCTION.md
git commit -m "docs: add SPEC-024 reduced capability-role syntax (TASK-257)

- Formal grammar for capability, role, workflow definitions
- Lowering semantics for implicit default role
- List of deferred features with rationale
- Design decision record explaining reduction
- References to related specs"
```

---

## Step 5: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-257 specification"
  context: |
    Files to verify:
    - docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md
    - docs/design/DESIGN-014-SYNTAX-REDUCTION.md
    
    Requirements:
    1. Grammar is unambiguous
    2. Examples compile (if parser exists)
    3. Deferred features clearly marked
    4. References to related specs valid
    5. Consistent with PHASE-44-46-ROADMAP decisions
    
    Run and report:
    1. Read SPEC-024 for clarity
    2. Check grammar consistency
    3. Verify all referenced specs exist
    4. Check consistency with roadmap
    5. cargo doc --no-deps (verify no errors)
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] SPEC-024 written
- [ ] Grammar defined
- [ ] Examples included
- [ ] Lowering semantics documented
- [ ] Deferred features listed
- [ ] DESIGN-014 written
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** Phase 44 complete
**Blocks:** TASK-258 (SPEC-017 update)
