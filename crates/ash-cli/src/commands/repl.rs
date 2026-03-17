//! REPL command for interactive Ash workflow evaluation.
//!
//! TASK-056: Implement `repl` command with rustyline integration.

use anyhow::Result;
use ash_core::Value;
use clap::Args;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Arguments for the REPL command
#[derive(Args, Debug, Clone)]
pub struct ReplArgs {
    /// History file path
    #[arg(short, long, default_value = ".ash_history")]
    pub history: String,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

/// Run the interactive REPL
pub async fn repl(args: &ReplArgs) -> Result<()> {
    if args.no_color {
        // colored crate respects NO_COLOR env var
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }

    println!("{}", "Ash Workflow Language REPL".cyan().bold());
    println!("Type {} for help, {} to quit\n", ":help".yellow(), ":quit".yellow());

    let mut rl = DefaultEditor::new()?;

    // Load history if available
    let _ = rl.load_history(&args.history);

    let mut eval_ctx = EvalContext::new();

    loop {
        let readline = rl.readline("ash> ");

        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                rl.add_history_entry(trimmed)?;

                if let Err(e) = handle_input(trimmed, &mut eval_ctx).await {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("\n{}", "Goodbye!".cyan());
                break;
            }
            Err(err) => {
                eprintln!("{} {:?}", "Error:".red().bold(), err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&args.history);

    Ok(())
}

/// Context for REPL evaluation
struct EvalContext {
    bindings: std::collections::HashMap<String, Value>,
    command_count: usize,
}

impl EvalContext {
    fn new() -> Self {
        Self {
            bindings: std::collections::HashMap::new(),
            command_count: 0,
        }
    }
}

/// Handle REPL input
async fn handle_input(input: &str, ctx: &mut EvalContext) -> Result<()> {
    ctx.command_count += 1;

    // Handle commands (starting with :)
    if input.starts_with(':') {
        return handle_command(input, ctx).await;
    }

    // Try to parse and evaluate as an expression
    match eval_expression(input) {
        Ok(value) => {
            println!("{} {}", "=>".green().bold(), value);
            // Store last result in _
            ctx.bindings.insert("_".to_string(), value);
        }
        Err(e) => {
            // Try to parse as a workflow statement
            match eval_workflow_stmt(input).await {
                Ok(Some(value)) => {
                    println!("{} {}", "=>".green().bold(), value);
                    ctx.bindings.insert("_".to_string(), value);
                }
                Ok(None) => {
                    println!("{}", "OK".green());
                }
                Err(_) => {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

/// Handle REPL commands
async fn handle_command(input: &str, ctx: &mut EvalContext) -> Result<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts.first().map(|s| *s).unwrap_or(":");

    match cmd {
        ":help" | ":h" => {
            print_help();
        }
        ":quit" | ":q" | ":exit" => {
            println!("{}", "Goodbye!".cyan());
            std::process::exit(0);
        }
        ":type" | ":t" => {
            if parts.len() < 2 {
                println!("Usage: :type <expression>");
            } else {
                let expr = parts[1..].join(" ");
                match infer_type(&expr).await {
                    Ok(ty) => println!("{} {}", "Type:".cyan(), ty),
                    Err(e) => eprintln!("{} {}", "Error:".red(), e),
                }
            }
        }
        ":bindings" | ":b" => {
            if ctx.bindings.is_empty() {
                println!("No bindings in current context.");
            } else {
                println!("{}", "Current bindings:".cyan().bold());
                for (name, value) in &ctx.bindings {
                    println!("  {} = {}", name.yellow(), value);
                }
            }
        }
        ":clear" => {
            ctx.bindings.clear();
            println!("{}", "Bindings cleared.".green());
        }
        _ => {
            println!("{} Unknown command: {}", "Error:".red(), cmd);
            println!("Type {} for available commands.", ":help".yellow());
        }
    }

    Ok(())
}

/// Print REPL help
fn print_help() {
    println!("{}", "Ash REPL Commands:".cyan().bold());
    println!();
    println!("  {}         Show this help message", ":help".yellow());
    println!("  {}        Show the type of an expression", ":type <expr>".yellow());
    println!("  {}      List all variable bindings", ":bindings".yellow());
    println!("  {}        Clear all bindings", ":clear".yellow());
    println!("  {}         Exit the REPL", ":quit".yellow());
    println!();
    println!("{}", "Expressions:".cyan().bold());
    println!();
    println!("  Enter any Ash expression or workflow statement to evaluate it.");
    println!("  Examples:");
    println!("    42");
    println!("    \"hello\"");
    println!("    let x = 10 in x + 5");
    println!();
}

/// Evaluate an expression (synchronous - not async)
fn eval_expression(input: &str) -> Result<Value> {
    use ash_parser::parse_expr::expr;
    use winnow::prelude::*;

    let mut parse_input = ash_parser::new_input(input);
    let surface_expr = expr.parse_next(&mut parse_input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let core_expr = ash_parser::lower::lower_expr(&surface_expr);

    // Evaluate using a simple context - eval_expr is synchronous
    let mut eval_ctx = ash_interp::Context::new();
    ash_interp::eval_expr(&core_expr, &mut eval_ctx)
        .map_err(|e| anyhow::anyhow!("Evaluation error: {:?}", e))
}

/// Evaluate a workflow statement
async fn eval_workflow_stmt(input: &str) -> Result<Option<Value>> {
    use ash_parser::parse_workflow::workflow_def;
    use winnow::prelude::*;

    // Parse a workflow definition
    let mut parse_input = ash_parser::new_input(input);
    let surface_wf_def = workflow_def.parse_next(&mut parse_input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let core_wf = ash_parser::lower::lower_workflow(&surface_wf_def);

    ash_interp::interpret(&core_wf)
        .await
        .map(Some)
        .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))
}

/// Infer the type of an expression
async fn infer_type(input: &str) -> Result<String> {
    use ash_parser::parse_expr::expr;
    use winnow::prelude::*;

    let mut parse_input = ash_parser::new_input(input);
    let surface_expr = expr.parse_next(&mut parse_input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Create a dummy workflow with this expression
    let dummy_wf = ash_parser::surface::Workflow::Orient {
        expr: surface_expr,
        binding: None,
        continuation: None,
        span: ash_parser::token::Span::default(),
    };

    let result = ash_typeck::type_check_workflow(&dummy_wf)
        .map_err(|e| anyhow::anyhow!("Type check error: {:?}", e))?;

    Ok(format!("{:?}", result.effect))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_args_parsing() {
        let args = ReplArgs {
            history: ".my_history".to_string(),
            no_color: true,
        };

        assert_eq!(args.history, ".my_history");
        assert!(args.no_color);
    }

    #[test]
    fn test_repl_args_defaults() {
        let args = ReplArgs {
            history: ".ash_history".to_string(),
            no_color: false,
        };

        assert_eq!(args.history, ".ash_history");
        assert!(!args.no_color);
    }

    #[test]
    fn test_eval_expression() {
        // Test literal evaluation - synchronous
        let result = eval_expression("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));

        // Test string evaluation
        let result = eval_expression("\"hello\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_context() {
        let mut ctx = EvalContext::new();
        assert!(ctx.bindings.is_empty());

        ctx.bindings.insert("x".to_string(), Value::Int(10));
        assert_eq!(ctx.bindings.get("x"), Some(&Value::Int(10)));
    }
}
