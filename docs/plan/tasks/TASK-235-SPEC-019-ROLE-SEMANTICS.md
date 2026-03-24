# TASK-235: SPEC-019 - Role Runtime Semantics

## Status: Ready for Development

## Description

Create a new specification (SPEC-019) defining how role `authority` and `obligations` fields are enforced at runtime. Currently, the runtime has only a "lightweight identity wrapper" for roles - this spec defines full runtime semantics.

## Background

The `Role` struct in `ast.rs` has:
- `name`: Role identifier
- `authority`: List of capabilities the role can exercise
- `obligations`: List of obligations the role must fulfill

However, the runtime (`ash-interp/src/capability_policy.rs`) only uses `Role` as an identity for policy evaluation, not for enforcing authority/obligations.

The idea space document `todo-examples/definitions/roles.md` describes roles as:
> "The anchor point that balances Authority and Obligation"
> - *Capability (Authority)*: What the role can do
> - *Obligation (Duty)*: What the role must achieve

## Requirements

### 1. Authority Enforcement

Specify how `role.authority` restricts capability access:

```ash
role developer {
    authority: [git_read, git_write, test_run]
}

-- This workflow runs with developer role
workflow code_task {
    act git_read;     -- Allowed (in authority)
    act git_write;    -- Allowed (in authority)
    act deploy_prod;  -- ERROR: not in developer authority
}
```

Authority check semantics:
- Before capability invocation: is capability in role's authority list?
- Runtime error if not authorized
- Authority checking can be disabled in development mode

### 2. Obligation Enforcement

Specify how `role.obligations` gates workflow progression:

```ash
role reviewer {
    obligations: [check_tests, check_security]
}

workflow review_task {
    -- Before completion: have all role obligations been discharged?
    -- Workflow cannot complete (ret/done) until obligations checked
}
```

Obligation check semantics:
- Role obligations are separate from workflow-local obligations (SPEC-022)
- Role obligations checked via `check role.obligation_name` syntax
- Workflow completion blocked until role obligations discharged

### 3. Role Assignment

Specify how roles are assigned to workflow instances:

```ash
-- At spawn time
spawn code_task {
    role: developer,  -- Assign role
    init: { ... }
}

-- Or default role from context
```

Role assignment options:
- Explicit at spawn
- Inherited from parent
- Default role for workflow type

### 4. Runtime Role Context

Define `RoleContext` structure:

```rust
pub struct RoleContext {
    pub active_role: Role,
    pub discharged_obligations: HashSet<String>,
    pub authority_cache: HashSet<Capability>,
}
```

### 5. Integration with Existing Systems

- **Capability policy evaluator**: Add authority check before policy evaluation
- **Obligation tracker**: Merge role obligations with workflow-local obligations
- **Audit trail**: Log role checks and authority denials

## Acceptance Criteria

- [ ] Authority check semantics documented
- [ ] Obligation check semantics documented
- [ ] Role assignment mechanisms specified
- [ ] Runtime role context structure defined
- [ ] Integration points with capability system documented
- [ ] Integration points with obligation system documented
- [ ] Examples of allowed/rejected capability access
- [ ] Examples of obligation discharge patterns

## Dependencies

- None (specification-only task)
- Related to SPEC-022 (workflow obligations) but can be specified independently

## Related Documents

- `todo-examples/definitions/roles.md` (idea space)
- `crates/ash-core/src/ast.rs` (Role struct)
- `crates/ash-interp/src/capability_policy.rs` (current role usage)
- `docs/spec/SPEC-022-WORKFLOW-TYPING.md` (workflow obligations)

## Notes

This specification should clarify the relationship between:
- Role authority vs policy permits
- Role obligations vs workflow-local obligations
- Static role definitions vs runtime role assignment

The specification will inform TASK-236 (implementation).
