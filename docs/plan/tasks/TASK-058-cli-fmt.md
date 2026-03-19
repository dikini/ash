# TASK-058: CLI Format Command

## Status: ✅ Complete

## Description

Implement the `ash fmt` command for formatting Ash workflow source files.

## Specification Reference

- SPEC-005: CLI Specification - Format section

## Requirements

### Format Command

```rust
use clap::Args;

#[derive(Debug, Args)]
pub struct FmtArgs {
    /// File or directory to format
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
    
    /// Check formatting without modifying files
    #[arg(long, short)]
    pub check: bool,
    
    /// Write formatted output to stdout instead of file
    #[arg(long, short)]
    pub stdin: bool,
    
    /// Number of spaces for indentation
    #[arg(long, default_value = "2")]
    pub indent: usize,
    
    /// Maximum line length
    #[arg(long, default_value = "80")]
    pub max_width: usize,
}

pub async fn fmt_command(args: FmtArgs) -> Result<ExitCode, CliError> {
    if args.stdin {
        format_stdin(args.indent, args.max_width).await
    } else if args.path.is_file() {
        format_file(&args.path, args.check, args.indent, args.max_width).await
    } else if args.path.is_dir() {
        format_directory(&args.path, args.check, args.indent, args.max_width).await
    } else {
        Err(CliError::NotFound(args.path))
    }
}
```

### Formatter Implementation

```rust
/// Format a single file
async fn format_file(
    path: &Path,
    check: bool,
    indent: usize,
    max_width: usize,
) -> Result<ExitCode, CliError> {
    let source = tokio::fs::read_to_string(path).await
        .map_err(|e| CliError::Io(path.to_path_buf(), e))?;
    
    let formatted = format_source(&source, indent, max_width)?;
    
    if check {
        if source != formatted {
            eprintln!("{}: would reformat", path.display());
            return Ok(ExitCode::FAILURE);
        }
        Ok(ExitCode::SUCCESS)
    } else {
        if source != formatted {
            tokio::fs::write(path, formatted).await
                .map_err(|e| CliError::Io(path.to_path_buf(), e))?;
            println!("{}: reformatted", path.display());
        }
        Ok(ExitCode::SUCCESS)
    }
}

/// Format from stdin to stdout
async fn format_stdin(indent: usize, max_width: usize) -> Result<ExitCode, CliError> {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source)
        .map_err(|e| CliError::Io(PathBuf::from("<stdin>"), e))?;
    
    let formatted = format_source(&source, indent, max_width)?;
    print!("{}", formatted);
    
    Ok(ExitCode::SUCCESS)
}

/// Format all files in a directory
async fn format_directory(
    path: &Path,
    check: bool,
    indent: usize,
    max_width: usize,
) -> Result<ExitCode, CliError> {
    let mut entries = tokio::fs::read_dir(path).await
        .map_err(|e| CliError::Io(path.to_path_buf(), e))?;
    
    let mut all_formatted = true;
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| CliError::Io(path.to_path_buf(), e))? {
        let path = entry.path();
        
        if path.extension().map_or(false, |e| e == "ash") {
            let result = format_file(&path, check, indent, max_width).await?;
            if result != ExitCode::SUCCESS {
                all_formatted = false;
            }
        }
    }
    
    if all_formatted {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}
```

### Source Formatter

