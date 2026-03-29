# SPEC-024: Capability-Role-Workflow Syntax (Reduced)

**Status:** Canonical  
**Replaces/Extends:** SPEC-017, SPEC-019, SPEC-022, SPEC-023  
**Date:** 2026-03-26  
**Version:** 1.0

---

## 1. Overview

This specification defines the reduced surface syntax for capability-role-workflow integration in Ash. The syntax reduction applies the principle of minimal surface complexity while preserving the essential expressiveness for governed AI systems.

### 1.1 Kept Syntax

| Syntax | Purpose |
|--------|---------|
| `plays role(R)` | Explicit role inclusion in workflows |
| `capabilities: [...]` | Direct capability declaration (desugars to implicit role) |
| `capability @ { constraints }` | Declaration-site capability constraints |

### 1.2 Deferred Features

| Feature | Syntax | Rationale |
|---------|--------|-----------|
| Capability composition | `capability X = A + B` | Achievable via role inclusion |
| Capability union | `capability X = A \| B` | Use case unclear |
| Use-site refinement | `cap "file" @ { path: "/x" }` | Start with declaration-site |
| Yield to workflow | `yield workflow(X)` | Use named roles |
| Implicit role leak | `yield role(X_default)` | Implementation detail |

---

## 2. Capability Definitions

### 2.1 EBNF Grammar

```ebnf
capability-def    ::= "capability" identifier capability-body
capability-body   ::= "{" capability-field* "}"
capability-field  ::= identifier ":" type-expr ","?

type-expr         ::= primitive-type
                    | "{" field-def+ "}"
                    | "[" type-expr "]"
                    | identifier

primitive-type    ::= "bool" | "int" | "float" | "string" | "effect"

field-def         ::= identifier ":" type-expr
```

### 2.2 Examples

```ash
-- Minimal capability with effect and permissions
capability file {
    effect: Operational,
    permissions: { read: bool, write: bool }
}

-- Capability with complex constraint structure
capability network {
    effect: Operational,
    hosts: [string],
    protocols: { tls: bool, http: bool }
}

-- Epistemic capability (read-only observation)
capability sensor {
    effect: Epistemic,
    schema: { value: int, unit: string }
}
```

### 2.3 Semantic Requirements

**Effect Type Mapping:**

| Surface Effect | Core Effect |
|----------------|-------------|
| `Epistemic` | Input acquisition, observation |
| `Deliberative` | Analysis, planning |
| `Evaluative` | Policy evaluation |
| `Operational` | External side effects |

**Constraint Evaluation:**
- Constraints are evaluated at capability invocation time
- All constraints must evaluate to `true` for invocation to proceed
- Constraint failure results in `ValidationError`

---

## 3. Role Definitions

### 3.1 EBNF Grammar

```ebnf
role-def          ::= "role" identifier "{" role-body "}"
role-body         ::= capability-decl*
capability-decl   ::= "capabilities" ":" "[" capability-ref-list "]"
capability-ref-list ::= capability-ref ("," capability-ref)*
capability-ref    ::= identifier constraint-refinement?
constraint-refinement ::= "@" "{" field-value* "}"
field-value       ::= identifier ":" value
value             ::= boolean | integer | string | array | record
array             ::= "[" value-list? "]"
value-list        ::= value ("," value)*
record            ::= "{" field-value* "}"
boolean           ::= "true" | "false"
```

### 3.2 Examples

```ash
-- Role with constrained capabilities
role ai_agent {
    capabilities: [
        file @ { paths: ["/tmp/*"], read: true, write: false }
    ]
}

-- Role with multiple capabilities
role http_client {
    capabilities: [
        network @ { hosts: ["*.example.com"], protocols: { tls: true, http: true } },
        file @ { paths: ["/cache/*"], read: true, write: true }
    ]
}

-- Role without constraints (pass-through)
role observer {
    capabilities: [sensor]
}
```

### 3.3 Semantic Properties

- **Authority is closed**: If a capability is not in the role's `capabilities`, the role cannot access it
- **Constraints are mandatory**: Capability constraints are checked at every invocation
- **Role assignment is static**: Assigned at spawn time, cannot change during execution

---

## 4. Workflow Definitions

### 4.1 EBNF Grammar

