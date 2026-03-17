# TASK-080: REPL Commands

## Status: 🔴 Not Started

## Description

Implement REPL commands: :help, :quit, :type, :ast.

## Specification Reference

- SPEC-011: REPL - Section 4 REPL Commands

## Requirements

### Functional Requirements

1. `:help` / `:h` - Show help
2. `:quit` / `:q` - Exit REPL
3. `:type` / `:t` - Show type of expression
4. `:ast` - Show AST representation
5. `:clear` - Clear screen

### Property Requirements

```rust
// Commands work
handle_command("help") -> prints help
handle_command("quit") -> exits
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_help_command() {
    let repl = Repl::new(false).unwrap();
    // Should not panic
    repl.handle_command("help");
}

#[test]
fn test_type_command() {
    let repl = Repl::new(false).unwrap();
    repl.handle_command("type 42");
    // Should print Number type
}
```

### Step 2: Implement (Green)

```rust
impl Repl {
    fn handle_command(&mut self, cmd: &str) -> Result<(), ReplError> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        match parts.first() {
            Some(&"help") | Some(&"h") => self.print_help(),
            Some(&"quit") | Some(&"q") => std::process::exit(0),
            Some(&"type") | Some(&"t") => {
                if parts.len() > 1 {
                    self.show_type(&parts[1..].join(" "))?;
                }
            }
            Some(&"ast") => {
                if parts.len() > 1 {
                    self.show_ast(&parts[1..].join(" "))?;
                }
            }
            Some(&"clear") => print!("\x1B[2J\x1B[1;1H"),
            _ => println!("Unknown command: :{}", cmd),
        }
        Ok(())
    }
    
    fn show_type(&self, expr: &str) -> Result<(), ReplError> {
        // Parse and get type without executing
        let workflow = self.engine.parse(&format!("workflow _ {{ action _ {{ effect: operational; body: || -> {}; }} }}", expr))?;
        // Type check and print type
        println!("Type: {:?}", workflow.actions[0].body.return_type());
        Ok(())
    }
}
```

## Completion Checklist

- [ ] :help command
- [ ] :quit command
- [ ] :type command
- [ ] :ast command
- [ ] :clear command
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

3 hours

## Dependencies

- TASK-077 (ash-repl crate)

## Blocked By

- TASK-077

## Blocks

None
