# TASK-237: DECISION - Obligation Syntax (Local vs Role-Bound)

## Status: ✅ Complete - DECIDED

## Decision

**Option C: Support both local and role-bound obligations**

- Local obligations (SPEC-022): `oblige name;` / `check name;`
- Role-bound obligations: `check role_name.obligation_name;`

**Rationale:**
- Backward compatible with existing SPEC-022 code
- Enables progressive enhancement from simple to complex governance
- Proxy workflows can use local obligations initially, migrate to role-bound later

**Implementation:**
- Phase 1 (now): Local obligations (already implemented in SPEC-022)
- Phase 2 (after proxy workflows): Add role-bound obligation syntax

**Decision Record:** See `docs/decisions/DECISION-237-OBLIGATION-SYNTAX.md`

## Description

Make a decision on obligation syntax: continue with SPEC-022's workflow-local obligations only, add role-bound obligations, or support both.

## Background

SPEC-022 (implemented in Phase 37) defines workflow-local obligations:

```ash
workflow example {
    oblige audit_trail;           -- Create obligation
    let ok = check audit_trail;   -- Discharge obligation
}
```

The idea space (`todo-examples/definitions/obligations.md`) describes role-bound obligations:

```ash
role reviewer {
    obligations: [check_security]
}

workflow review_task {
    check reviewer.check_security;  -- Role-bound obligation
}
```

## Options

### Option A: Local Obligations Only (Status Quo)

**Syntax:** `oblige name;` / `check name;`

**Pros:**
- Already implemented (SPEC-022)
- Simpler mental model
- Sufficient for most use cases
- Proxy workflows can simulate role-bound checks

**Cons:**
- Less explicit connection to role duties
- No compile-time check that role obligations are covered

### Option B: Role-Bound Obligations Only

**Syntax:** `check role_name.obligation_name;`

**Pros:**
- Explicit connection to role definitions
- Clearer governance model

**Cons:**
- Breaks existing SPEC-022 code
- Requires role context for all obligation checks
- More complex syntax

### Option C: Both (Recommended)

**Syntax:** 
- `oblige name;` / `check name;` (local)
- `check role_name.obligation_name;` (role-bound)

**Pros:**
- Backward compatible with SPEC-022
- Richer governance when needed
- Local for simple cases, role-bound for complex governance

**Cons:**
- Two ways to do similar things (complexity)
- Requires both local and role obligation tracking

## Decision Factors

| Factor | Local (A) | Role-Bound (B) | Both (C) |
|--------|-----------|----------------|----------|
| Implementation effort | ✅ Done | Medium | Higher |
| Backward compatibility | ✅ Yes | ❌ Breaking | ✅ Yes |
| Governance expressiveness | Limited | High | High |
| Complexity for users | Low | Medium | Medium |
| Proxy workflow support | Via local | Native | Native |

## Recommendation

**Option C (Both)** with phased implementation:

1. **Phase 1 (now):** Keep local obligations (status quo)
2. **Phase 2 (after proxy workflows):** Add role-bound syntax

Rationale:
- SPEC-022 is done and working
- Proxy workflows (TASK-238/239) can simulate role-bound obligations via local ones
- Once proxy workflows exist, the value of native role-bound obligations becomes clearer
- Can add role-bound syntax without breaking existing code

## Acceptance Criteria

Decision documented with:
- [ ] Selected option (A, B, or C)
- [ ] Rationale for selection
- [ ] Migration path if breaking changes needed
- [ ] Timeline for implementation (if not Option A)

## Dependencies

- None (decision-only task)
- Informs TASK-235/SPEC-019 (role semantics)

## Related Documents

- `docs/spec/SPEC-022-WORKFLOW-TYPING.md` (local obligations)
- `todo-examples/definitions/obligations.md` (idea space)
- `todo-examples/definitions/roles.md` (role concept)

## Notes

This decision can be made independently of TASK-233 through TASK-236. The decision should be recorded before starting TASK-238 (proxy workflows) as it affects proxy workflow design.
