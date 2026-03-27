# TASK-322B: Update Role Parser for capabilities: Syntax

## Status: 🔴 Blocking - TASK-322 Sub-task

## Problem

Role parser currently parses `authority: [name1, name2]` but SPEC-024 requires `capabilities: [name1 @ { ... }, name2]`.

## Scope

**This task ONLY updates the parser.** AST is already updated (TASK-322A).

## Implementation

### 1. Modify `parse_role_definition` in `crates/ash-parser/src/parse_module.rs`

**Current (lines 190-193):**
```rust
skip_whitespace_and_comments(input);
let authority = parse_authority_clause(input)?;
```

**New:**
```rust
skip_whitespace_and_comments(input);
let capabilities = parse_capabilities_clause(input)?;  // Reuse from parse_workflow
```

### 2. Import or Inline `parse_capabilities_clause`

Option A: Import from `parse_workflow` module
Option B: Inline the parsing logic (similar pattern)

The `parse_capabilities_clause` function already exists in `parse_workflow.rs:119` and handles:
- `capabilities:` keyword
- `[...]` list syntax
- `name @ { constraints }` syntax

### 3. Remove `parse_authority_clause` Function

Delete lines 216-222 (or mark as deprecated).

## TDD Steps

### Step 1: Write Parser Tests (Before Implementation)

```rust
#[test]
fn test_parse_role_with_capabilities() {
    let input = r#"role ai_agent {
        capabilities: [
            file @ { paths: ["/tmp/*"], read: true, write: false }
        ]
    }"#;
    
    let result = parse_module(input);
    assert!(result.is_ok());
    
    let module = result.unwrap();
    let role = match &module.definitions[0] {
        Definition::Role(r) => r,
        _ => panic!("Expected role definition"),
    };
    
    assert_eq!(role.name.as_ref(), "ai_agent");
    assert_eq!(role.capabilities.len(), 1);
    assert_eq!(role.capabilities[0].capability.as_ref(), "file");
    assert!(role.capabilities[0].constraints.is_some());
}

#[test]
fn test_parse_role_with_multiple_capabilities() {
    let input = r#"role http_client {
        capabilities: [
            network @ { hosts: ["*.example.com"] },
            file @ { paths: ["/cache/*"], read: true }
        ]
    }"#;
    
    let result = parse_module(input);
    assert!(result.is_ok());
    
    let module = result.unwrap();
    let role = match &module.definitions[0] {
        Definition::Role(r) => r,
        _ => panic!("Expected role definition"),
    };
    
    assert_eq!(role.capabilities.len(), 2);
}

#[test]
fn test_parse_role_without_constraints() {
    let input = r#"role observer {
        capabilities: [sensor]
    }"#;
    
    let result = parse_module(input);
    assert!(result.is_ok());
    
    let module = result.unwrap();
    let role = match &module.definitions[0] {
        Definition::Role(r) => r,
        _ => panic!("Expected role definition"),
    };
    
    assert_eq!(role.capabilities.len(), 1);
    assert_eq!(role.capabilities[0].capability.as_ref(), "sensor");
    assert!(role.capabilities[0].constraints.is_none());
}

#[test]
fn test_authority_syntax_rejected() {
    // Old syntax should fail to parse
    let input = r#"role ai_agent {
        authority: [file, network]
    }"#;
    
    let result = parse_module(input);
    assert!(result.is_err());  // authority: no longer valid
}
```

### Step 2: Verify Tests Fail

```bash
cargo test --package ash-parser test_parse_role
# Expected: Fail - parser doesn't recognize capabilities: syntax yet
```

### Step 3: Implement Parser Change

Update `parse_role_definition` to use `parse_capabilities_clause`.

### Step 4: Verify Tests Pass

```bash
cargo test --package ash-parser test_parse_role
# Expected: Pass
```

## Step 5: Code Review Sub-Process

Spawn a code review sub-agent to verify:

```
Review Focus for TASK-322B:
- Parser correctly handles capabilities: keyword
- Constraint syntax @ { ... } is parsed correctly
- Multiple capabilities with mixed constraints work
- Empty capabilities list is handled
- Old authority: syntax is properly removed/rejected
- Error messages are helpful for syntax mistakes
- Parser doesn't panic on malformed input
- Integration with parse_workflow's parse_capabilities_clause is clean
```

### Review Checklist (Rust-Specific)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --package ash-parser` clean
- [ ] `cargo test --package ash-parser` all tests pass
- [ ] Property tests added for constraint parsing (if proptest available)
- [ ] Edge cases covered: empty constraints, nested objects, arrays
- [ ] No parser panics on invalid input

### Review Output

Reviewer should provide:
1. **Critical issues** (must fix before merge)
2. **Suggestions** (can be addressed or noted for follow-up)
3. **Approval** or **Request Changes**

## Files to Modify

- `crates/ash-parser/src/parse_module.rs` - Update role parser
- `crates/ash-parser/src/parse_module.rs` - Remove `parse_authority_clause`

## Completion Checklist

- [ ] `parse_role_definition` parses `capabilities:` clause
- [ ] Constraint syntax `@ { ... }` works in role definitions
- [ ] Multiple capabilities can be declared
- [ ] Capabilities without constraints parse correctly
- [ ] Old `authority:` syntax is rejected (or removed)
- [ ] All parser tests pass
- [ ] `cargo test --package ash-parser` passes
- [ ] **Code review completed** with no critical issues
- [ ] Review feedback addressed (if any)

**Estimated Hours:** 2-3 (including review)
**Priority:** Blocking
**Blocked By:** TASK-322A
**Blocks:** TASK-322C