```ebnf
workflow-def      ::= "workflow" identifier workflow-header workflow-body
workflow-header   ::= role-inclusion* capability-decl?
role-inclusion    ::= "plays" "role" "(" identifier ")"
workflow-body     ::= "{" workflow-stmt* "}"

workflow-stmt     ::= "let" identifier "=" expr
                    | "act" identifier ("with" expr)?
                    | "observe" identifier "as" identifier
                    | "set" identifier "=" expr
                    | "send" identifier expr
                    | "receive" "{" receive-arm* "}"
                    | "yield" "role" "(" identifier ")" expr
                    | "check" identifier
                    | "oblige" identifier
                    | "match" expr "{" match-arm* "}"
                    | "if" expr "{" workflow-stmt* "}" ("else" "{" workflow-stmt* "}")?
                    | "done"
                    | "ret" expr?

receive-arm       ::= identifier ":" identifier "=>" "{" workflow-stmt* "}"
match-arm         ::= pattern "=>" "{" workflow-stmt* "}"
expr              ::= (* expression grammar per SPEC-001 *)
```

### 4.2 Examples

```ash
-- Workflow with explicit role inclusion
workflow processor
    plays role(ai_agent)
    capabilities: [network @ { hosts: ["*.example.com"] }]
{
    -- workflow body with access to ai_agent capabilities
    -- plus additional network constraints
}

-- Workflow with only explicit role (no additional capabilities)
workflow analyzer plays role(observer) {
    observe sensor as reading;
    ret reading.value;
}

-- Workflow with direct capability declaration (desugars to implicit role)
workflow quick_task capabilities: [file @ { paths: ["/tmp/*"], read: true }] {
    -- workflow body
    done;
}

-- Workflow with multiple role inclusions
workflow supervisor
    plays role(http_client)
    plays role(ai_agent)
{
    -- Combines capabilities from both roles
    done;
}
```

### 4.3 Yield Statement

```ash
-- Yield to a role for approval
workflow transfer_funds
    plays role(executor)
    capabilities: [network, file]
{
    let amount = 10000;
    
    yield role(manager) TransferRequest { amount: amount };
    -- workflow suspends, resumes when proxy responds
}
```

---

## 5. Lowering Semantics

### 5.1 Implicit Default Role

Workflows that declare capabilities directly (without an explicit role) desugar to an implicit default role at lowering time.

**Surface syntax:**
```ash
workflow quick_task capabilities: [C1, C2] { ... }
```

**Lowers to:**
```ash
role quick_task_default {
    capabilities: [C1, C2]
}

workflow quick_task plays role(quick_task_default) { ... }
```

**Important:** The implicit role name (`quick_task_default`) is an implementation detail and must not be exposed in surface syntax or user-facing error messages.

### 5.2 Role Inclusion Composition

When a workflow `plays role(R)` with its own `capabilities: [...]`, the effective capability set is the union:

```ash
workflow combined
    plays role(http_client)           -- Set A
    capabilities: [sensor @ { ... }]  -- Set B
{
    -- Effective capabilities: A ∪ B
}
```

**Constraint precedence:** If the same capability appears in both the role and the workflow's direct declaration, the workflow's constraints take precedence (narrowing).

### 5.3 Capability Composition Pattern

For Phase 46, capability composition is achieved through role inclusion rather than composition operators:

```ash
-- Instead of: capability http = network + tls
-- Use: Define a role that bundles the capabilities

role secure_http_client {
    capabilities: [network @ { protocols: { tls: true } }, file]
}

workflow api plays role(secure_http_client) { ... }
```

---

## 6. Deferred Features

The following features are deferred to future phases to minimize surface complexity:

| Feature | Syntax | Rationale | Possible Return |
|---------|--------|-----------|-----------------|
| **Capability composition operators** | `capability X = A + B` | Achievable via role inclusion; adds parser/type complexity for marginal gain | Phase 47+ if demand proven |
| **Capability union types** | `capability X = A \| B` | Use case unclear; may confuse with role choice | Future if union semantics needed |
| **Use-site constraint refinement** | `cap "file" @ { path: "/x" }` within workflow body | Start with declaration-site; can add later without breaking change | Phase 47+ for fine-grained control |
| **Implicit role syntax leak** | `yield role(X_default)` | Implementation detail should not be user-visible | Never - keep internal |
| **Yield to workflow sugar** | `yield workflow(X)` | Use named roles as interfaces for now; workflow names may change | Phase 47+ if direct calls needed |
| **Role hierarchy/inheritance** | `role admin extends user` | Adds complexity; composition via multiple `plays role` sufficient | Future if inheritance patterns emerge |
| **Dynamic role switching** | `become role(R)` | Conflicts with static analysis; unclear use case | Future if dynamic delegation needed |

### 6.1 Migration Path for Deferred Features

If deferred features are needed later:

1. **Capability composition**: Can be added as syntactic sugar that desugars to role definitions
2. **Use-site refinement**: Can be added without breaking existing code (narrowing only)
3. **Yield to workflow**: Can be added as sugar for `yield role(X_default)` (if internal exposure deemed acceptable)

---

## 7. Type Checking

### 7.1 Role Inclusion Validation

The type checker verifies:

