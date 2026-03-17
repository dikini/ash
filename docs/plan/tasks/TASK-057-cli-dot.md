# TASK-057: ash dot Command

## Objective

Implement the `ash dot` command for AST visualization.

## Test Strategy

```rust
#[test]
fn test_dot_output() {
    let output = DotCommand::run(&["test_data/workflow.ash"]).unwrap();
    assert!(output.starts_with("digraph Workflow {"));
    assert!(output.ends_with("}\n"));
}

#[test]
fn test_dot_with_effect_colors() {
    let output = DotCommand::run(&["test_data/multi_effect.ash"]).unwrap();
    assert!(output.contains("lightgreen"));  // Epistemic
    assert!(output.contains("lightcoral")); // Operational
}

#[test]
fn test_dot_output_to_file() {
    let temp_file = NamedTempFile::new().unwrap();
    DotCommand::run(&[
        "--output", temp_file.path().to_str().unwrap(),
        "test_data/workflow.ash"
    ]).unwrap();
    
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("digraph"));
}
```

## Implementation

```rust
#[derive(Parser)]
pub struct DotArgs {
    /// Workflow file
    #[arg(required = true)]
pub workflow: PathBuf,
    
    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "dot")]
    pub format: DotFormat,
    
    /// Color nodes by effect
    #[arg(long, default_value = "true")]
    pub effect_colors: bool,
    
    /// Simplify nested structures
    #[arg(long)]
    pub simplify: bool,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum DotFormat {
    #[default]
    Dot,
    Svg,
    Png,
}

impl DotCommand {
    pub fn run(args: &DotArgs) -> Result<String> {
        // Parse workflow
        let workflow = parse_file(&args.workflow)?;
        
        // Generate DOT
        let dot = workflow.to_dot();
        
        // Convert format if needed
        let output = match args.format {
            DotFormat::Dot => dot,
            DotFormat::Svg => convert_to_svg(&dot)?,
            DotFormat::Png => convert_to_png(&dot)?,
        };
        
        // Write output
        if let Some(output_path) = &args.output {
            std::fs::write(output_path, &output)?;
        }
        
        Ok(output)
    }
}
```

## Completion Criteria

- [ ] Generates DOT output
- [ ] Writes to file or stdout
- [ ] Multiple formats (dot, svg, png)
- [ ] Effect coloring option
- [ ] Simplification option
- [ ] Tests pass

## Dependencies

- ash-core visualize module (already implemented)
- graphviz (optional, for svg/png)

## Estimation

4 hours
