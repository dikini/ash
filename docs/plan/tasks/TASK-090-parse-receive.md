# TASK-090: Parse Receive Construct

## Status: ✅ Complete

## Description

Implement parser for the receive construct with pattern matching and guards.

## Specification Reference

- SPEC-013: Streams and Event Processing - Section 3.2 Receive Construct

## Requirements

### Functional Requirements

1. Parse `receive { ... }` (non-blocking)
2. Parse `receive wait { ... }` (blocking forever)
3. Parse `receive wait DURATION { ... }` (blocking with timeout)
4. Parse receive arms with patterns
5. Parse guard clauses (`if` conditions)
6. Parse control receive

### Property Requirements

```rust
parse_receive("receive { sensor:temp as t => done }").is_ok()
parse_receive("receive wait { _ => done }").is_ok()
parse_receive("receive wait 30s { _ => done }").is_ok()
parse_receive("receive control { \"shutdown\" => break }").is_ok()
```

## TDD Steps

### Step 1: Write Tests (Red)

Create `crates/ash-parser/src/parse_receive.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_receive_simple() {
        let input = "receive { sensor:temp as t => done }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok());
        let receive = result.unwrap();
        assert!(matches!(receive.mode, ReceiveMode::NonBlocking));
        assert_eq!(receive.arms.len(), 1);
    }

    #[test]
    fn test_parse_receive_wait() {
        let input = "receive wait { sensor:temp as t => done }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok());
        let receive = result.unwrap();
        assert!(matches!(receive.mode, ReceiveMode::Blocking(None)));
    }

    #[test]
    fn test_parse_receive_wait_timeout() {
        let input = "receive wait 30s { _ => act heartbeat() }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok());
        // Verify timeout parsed correctly
    }

    #[test]
    fn test_parse_receive_with_guard() {
        let input = r#"receive { 
            sensor:temp as t if t > 100 => act alert()
        }"#;
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok());
        let receive = result.unwrap();
        assert!(receive.arms[0].guard.is_some());
    }

    #[test]
    fn test_parse_receive_control() {
        let input = r#"receive control { 
            "shutdown" => break,
            _ => ()
        }"#;
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok());
        assert!(result.unwrap().is_control);
    }

    #[test]
    fn test_parse_receive_multiple_arms() {
        let input = r#"receive {
            sensor:temp as t if t > 100 => act alert(),
            sensor:temp as t => act log(t),
            _ => act skip()
        }"#;
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().arms.len(), 3);
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-parser parse_receive -- --nocapture`
Expected: FAIL - parser not implemented

### Step 3: Implement Parser (Green)

```rust
/// Parse receive expression
pub fn parse_receive(input: &mut ParseInput) -> ModalResult<Receive> {
    keyword("receive").parse_next(input)?;
    skip_whitespace(input);
    
    // Check for 'control'
    let is_control = if keyword("control").parse_next(input).is_ok() {
        skip_whitespace(input);
        true
    } else {
        false
    };
    
    // Parse mode
    let mode = if keyword("wait").parse_next(input).is_ok() {
        skip_whitespace(input);
        let timeout = opt(parse_duration).parse_next(input)?;
        ReceiveMode::Blocking(timeout)
    } else {
        ReceiveMode::NonBlocking
    };
    
    skip_whitespace(input);
    
    // Parse arms in braces
    let arms = delimited(
        literal("{"),
        separated_list0(literal(","), parse_receive_arm),
        literal("}")
    ).parse_next(input)?;
    
    Ok(Receive { mode, arms, is_control })
}

fn parse_receive_arm(input: &mut ParseInput) -> ModalResult<ReceiveArm> {
    // Parse pattern: capability:channel as pattern
    let pattern = parse_stream_pattern(input)?;
    skip_whitespace(input);
    
    // Optional guard
    let guard = if keyword("if").parse_next(input).is_ok() {
        skip_whitespace(input);
        Some(parse_expr(input)?)
    } else {
        None
    };
    
    skip_whitespace(input);
    literal("=>").parse_next(input)?;
    skip_whitespace(input);
    
    let body = parse_workflow(input)?;
    
    Ok(ReceiveArm { pattern, guard, body })
}

fn parse_stream_pattern(input: &mut ParseInput) -> ModalResult<StreamPattern> {
    // sensor:temp as { value: t }
    // or: _ (wildcard)
    // or: "literal" (for control)
    alt((
        parse_wildcard.map(|_| StreamPattern::Wildcard),
        parse_string_literal.map(StreamPattern::Literal),
        parse_binding_pattern,
    )).parse_next(input)
}

fn parse_duration(input: &mut ParseInput) -> ModalResult<Duration> {
    // Parse like "30s", "5m", "1h"
    let num: u64 = digit1.parse_next(input)?;
    let unit = one_of(['s', 'm', 'h']).parse_next(input)?;
    Ok(match unit {
        's' => Duration::from_secs(num),
        'm' => Duration::from_secs(num * 60),
        'h' => Duration::from_secs(num * 60 * 60),
        _ => unreachable!(),
    })
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-parser parse_receive -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-parser/src/parse_receive.rs
git commit -m "feat: parse receive construct with guards"
```

## Completion Checklist

- [ ] Non-blocking receive parsed
- [ ] Blocking receive with wait parsed
- [ ] Timeout duration parsed
- [ ] Receive arms with patterns parsed
- [ ] Guard clauses parsed
- [ ] Control receive parsed
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours

## Dependencies

- TASK-088 (Stream AST types)

## Blocked By

- TASK-088

## Blocks

- TASK-092 (Stream execution)
