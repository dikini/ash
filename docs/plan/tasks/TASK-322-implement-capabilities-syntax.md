# TASK-322: Implement SPEC-024 Capabilities Syntax with Declaration-Site Constraints

## Status: 🔴 Blocking - Critical Contract Gap

## Problem

Phase 46's reduced role syntax is **not implemented as specified in SPEC-024**. The current implementation uses `authority:` lists that store bare capability names, but SPEC-024 requires `capabilities:` entries with declaration-site constraints.

### Current Implementation (Incorrect)

**Parser** (`crates/ash-parser/src/parse_module.rs:181`):
```rust
let authority = parse_authority_clause(input)?;  // Only parses `authority:` lists of bare names
```

**AST** (`crates/ash-parser/src/surface.rs:94`):
```rust
pub struct RoleDef {
    pub name: Name,
    pub authority: Vec<Name>,  // Only stores bare capability names, no constraints!
    pub obligations: Vec<Name>,
    pub span: Span,
}
```

**Type Checker** (`crates/ash-typeck/src/role_checking.rs:183`):
```rust
fn compose_role_capabilities(&self, role_def: &RoleDef) -> EffectiveCapabilities {
    for authority in &role_def.authority {
        let cap_decl = CapabilityDecl {
            capability: authority.clone(),
            constraints: None,  // Always None - no constraint support!
            span: role_def.span,
        };
        effective.add_capability(role_name, cap_decl);
    }
}
```

### Required by SPEC-024

SPEC-024 Section 3.1 requires:
```ebnf
role-def          ::= "role" identifier "{" role-body "}"
role-body         ::= capability-decl*
capability-decl   ::= "capabilities" ":" "[" capability-ref-list "]"
capability-ref    ::= identifier constraint-refinement?
constraint-refinement ::= "@" "{" field-value* "}"
```

Example from SPEC-024 Section 3.2:
```ash
role ai_agent {
    capabilities: [
        file @ { paths: ["/tmp/*"], read: true, write: false }
    ]
}
```

## Impact

This is a **critical contract gap** that allows providers to be written without any of the intended safety guarantees. Constrained role capabilities cannot actually be expressed or enforced, defeating a core safety mechanism of the capability system.

## Key Finding: Infrastructure Already Exists

**Important:** The constraint parsing infrastructure is already implemented for workflows:

- `CapabilityDecl` in `surface.rs:230` already has `constraints: Option<ConstraintBlock>`
- `parse_capabilities_clause()` in `parse_workflow.rs:119` already parses constrained capability lists
- `ConstraintBlock`, `ConstraintField`, `ConstraintValue` types exist

The gap is **only in role definitions** - roles use `authority: Vec<Name>` instead of `capabilities: Vec<CapabilityDecl>`.

## Coordination and Sub-Tasks

This task uses a **sub-process architecture** with a coordination task and 6 implementation sub-tasks, each with dedicated code review:

| Task | Description | Est. Hours | Dependencies |
|------|-------------|------------|--------------|
| [TASK-322-COORD](TASK-322-COORD.md) | **Coordination Plan** - Orchestrates all sub-tasks and reviews | - | - |
| [TASK-322A](TASK-322A-role-ast-capabilitydecl.md) | Update RoleDef AST to use `CapabilityDecl` | 1-2 | None |
| [TASK-322B](TASK-322B-role-parser-capabilities.md) | Update role parser for `capabilities:` syntax | 2-3 | TASK-322A |
| [TASK-322C](TASK-322C-typeck-constrained-caps.md) | Update type checker for constrained capabilities | 2-3 | TASK-322B |
| [TASK-322D](TASK-322D-runtime-constraint-enforcement.md) | Runtime constraint enforcement | 3-4 | TASK-322C |
| [TASK-322E](TASK-322E-lower-implicit-role.md) | Update lowering for implicit default role | 2-3 | TASK-322D |
| [TASK-322F](TASK-322F-update-tests-integration.md) | Update tests and integration | 2-3 | TASK-322E |

**Total:** 13-19 hours (including coordination and reviews)

### Sub-Process Flow

Each sub-task follows this pattern:
1. **Implementation Sub-Agent** - Writes tests, implements changes
2. **Review Sub-Agent** - Code review with rust-skills compliance
3. **Fix Loop** - Address any review issues
4. **Mark Complete** - Proceed to next sub-task

See [TASK-322-COORD](TASK-322-COORD.md) for full coordination details.

## Work That Must Be Superseded

This task **supersedes and replaces** the following previous work:

- **Phase 46 role syntax implementation** (`authority:` parsing in `parse_module.rs`)
- **RoleDef AST structure** (the `authority: Vec<Name>` field)
- **`compose_role_capabilities` type checking** (unconstrained capability synthesis)

The old `authority:` syntax must be replaced with SPEC-024 compliant `capabilities:` syntax.

## Implementation Order

```
TASK-322A (AST) → TASK-322B (Parser) → TASK-322C (Type Checker) 
    → TASK-322D (Runtime) → TASK-322E (Lowering) → TASK-322F (Tests)
```

Each sub-task:
1. Can be implemented with full TDD (tests first)
2. Has clear completion criteria
3. Is small enough for a single focused session
4. Can be verified independently

## High-Level Verification

```rust
// Test: Role with constrained capabilities parses correctly
let source = r#"
role ai_agent {
    capabilities: [
        file @ { paths: ["/tmp/*"], read: true, write: false }
    ]
}
"#;
let module = parse_module(source).unwrap();
assert!(matches!(module.definitions[0], Definition::Role(RoleDef { 
    capabilities, 
    .. 
}) if capabilities[0].constraints.is_some()));

// Test: Constraint violation fails at runtime
let workflow = r#"
workflow test plays role(ai_agent) {
    act file.write with { path: "/etc/passwd", data: "x" };
}
"#;
// Should fail: /etc/passwd not in allowed paths
```

## Completion Checklist (All Sub-Tasks)

- [ ] TASK-322A: `RoleDef` uses `Vec<CapabilityDecl>` with constraints support
- [ ] TASK-322B: Role parser accepts `capabilities:` syntax with `@ { constraints }`
- [ ] TASK-322C: Type checker processes constrained capabilities from roles
- [ ] TASK-322D: Runtime enforces constraints at capability invocation
- [ ] TASK-322E: Implicit default role lowering works with new syntax
- [ ] TASK-322F: All tests updated, integration tests added
- [ ] Old `authority:` syntax completely removed
- [ ] SPEC-024 compliance verified with example programs
- [ ] CHANGELOG.md updated

**Estimated Hours:** 12-18 (sum of sub-tasks)
**Priority:** Blocking (core safety mechanism broken)
**Spec:** SPEC-024-CAPABILITY-ROLE-REDUCED.md Section 3

## Related

- SPEC-024: Capability-Role-Workflow Syntax (Reduced)
- Workflow capabilities parsing: `parse_workflow.rs:119`
- Constraint types: `surface.rs:239-272`
- Supersedes: Phase 46 `authority:` syntax implementation
- Blocks: Any production use of role-based capability constraints
