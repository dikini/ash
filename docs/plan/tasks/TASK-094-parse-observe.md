# TASK-094: Parse Observe

## Status: ✅ Complete

## Description

Implement parser for the observe construct with constraints.

## Specification Reference

- SPEC-014: Behaviours - Section 3 Syntax

## Requirements

### Functional Requirements

1. Parse `observe capability:channel as pattern`
2. Parse `observe capability:channel where constraints as pattern`
3. Parse `changed capability:channel` (change detection)
4. Support for constraint expressions

### Property Requirements

```rust
parse_observe("observe sensor:temp as t").is_ok()
parse_observe("observe sensor:temp where unit = \"celsius\" as t").is_ok()
parse_observe("observe market:stock where symbol = \"AAPL\" as price").is_ok()
parse_changed("changed sensor:temp").is_ok()
```

## TDD Steps

### Step 1: Write Tests (Red)

Create `crates/ash-parser/src/parse_observe.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_observe_simple() {
        let input = "observe sensor:temp as t";
        let result = parse_observe(&mut new_input(input));
        assert!(result.is_ok());
        
        let observe = result.unwrap();
        assert_eq!(observe.capability, "sensor");
        assert_eq!(observe.channel, "temp");
        assert!(observe.constraints.is_empty());
    }

    #[test]
    fn test_parse_observe_with_constraint() {
        let input = r#"observe sensor:temp where unit = "celsius" as t"#;
        let result = parse_observe(&mut new_input(input));
        assert!(result.is_ok());
        
        let observe = result.unwrap();
        assert_eq!(observe.constraints.len(), 1);
        assert_eq!(observe.constraints[0].name, "unit");
    }

    #[test]
    fn test_parse_observe_multiple_constraints() {
        let input = r#"observe market:stock where symbol = "AAPL" and exchange = "NASDAQ" as price"#;
        let result = parse_observe(&mut new_input(input));
        assert!(result.is_ok());
        
        let observe = result.unwrap();
        assert_eq!(observe.constraints.len(), 2);
    }

    #[test]
    fn test_parse_observe_destructuring() {
        let input = "observe sensor:temp as { value: t, unit: u }";
        let result = parse_observe(&mut new_input(input));
        assert!(result.is_ok());
        
        let observe = result.unwrap();
        // Pattern should be record destructuring
    }

    #[test]
    fn test_parse_changed() {
        let input = "changed sensor:temp";
        let result = parse_changed(&mut new_input(input));
        assert!(result.is_ok());
        
        let changed = result.unwrap();
        assert_eq!(changed.capability, "sensor");
        assert_eq!(changed.channel, "temp");
    }

    #[test]
    fn test_parse_changed_with_constraints() {
        let input = "changed sensor:temp where unit = \"celsius\"";
        let result = parse_changed(&mut new_input(input));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().constraints.len(), 1);
    }

    #[test]
    fn test_parse_observe_complex_capability() {
        let input = "observe agent:environment as env";
        let result = parse_observe(&mut new_input(input));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().capability, "agent");
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-parser parse_observe -- --nocapture`
Expected: FAIL - parser not implemented

### Step 3: Implement Parser (Green)

```rust
use winnow::combinator::{opt, preceded};
use winnow::prelude::*;

use crate::combinators::{keyword, skip_whitespace};
use crate::input::ParseInput;
use crate::surface::{Observe, Changed, Constraint, Pattern};
use crate::parse_expr::parse_expr;
use crate::parse_pattern::parse_pattern;

/// Parse observe expression
pub fn parse_observe(input: &mut ParseInput) -> ModalResult<Observe> {
    keyword("observe").parse_next(input)?;
    skip_whitespace(input);
    
    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace(input);
    
    // Parse optional constraints
    let constraints = if keyword("where").parse_next(input).is_ok() {
        skip_whitespace(input);
        parse_constraints(input)?
    } else {
        vec![]
    };
    
    skip_whitespace(input);
    
    // Parse 'as pattern'
    keyword("as").parse_next(input)?;
    skip_whitespace(input);
    let pattern = parse_pattern(input)?;
    
    Ok(Observe {
        capability: capability.into(),
        channel: channel.into(),
        constraints,
        pattern,
    })
}

/// Parse changed expression
pub fn parse_changed(input: &mut ParseInput) -> ModalResult<Changed> {
    keyword("changed").parse_next(input)?;
    skip_whitespace(input);
    
    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace(input);
    
    // Parse optional constraints
    let constraints = if keyword("where").parse_next(input).is_ok() {
        skip_whitespace(input);
        parse_constraints(input)?
    } else {
        vec![]
    };
    
    Ok(Changed {
        capability: capability.into(),
        channel: channel.into(),
        constraints,
    })
}

fn parse_capability_ref(input: &mut ParseInput) -> ModalResult<(&str, &str)> {
    let capability = identifier(input)?;
    skip_whitespace(input);
    literal(":").parse_next(input)?;
    skip_whitespace(input);
    let channel = identifier(input)?;
    Ok((capability, channel))
}

fn parse_constraints(input: &mut ParseInput) -> ModalResult<Vec<Constraint>> {
    let mut constraints = vec![];
    
    loop {
        let constraint = parse_constraint(input)?;
        constraints.push(constraint);
        
        skip_whitespace(input);
        if !keyword("and").parse_next(input).is_ok() {
            break;
        }
        skip_whitespace(input);
    }
    
    Ok(constraints)
}

fn parse_constraint(input: &mut ParseInput) -> ModalResult<Constraint> {
    let name = identifier(input)?;
    skip_whitespace(input);
    literal("=").parse_next(input)?;
    skip_whitespace(input);
    let value = parse_expr(input)?;
    
    Ok(Constraint {
        name: name.into(),
        value,
    })
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-parser parse_observe -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-parser/src/parse_observe.rs
git commit -m "feat: parse observe with constraints"
```

## Completion Checklist

- [ ] Simple observe parsed
- [ ] Observe with constraints parsed
- [ ] Multiple constraints with 'and' parsed
- [ ] Destructuring patterns in observe
- [ ] Changed detection parsed
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

3 hours

## Dependencies

- TASK-093 (Behaviour provider)

## Blocked By

- TASK-093

## Blocks

- TASK-095 (Observe execution)
