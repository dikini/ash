# DESIGN-014: Syntax Reduction Decisions

**Status:** DECIDED  
**Date:** 2026-03-26  
**Decided by:** Language Design Team  
**Scope:** Ash Language 0.6.0+ (Phase 45-46)  
**Related:** SPEC-024, PHASE-44-46-ROADMAP

---

## Context

The brainstorm document (`unified-capability-role-workflow-synthesis.md`) introduced significant new syntax for capability-role-workflow integration:

- **Capability composition operators**: `capability http = network + tls`
- **Three-level constraint refinement**: definition → role → use site
- **Implicit role syntax exposure**: `yield role(specialized_analyzer_default)`
- **Multiple yield target syntaxes**: `yield workflow(X)` sugar
- **Dual capability assignment**: `plays role(R)` vs `capabilities: [...]`

This represented a substantial expansion of surface syntax with overlapping expressiveness and potential for confusion. Before implementation, we conducted a syntax reduction review to minimize surface complexity while preserving essential functionality.

### Syntax Proliferation Analysis

| Proposed Syntax | Overlap with Existing | Complexity Impact |
|-----------------|----------------------|-------------------|
| `capability X = A + B` | Role capability bundles | High (parser, type system) |
| `capability X = A \| B` | Role selection | Medium (union types) |
| `@ { ... }` at use site | Declaration-site constraints | High (refinement types) |
| `yield role(X_default)` | Direct `yield role(X)` | Low (naming leak) |
| `yield workflow(X)` | `yield role(X)` | Low (sugar complexity) |

**Risk:** Without reduction, we would have multiple ways to express the same concept, increasing cognitive load and implementation complexity.

---

## Decision

Apply syntax reduction to minimize surface complexity for Phase 46 implementation.

### Kept Syntax

| Syntax | Use Case |
|--------|----------|
| `plays role(R)` | Explicit, reusable role assignment |
| `capabilities: [...]` | Brevity for common case (desugars to implicit role) |
| `capability @ { constraints }` | One-level constraint refinement at declaration site |

### Deferred Syntax

| Syntax | Deferred To | Rationale |
|--------|-------------|-----------|
| `capability X = A + B` | Phase 47+ | Achievable via role inclusion |
| `capability X = A \| B` | Phase 47+ | Use case unclear |
| `@ { ... }` at use site | Phase 47+ | Start simpler, add if needed |
| `yield role(X_default)` | Never | Implementation leak |
| `yield workflow(X)` | Phase 47+ | Use roles as interfaces |

---

## Consequences

### Positive Consequences

1. **Smaller Implementation Surface**
   - Fewer parser productions
   - Simpler type checking rules
   - Reduced test matrix

2. **Clearer User Mental Model**
   - One way to assign capabilities: `plays role(R)`
   - One way to declare capabilities: `capabilities: [...]`
   - No confusion between composition operators and role inclusion

3. **Easier Evolution**
   - Deferred features can be added without breaking changes
   - Use-site refinement is a narrowing-only addition
   - Composition operators can be added as sugar later

4. **Faster Time to Value**
   - Phase 46 can focus on core functionality
   - Less time on edge cases and interactions
   - Earlier user feedback on fundamental design

### Negative Consequences

1. **Some Expressiveness Deferred**
   - Cannot compose capabilities inline: `capability http = network + tls`
   - Must define a role instead (more verbose for one-offs)
   - No use-site constraint narrowing

2. **Potential Rework**
   - If users demand deferred features, we must implement them later
   - Migration from role-based composition to operator-based is breaking

3. **Learning Curve for Role-First Design**
   - Users may expect inline composition from other languages
   - Need documentation explaining "define a role" pattern

---

## Alternatives Considered

### Alternative 1: Implement All Syntax (Rejected)

**Proposal:** Implement all brainstorm syntax without reduction.

**Pros:**
- Maximum expressiveness from day one
- No rework risk