1. **Role exists**: The role named in `plays role(R)` must be defined
2. **Capability compatibility**: Workflow's `capabilities` must not contradict role's capabilities
3. **Constraint satisfiability**: Combined constraints must be satisfiable (checked via SMT)

```rust
pub fn check_role_inclusion(
    workflow: &Workflow,
    role: &Role,
    declared_caps: &[CapabilityRef],
) -> Result<(), TypeError> {
    // Check role exists
    if !role_registry.contains(&role.name) {
        return Err(TypeError::UndefinedRole(role.name.clone()));
    }
    
    // Check capability compatibility
    for cap_ref in declared_caps {
        if let Some(role_cap) = role.find_capability(&cap_ref.name) {
            if !cap_ref.constraints.is_subset_of(&role_cap.constraints) {
                return Err(TypeError::ConstraintConflict { ... });
            }
        }
    }
    
    Ok(())
}
```

### 7.2 Capability Constraint Checking

Constraints in `@ { ... }` are type-checked as:

1. Field names must exist in capability definition
2. Field values must match declared types
3. Array values must be homogeneous
4. Record values must match expected structure

---

## 8. Runtime Semantics

### 8.1 Role Resolution

At workflow spawn:

1. Resolve all `plays role(R)` references
2. Generate implicit role if `capabilities:` declared without role
3. Compose effective capability set
4. Validate against capability registry

### 8.2 Authority Check

Before capability invocation:

```rust
pub fn check_authority(
    effective_caps: &CapabilitySet,
    requested: &Capability,
) -> Result<(), AuthorityError> {
    if let Some(cap) = effective_caps.find(&requested.name) {
        // Check constraints are satisfied
        if cap.constraints.satisfy(&requested.args) {
            return Ok(());
        }
    }
    Err(AuthorityError::NotAuthorized { ... })
}
```

### 8.3 Yield Routing

When `yield role(R)` executes:

1. Lookup proxy for role `R` in `ProxyRegistry`
2. Serialize request with correlation ID
3. Suspend workflow, add to `suspended_yields`
4. Route message to proxy's mailbox
5. On response, validate correlation ID and resume

---

## 9. References

- [SPEC-017: Capability Integration](SPEC-017-CAPABILITY-INTEGRATION.md) - Capability effects, policies, provenance
- [SPEC-019: Role Runtime Semantics](SPEC-019-ROLE-RUNTIME-SEMANTICS.md) - Role authority and obligations
- [SPEC-022: Workflow Typing](SPEC-022-WORKFLOW-TYPING.md) - Workflow contracts and obligations
- [SPEC-023: Proxy Workflows](SPEC-023-PROXY-WORKFLOWS.md) - Yield/resume semantics
- [DESIGN-014: Syntax Reduction Decisions](../design/DESIGN-014-SYNTAX-REDUCTION.md) - Design rationale
- [PHASE-44-46-ROADMAP](../plan/PHASE-44-46-ROADMAP.md) - Implementation roadmap

---

## 10. Appendix: Complete Example

```ash
-- Capability definitions
capability file {
    effect: Operational,
    permissions: { read: bool, write: bool }
}

capability network {
    effect: Operational,
    hosts: [string],
    protocols: { tls: bool }
}

capability sensor {
    effect: Epistemic,
    schema: { value: int, unit: string }
}

-- Role definitions
role data_processor {
    capabilities: [
        file @ { paths: ["/data/*", "/tmp/*"], read: true, write: true }
    ]
}

role api_client {
    capabilities: [
        network @ { hosts: ["api.example.com"], protocols: { tls: true } }
    ]
}

role observer {
    capabilities: [sensor]
}

-- Workflow definitions

-- Using explicit role
workflow file_analyzer plays role(data_processor) {
    -- Can read/write files in /data/* and /tmp/*
    act file.read with { path: "/data/input.csv" };
    done;
}

-- Using capabilities directly (desugars to implicit role)
workflow quick_check capabilities: [sensor] {
    observe sensor as reading;
    ret reading.value > 100;
}

-- Combining role and additional capabilities
workflow integrated_system
    plays role(api_client)
    capabilities: [file @ { paths: ["/cache/*"], read: true, write: true }]
{
    -- Can access api.example.com via TLS
    -- Can read/write /cache/* files
    observe sensor as status;
    
    if status.value > 100 {
        act network.post with { 
            host: "api.example.com", 
            path: "/alert",
            body: status 
        };
    }
    
    done;
}

-- Yielding to a role
workflow approval_workflow
    plays role(executor)
    capabilities: [file, network]
{
    let request = { amount: 5000, purpose: "equipment" };
    
    yield role(manager) ApprovalRequest request;
    -- Suspended until manager proxy responds
    
    done;
}
```

---

*Document Version: 1.0*  
*Status: Canonical*  
*Part of: Phase 45 Deliverable*
