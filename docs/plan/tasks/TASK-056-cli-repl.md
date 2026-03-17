# TASK-056: ash repl Command

## Objective

Implement the interactive REPL for Ash development.

## Test Strategy

```rust
#[test]
fn test_repl_eval_expression() {
    let mut repl = Repl::new();
    let result = repl.eval("42");
    assert_eq!(result, Ok(Value::Int(42)));
}

#[test]
fn test_repl_type_command() {
    let mut repl = Repl::new();
    repl.eval("let x = 42").unwrap();
    let result = repl.command(":type x");
    assert!(result.contains("Int"));
}

#[test]
fn test_repl_dot_command() {
    let mut repl = Repl::new();
    let result = repl.command(":dot observe capability \"test\"");
    assert!(result.contains("digraph"));
}
```

## Implementation

```rust
#[derive(Parser)]
pub struct ReplArgs {
    /// History file
    #[arg(long)]
    pub history: Option<PathBuf>,
    
    /// Startup script
    #[arg(long)]
    pub init: Option<PathBuf>,
    
    /// Default capability bindings
    #[arg(long)]
    pub capability: Vec<(String, String)>,
}

pub struct Repl {
    context: Context,
    history: Vec<String>,
}

impl Repl {
    pub fn run(&mut self) -> Result<()> {
        // Use rustyline for line editing
        let mut rl = DefaultEditor::new()?;
        
        loop {
            let readline = rl.readline("ash> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line)?;
                    
                    if line.starts_with(':') {
                        self.handle_command(&line)?;
                    } else {
                        self.eval(&line)?;
                    }
                }
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => break,
                Err(err) => return Err(err.into()),
            }
        }
        
        Ok(())
    }
    
    fn handle_command(&mut self, cmd: &str) -> Result<()> {
        match cmd {
            ":help" => self.show_help(),
            ":quit" => std::process::exit(0),
            cmd if cmd.starts_with(":type ") => self.show_type(&cmd[6..]),
            cmd if cmd.starts_with(":effect ") => self.show_effect(&cmd[8..]),
            cmd if cmd.starts_with(":parse ") => self.show_ast(&cmd[7..]),
            cmd if cmd.starts_with(":dot ") => self.show_dot(&cmd[5..]),
            cmd if cmd.starts_with(":load ") => self.load_file(&cmd[6..]),
            _ => Err(anyhow!("Unknown command: {}", cmd)),
        }
    }
}
```

## Completion Criteria

- [ ] Interactive prompt with history
- [ ] Expression evaluation
- [ ] :help, :quit commands
- [ ] :type command shows types
- [ ] :effect command shows effect levels
- [ ] :parse command shows AST
- [ ] :dot command generates DOT
- [ ] :load command loads files
- [ ] Tests pass

## Dependencies

- TASK-012: Parser (for :parse)
- TASK-027: Expression evaluator
- ash-core visualize (for :dot)

## Estimation

8 hours
