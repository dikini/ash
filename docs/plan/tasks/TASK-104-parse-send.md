# TASK-104: Parse Send Statement

## Status: 🔴 Not Started

## Description

Implement parser for the send statement for output streams.

## Specification Reference

- SPEC-016: Output Capabilities - Section 3.2 Syntax

## Requirements

### Functional Requirements

1. Parse `send capability:channel expr`
2. Support all value types (literals, records, variables)

### Property Requirements

```rust
parse_send("send kafka:orders order").is_ok()
parse_send("send alerts:slack \"System down\"").is_ok()
parse_send("send metrics:timings { op: op_name, duration: elapsed }").is_ok()
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_parse_send_variable() {
    let input = "send kafka:orders order";
    let result = parse_send(&mut new_input(input));
    assert!(result.is_ok());
    
    let send = result.unwrap();
    assert_eq!(send.capability, "kafka");
    assert_eq!(send.channel, "orders");
    // Check value is variable reference
}

#[test]
fn test_parse_send_string() {
    let input = r#"send alerts:slack "System down""#;
    let result = parse_send(&mut new_input(input));
    assert!(result.is_ok());
}

#[test]
fn test_parse_send_record() {
    let input = "send metrics:timings { op: name, duration: elapsed }";
    let result = parse_send(&mut new_input(input));
    assert!(result.is_ok());
}
```

### Step 2: Verify RED

Expected: FAIL - parser not defined

### Step 3: Implement (Green)

```rust
pub fn parse_send(input: &mut ParseInput) -> ModalResult<Send> {
    keyword("send").parse_next(input)?;
    skip_whitespace(input);
    
    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace(input);
    
    // Parse value expression
    let value = parse_expr(input)?;
    
    Ok(Send {
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
git commit -m "feat: parse send statement"
```

## Completion Checklist

- [ ] send statement parsed
- [ ] Variables work
- [ ] Literals work
- [ ] Records work
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

2 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-106 (Send execution)