```rust
/// Format Ash source code
pub fn format_source(source: &str, indent: usize, max_width: usize) -> Result<String, FmtError> {
    // Parse the source
    let program = parse(source).map_err(|e| FmtError::Parse(e))?;
    
    // Format the AST
    let mut formatter = Formatter::new(indent, max_width);
    formatter.format_program(&program);
    
    Ok(formatter.output())
}

/// Source code formatter
pub struct Formatter {
    indent: usize,
    max_width: usize,
    output: String,
    current_indent: usize,
}

impl Formatter {
    pub fn new(indent: usize, max_width: usize) -> Self {
        Self {
            indent,
            max_width,
            output: String::new(),
            current_indent: 0,
        }
    }
    
    pub fn format_program(&mut self, program: &Program) {
        for def in &program.definitions {
            self.format_definition(def);
            self.newline();
        }
        
        self.format_workflow(&program.workflow);
    }
    
    fn format_definition(&mut self, def: &Definition) {
        match def {
            Definition::Capability(cap) => {
                self.write("capability ");
                self.write(&cap.name);
                self.write(": ");
                self.write(&format!("{:?}", cap.effect));
                // ... more formatting
            }
            Definition::Policy(policy) => {
                self.write("policy ");
                self.write(&policy.name);
                self.write(":");
                self.indent();
                self.newline();
                self.write("when ");
                self.format_expr(&policy.condition);
                // ... more formatting
            }
            _ => {}
        }
    }
    
    fn format_workflow(&mut self, workflow: &WorkflowDef) {
        self.write("workflow ");
        self.write(&workflow.name);
        self.write(" {");
        self.indent();
        self.newline();
        
        self.format_workflow_body(&workflow.body);
        
        self.dedent();
        self.newline();
        self.write("}");
    }
    
    fn format_workflow_body(&mut self, workflow: &Workflow) {
        match workflow {
            Workflow::Observe { capability, pattern, continuation } => {
                self.write("observe ");
                self.write(&capability.name);
                
                if let Some(pat) = pattern {
                    self.write(" as ");
                    self.format_pattern(pat);
                }
                
                if let Some(cont) = continuation {
                    self.write(";");
                    self.newline();
                    self.format_workflow_body(cont);
                }
            }
            Workflow::Let { pattern, expr, continuation } => {
                self.write("let ");
                self.format_pattern(pattern);
                self.write(" = ");
                self.format_expr(expr);
                
                if let Some(cont) = continuation {
                    self.write(";");
                    self.newline();
                    self.format_workflow_body(cont);
                }
            }
            // ... more cases
            _ => {}
        }
    }
    
    fn format_expr(&mut self, expr: &Expr) {
        // Format expression
        match expr {
            Expr::Literal(val) => self.write(&format!("{:?}", val)),
            Expr::Var(name) => self.write(name),
            Expr::BinOp { op, left, right } => {
                self.format_expr(left);
                self.write(&format!(" {:?} ", op));
                self.format_expr(right);
            }
            _ => {}
        }
    }
    
    fn format_pattern(&mut self, pat: &Pattern) {
        match pat {
            Pattern::Variable(name) => self.write(name),
            Pattern::Wildcard => self.write("_"),
            Pattern::Tuple(pats) => {
                self.write("(");
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.format_pattern(p);
                }
                self.write(")");
            }
            _ => {}
        }
    }
    
    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }
    
    fn newline(&mut self) {
        self.output.push('\n');
        for _ in 0..self.current_indent {
            self.output.push(' ');
        }
    }
    
    fn indent(&mut self) {
        self.current_indent += self.indent;
    }
    
    fn dedent(&mut self) {
        self.current_indent = self.current_indent.saturating_sub(self.indent);
    }
    
    pub fn output(&self) -> String {
        self.output.clone()
    }
}
```

## TDD Steps

### Step 1: Create FmtArgs

Add to CLI args.

### Step 2: Implement Formatting

Create Formatter struct.

### Step 3: Implement Command

Add fmt_command.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_observe() {
        let source = r#"workflow test{observe read as x;done}"#;
        let formatted = format_source(source, 2, 80).unwrap();
        
        assert!(formatted.contains("workflow test {"));
        assert!(formatted.contains("  observe read as x;"));
    }

    #[test]
    fn test_format_let() {
        let source = r#"workflow test{let x=1;done}"#;
        let formatted = format_source(source, 2, 80).unwrap();
        
        assert!(formatted.contains("let x = 1;"));
    }

    #[test]
    fn test_indent_custom() {
        let source = r#"workflow test{observe read;done}"#;
        let formatted = format_source(source, 4, 80).unwrap();
        
        assert!(formatted.contains("    observe read;"));
    }
}
```

## Completion Checklist

- [ ] FmtArgs struct
- [ ] fmt_command
- [ ] format_file
- [ ] format_stdin
- [ ] format_directory
- [ ] Formatter struct
- [ ] AST formatting
- [ ] Unit tests for formatting
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Correctness**: Is formatting idempotent?
2. **Coverage**: Are all constructs formatted?
3. **Options**: Are indent/width options respected?

## Estimated Effort

4 hours

## Dependencies

- ash-parser: AST types

## Blocked By

- ash-parser: Parser

## Blocks

- TASK-060: Integration tests