**Cons:**
- 3-4x implementation effort
- Multiple ways to do the same thing
- Higher cognitive load for users
- More edge cases and bugs

**Rejection Reason:** Too complex for initial release; premature optimization of expressiveness.

---

### Alternative 2: Keep Composition, Defer Constraints (Rejected)

**Proposal:** Keep `+` and `|` operators but defer `@` refinement.

**Pros:**
- Familiar composition syntax from other languages

**Cons:**
- `@` refinement is more fundamental (needed for security)
- `+` is sugar for roles anyway
- Still adds parser/type complexity

**Rejection Reason:** `@` is more important than `+`; keeping `+` adds complexity without proportional value.

---

### Alternative 3: Different Reduction (Keep `yield workflow()`) (Rejected)

**Proposal:** Keep `yield workflow(X)` sugar, defer `@` refinement.

**Pros:**
- Direct workflow calls are intuitive

**Cons:**
- Exposes workflow naming as API (fragile)
- `yield role(X)` is the governance pattern we want to encourage
- `@` refinement is more fundamental for security

**Rejection Reason:** Encourages wrong pattern (direct coupling); role-based routing is the design goal.

---

### Alternative 4: OMIT Instead of DEFER (Partially Rejected)

**Proposal:** Mark deferred features as OMIT (never implement) instead of DEFER.

**Applied To:**
- `yield role(X_default)` → **OMIT** (implementation leak)

**Deferred Instead:**
- `+`, `|`, `@` use-site, `yield workflow()` → **DEFER** (may return if use cases demand)

**Rationale:** Some features are clearly wrong (`_default` leak). Others may be genuinely useful if usage patterns emerge. Keeping DEFER leaves options open without commitment.

---

## Migration Path

If deferred features are needed later:

### Adding Capability Composition

```ash
-- Phase 46: Define a role
role http_client {
    capabilities: [network, tls]
}

-- Phase 47+: Inline composition (sugar)
capability http = network + tls  -- Desugars to role definition
```

This is a pure syntactic addition - desugars to existing constructs.

### Adding Use-Site Refinement

```ash
-- Phase 46: Declaration-site only
workflow w capabilities: [file @ { paths: ["/tmp/*"] }] { ... }

-- Phase 47+: Use-site narrowing (non-breaking)
workflow w capabilities: [file] {
    act file.read with { path: "/tmp/data" } @ { path: "/tmp/*" }  -- Narrowing
}
```

Use-site refinement is narrowing-only, so it's a safe addition.

---

## Related Documents

- [PHASE-44-46-ROADMAP](../plan/PHASE-44-46-ROADMAP.md) - Implementation roadmap with syntax reduction gate
- [SPEC-024: Capability-Role-Workflow Syntax (Reduced)](../spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md) - Formal specification of reduced syntax
- [unified-capability-role-workflow-synthesis.md](../../todo-examples/definitions/hermes-conversations/unified-capability-role-workflow-synthesis.md) - Original brainstorm document

---

## Open Questions

1. **Use-site refinement priority**: If added in Phase 47, should it be validation-only or affect capability availability?
2. **Composition sugar complexity**: If `+` is added later, should it support arbitrary expressions or just capability names?
3. **Union types**: Is `A | B` ever needed, or do roles subsume all use cases?

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-26 | DEFER `+` and `\|` operators | Achievable via roles; adds complexity |
| 2026-03-26 | DEFER use-site `@` refinement | Start simpler; can add narrowing later |
| 2026-03-26 | OMIT `yield role(X_default)` | Implementation leak; never expose |
| 2026-03-26 | OMIT `yield workflow(X)` sugar | Use roles as interfaces; avoid fragile coupling |
| 2026-03-26 | KEEP `plays role(R)` | Explicit, clear, enables reuse |
| 2026-03-26 | KEEP `capabilities: [...]` | Brevity for common case |
| 2026-03-26 | KEEP `@` at declaration site | Fundamental for security |

---

*Document Version: 1.0*  
*Status: DECIDED*
