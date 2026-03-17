# TASK-078: Expression Evaluation in REPL

## Status: 🔴 Not Started

## Description

Implement expression evaluation in the REPL.

## Specification Reference

- SPEC-011: REPL - Section 3 Input Handling

## Requirements

### Functional Requirements

1. Wrap expressions in workflow for execution
2. Display results
3. Handle both expressions and workflow definitions

### Property Requirements

```rust
// Simple expression evaluates
repl.input("1 + 2") -> prints "3"

// Workflow definition stores
repl.input("workflow w { ... }") -> stores for later use
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_repl_eval_expression() {
    let repl = Repl::new(false).unwrap();
    let result = repl.eval("42").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_repl_eval_workflow() {
    let mut repl = Repl::new(false).unwrap();
    let result = repl.eval(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#).await;
    assert!(result.is_ok());
}
```

### Step 2: Implement (Green)

```rust
impl Repl {
    async fn eval(&mut self, input: &str) -> Result<Value, ReplError> {
        // Check if it's a workflow definition
        if input.contains("workflow") {
            // Store workflow definitions
            self.engine.parse(input)?;
            println!("Workflow defined");
            return Ok(Value::Null);
        }
        
        // Wrap expression in workflow
        let wrapped = format!(r#"
            workflow __repl__ {{
                action __expr__ {{
                    effect: operational;
                    body: || -> {};
                }}
            }}
        "#, input);
        
        self.engine.run(&wrapped).await
    }
}
```

## Completion Checklist

- [ ] Expression evaluation works
- [ ] Workflow definitions stored
- [ ] Results displayed
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-077 (ash-repl crate)

## Blocked By

- TASK-077

## Blocks

- TASK-079 (Multiline input)
