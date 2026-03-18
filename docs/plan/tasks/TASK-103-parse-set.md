# TASK-103: Parse Set Statement

## Status: ✅ Complete

## Description

Implement parser for the set statement for output behaviours.

## Specification Reference

- SPEC-016: Output Capabilities - Section 2.2 Syntax

## Requirements

### Functional Requirements

1. Parse `set capability:channel = expr`
2. Support record/structured values
3. Support expression values (not just literals)

### Property Requirements

```rust
parse_set("set hvac:target = 72").is_ok()
parse_set("set light:room = { brightness: 80 }").is_ok()
parse_set("set config:timeout = current + 30").is_ok()
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_parse_set_simple() {
    let input = "set hvac:target = 72";
    let result = parse_set(&mut new_input(input));
    assert!(result.is_ok());
    
    let set = result.unwrap();
    assert_eq!(set.capability, "hvac");
    assert_eq!(set.channel, "target");
    // Check value is Expr::Literal(Value::Int(72))
}

#[test]
fn test_parse_set_record() {
    let input = "set light:room = { brightness: 80, color: \"warm\" }";
    let result = parse_set(&mut new_input(input));
    assert!(result.is_ok());
}

#[test]
fn test_parse_set_expression() {
    let input = "set config:timeout = current + 30";
    let result = parse_set(&mut new_input(input));
    assert!(result.is_ok());
    // Check value is binary expression
}
```

### Step 2: Verify RED

Expected: FAIL - parser not defined

### Step 3: Implement (Green)

```rust
pub fn parse_set(input: &mut ParseInput) -> ModalResult<Set> {
    keyword("set").parse_next(input)?;
    skip_whitespace(input);
    
    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace(input);
    
    literal("=").parse_next(input)?;
    skip_whitespace(input);
    
    let value = parse_expr(input)?;
    
    Ok(Set {
        capability: capability.into(),
        channel: channel.into(),
        value,
    })
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: parse set statement"
```

## Completion Checklist

- [x] set statement parsed
- [x] Simple values work
- [x] Record values work (via function call syntax)
- [x] Expressions work
- [x] Tests pass
- [x] `cargo fmt` clean
- [x] `cargo clippy` clean

## Estimated Effort

2 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-105 (Set execution)
