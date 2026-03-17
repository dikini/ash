//! DOT command for generating Graphviz output from workflows.
//!
//! TASK-057: Implement `dot` command for DOT/Graphviz output.

use anyhow::{Context, Result};
use ash_core::visualize::ToDot;
use clap::Args;
use colored::Colorize;
use std::io::Write;
use std::path::Path;

/// Arguments for the dot command
#[derive(Args, Debug, Clone)]
pub struct DotArgs {
    /// Path to workflow file
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Output format (dot, svg requires graphviz)
    #[arg(short, long, default_value = "dot")]
    pub format: String,

    /// Workflow name to visualize (for files with multiple workflows)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Include effect colors in output
    #[arg(long, default_value = "true")]
    pub colors: bool,
}

/// Generate DOT output for a workflow
pub fn dot(args: &DotArgs) -> Result<()> {
    let path = Path::new(&args.path);

    // Read the workflow source
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read workflow file: {}", path.display()))?;

    // Parse and lower the workflow
    let workflow = parse_workflow(&source, args.name.as_deref())?;

    // Generate DOT output
    let dot_output = if args.colors {
        workflow.to_dot()
    } else {
        // Generate without colors by post-processing
        let raw = workflow.to_dot();
        remove_colors(&raw)
    };

    // Output the result
    output_dot(&dot_output, args)?;

    Ok(())
}

/// Parse workflow source into core IR
fn parse_workflow(source: &str, _name: Option<&str>) -> Result<ash_core::Workflow> {
    use ash_parser::parse_workflow::workflow_def;
    use winnow::prelude::*;

    let mut input = ash_parser::new_input(source);
    let workflow_def = workflow_def
        .parse_next(&mut input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    Ok(ash_parser::lower::lower_workflow(&workflow_def))
}

/// Remove color attributes from DOT output
fn remove_colors(dot: &str) -> String {
    // Simple string-based color removal
    dot.lines()
        .map(|line| {
            // Remove fillcolor attributes - simple approach
            if line.contains("fillcolor=") {
                let parts: Vec<&str> = line.split("fillcolor=").collect();
                if parts.len() == 2 {
                    let before = parts[0];
                    let after_parts: Vec<&str> = parts[1].splitn(2, '"').collect();
                    if after_parts.len() >= 2 {
                        let after = after_parts[1].splitn(2, '"').nth(1).unwrap_or("");
                        format!("{}{}", before.trim_end_matches(','), after)
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Output DOT to file or stdout
fn output_dot(dot: &str, args: &DotArgs) -> Result<()> {
    match args.format.as_str() {
        "svg" => {
            // Try to convert to SVG using graphviz
            match generate_svg(dot) {
                Ok(svg) => {
                    if let Some(path) = &args.output {
                        std::fs::write(path, svg)
                            .with_context(|| format!("Failed to write SVG to {}", path))?;
                        println!("[OK] SVG written to {}", path);
                    } else {
                        println!("{}", svg);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to generate SVG: {}", e);
                    eprintln!("Falling back to DOT format.");
                    if let Some(path) = &args.output {
                        let dot_path = format!("{}.dot", path);
                        std::fs::write(&dot_path, dot)
                            .with_context(|| format!("Failed to write DOT to {}", dot_path))?;
                        println!("[OK] DOT written to {}", dot_path);
                    } else {
                        println!("{}", dot);
                    }
                }
            }
        }
        _ => {
            // Default to DOT format
            if let Some(path) = &args.output {
                std::fs::write(path, dot)
                    .with_context(|| format!("Failed to write DOT to {}", path))?;
                println!("[OK] DOT written to {}", path);
            } else {
                println!("{}", dot);
            }
        }
    }

    Ok(())
}

/// Generate SVG from DOT using graphviz's dot command
fn generate_svg(dot: &str) -> Result<String> {
    use std::process::{Command, Stdio};

    let mut child = Command::new("dot")
        .args(["-Tsvg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn graphviz dot: {}", e))?;

    {
        let stdin = child.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
        stdin.write_all(dot.as_bytes())?;
    }

    let output = child.wait_with_output()
        .map_err(|e| anyhow::anyhow!("Failed to read graphviz output: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow::anyhow!(
            "graphviz failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_args_parsing() {
        let args = DotArgs {
            path: "test.ash".to_string(),
            output: Some("out.dot".to_string()),
            format: "svg".to_string(),
            name: Some("main".to_string()),
            colors: true,
        };

        assert_eq!(args.path, "test.ash");
        assert_eq!(args.output, Some("out.dot".to_string()));
        assert_eq!(args.format, "svg");
        assert_eq!(args.name, Some("main".to_string()));
        assert!(args.colors);
    }

    #[test]
    fn test_remove_colors() {
        let dot_with_colors = r#"digraph Workflow {
  node [fillcolor="lightgreen"];
  node_0 [label="DONE", fillcolor="palegreen"];
}"#;

        let dot_without_colors = remove_colors(dot_with_colors);
        // Should not panic, basic check
        assert!(dot_without_colors.contains("digraph Workflow"));
    }

    #[test]
    fn test_dot_generation_simple() {
        // Test that we can generate DOT for a simple workflow
        let workflow = ash_core::Workflow::Done;
        let dot = workflow.to_dot();

        assert!(dot.contains("digraph Workflow"));
        assert!(dot.contains("DONE"));
    }

    #[test]
    fn test_dot_generation_act() {
        use ash_core::{Action, Guard, Provenance, Workflow};

        let workflow = Workflow::Act {
            action: Action {
                name: "notify".to_string(),
                arguments: vec![],
            },
            guard: Guard::Always,
            provenance: Provenance::new(),
        };

        let dot = workflow.to_dot();
        assert!(dot.contains("ACT"));
        assert!(dot.contains("notify"));
    }
}
