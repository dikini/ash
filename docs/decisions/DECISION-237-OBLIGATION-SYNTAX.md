# DECISION-237: Obligation Syntax

## Status: **DECIDED**

**Decision:** Support **both** local and role-bound obligations (Option C)  
**Date:** 2026-03-24  
**Decided by:** dikini with Hermes  
**Scope:** Ash Language 0.6.0+

---

## Context

SPEC-022 (Phase 37) implemented workflow-local obligations:

```ash
workflow example {
    oblige audit_trail;           -- Create local obligation
    let ok = check audit_trail;   -- Discharge local obligation
}
```

SPEC-019 defines roles with obligations:

```ash
role reviewer {
    obligations: [check_security]
}
```

The question: How should role obligations be discharged? Should we:
- Keep only local obligations (status quo)
- Add role-bound obligation syntax
- Support both

---

## Options Analysis

### Option A: Local Obligations Only

**Syntax:**
```ash
workflow review_task {
    oblige check_security;        -- Must explicitly create
    check check_security;         -- Discharge
}
```

**Pros:**
- ✅ Already implemented (SPEC-022)
- ✅ Simple mental model
- ✅ No new syntax to learn
- ✅ Proxy workflows can simulate role-bound checks via naming convention

**Cons:**
- ❌ No compile-time verification that role obligations are covered
- ❌ Less explicit connection between roles and their obligations
- ❌ Must manually track which obligations correspond to which roles

---

### Option B: Role-Bound Obligations Only

**Syntax:**
```ash
workflow review_task {
    check reviewer.check_security;  -- Discharge role obligation
}
```

**Pros:**
- ✅ Explicit connection to role definitions
- ✅ Clear governance model

**Cons:**
- ❌ Breaks existing SPEC-022 code
- ❌ Requires role context for all obligation checks
- ❌ More verbose for simple cases
- ❌ Removes flexibility of workflow-local obligations

---

### Option C: Both (SELECTED)

**Syntax:**
```ash
-- Local obligations (existing)
workflow simple_task {
    oblige deadline_met;
    check deadline_met;
}

-- Role-bound obligations (new)
workflow review_task runs_as reviewer {
    check role.check_security;    -- Discharge role obligation
}
```

**Pros:**
- ✅ Backward compatible with SPEC-022
- ✅ Local for simple cases, role-bound for complex governance
- ✅ Enables gradual adoption
- ✅ Richer governance when needed

**Cons:**
- ⚠️ Two ways to do similar things (complexity)
- ⚠️ Requires both local and role obligation tracking

---

## Decision

**Adopt Option C: Support both local and role-bound obligations.**

### Phased Implementation

**Phase 1 (Now):** Local obligations only (status quo)
- SPEC-022 implementation stands
- No changes needed

**Phase 2 (After proxy workflows):** Add role-bound syntax
- Syntax: `check role.obligation_name`
- Requires role context from SPEC-019
- Enables role obligation discharge

### Syntax Details

**Local obligations (existing):**
```ash
oblige obligation_name;       -- Create
check obligation_name;        -- Discharge (returns bool)
```

**Role-bound obligations (new, in Phase 2):**
```ash
check role.obligation_name;   -- Discharge role obligation
```

**Disambiguation:**
- If `obligation_name` exists locally → discharge local
- If `role.obligation_name` specified → discharge role obligation
- Role obligations cannot be created in workflow (must be defined in role)

### Implementation Notes

```rust
pub enum ObligationCheck {
    Local(Name),           -- check obligation_name
    RoleBound(Name, Name), -- check role.obligation_name
}
```

**Type checking:**
- Local obligations tracked in `ObligationSet` (SPEC-022)
- Role obligations tracked in `RoleContext` (SPEC-019)
- Both checked at workflow completion

---

## Consequences

### Positive

1. **Backward compatible:** Existing SPEC-022 code continues to work
2. **Progressive enhancement:** Can add role-bound obligations later
3. **Flexible:** Simple workflows stay simple, complex workflows get governance
4. **Clear migration path:** Local → role-bound when governance needs increase

### Negative

1. **Two syntaxes to document and teach**
2. **More complex obligation tracking** (both local and role contexts)
3. **Potential confusion:** When to use local vs role-bound?

### Mitigations

- **Documentation:** Clear guidance on when to use each
- **Linting:** Warn when local obligation shadows role obligation
- **Best practices:** Recommend local for single-workflow, role-bound for multi-workflow governance

---

## Related Work

- SPEC-022: Workflow Typing with Constraints (local obligations)
- SPEC-019: Role Runtime Semantics (role obligations)
- TASK-238: Proxy Workflows (motivates role-bound obligations)

---

## Open Questions (Non-blocking)

1. **Should we deprecate local obligations eventually?**  
   No, they serve different use cases.

2. **Can a workflow have both local and role-bound obligations?**  
   Yes, both are tracked independently.

3. **What if local and role obligations have the same name?**  
   `check name` refers to local; `check role.name` refers to role. Shadowing warning recommended.

---

## References

- `docs/spec/SPEC-022-WORKFLOW-TYPING.md`
- `docs/spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md`
- `todo-examples/definitions/obligations.md`
- `todo-examples/definitions/roles.md`
