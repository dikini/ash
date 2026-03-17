//! Doc-Test Extractor for Ash
//!
//! Extracts and tests code examples from markdown specification documents.
//!
//! # Usage
//! ```
//! ash-doc-tests docs/spec/*.md
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use pulldown_cmark::{Event, Parser as MdParser, Tag, TagEnd, CodeBlockKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "ash-doc-tests")]
#[command(about = "Extract and test code examples from markdown")]
struct Args {
    /// Markdown files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,
    
    /// Only extract, don't test
    #[arg(short, long)]
    extract_only: bool,
    
    /// Output directory for extracted tests
    #[arg(short, long, default_value = "/tmp/ash-doc-tests")]
    output: PathBuf,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug)]
struct CodeBlock {
    file: PathBuf,
    line: usize,
    lang: String,
    code: String,
    info: Vec<String>,
}

impl CodeBlock {
    fn should_test(&self) -> bool {
        // Test rust blocks unless marked ignore
        if self.lang == "rust" || self.lang == "rs" {
            !self.info.contains(&"ignore".to_string())
                && !self.info.contains(&"no_run".to_string())
        } else {
            false
        }
    }
    
    fn should_compile_only(&self) -> bool {
        self.info.contains(&"no_run".to_string())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    fs::create_dir_all(&args.output)
        .with_context(|| format!("Failed to create output directory: {:?}", args.output))?;
    
    let mut all_blocks = Vec::new();
    
    // Extract code blocks from all files
    for file in &args.files {
        if args.verbose {
            println!("{} {:?}", "Processing".blue(), file);
        }
        
        let blocks = extract_blocks(file)?;
        all_blocks.extend(blocks);
    }
    
    println!("\n{} {} code blocks", "Extracted".green(), all_blocks.len());
    
    let testable: Vec<_> = all_blocks.iter().filter(|b| b.should_test()).collect();
    let compile_only: Vec<_> = all_blocks.iter().filter(|b| b.should_compile_only()).collect();
    let ignored: Vec<_> = all_blocks.iter()
        .filter(|b| !b.should_test() && !b.should_compile_only())
        .filter(|b| b.lang == "rust" || b.lang == "rs")
        .collect();
    
    println!(
        "  {} testable, {} compile-only, {} ignored\n",
        testable.len(),
        compile_only.len(),
        ignored.len()
    );
    
    if args.extract_only {
        // Write extracted tests to output directory
        for (i, block) in all_blocks.iter().enumerate() {
            let filename = format!(
                "{}_{}.rs",
                sanitize_filename(&block.file.file_name().unwrap().to_string_lossy()),
                i
            );
            let path = args.output.join(&filename);
            fs::write(&path, &block.code)
                .with_context(|| format!("Failed to write: {:?}", path))?;
        }
        println!("{} Extracted tests to {:?}", "Success".green(), args.output);
        return Ok(());
    }
    
    // Test all testable blocks
    let mut passed = 0;
    let mut failed = 0;
    
    for (i, block) in testable.iter().enumerate() {
        print!("  Test {:3}: {}:{:4} ", i + 1, 
            block.file.display().to_string().dimmed(),
            block.line
        );
        
        match test_block(block, &args.output) {
            Ok(TestResult::Passed) => {
                println!("{}", "PASS".green());
                passed += 1;
            }
            Ok(TestResult::Compiled) => {
                println!("{}", "COMPILED".blue());
                passed += 1;
            }
            Err(e) => {
                println!("{}\n    {}", "FAIL".red(), e);
                failed += 1;
            }
        }
    }
    
    println!("\n{}: {} passed, {} failed", 
        "Results".bold(),
        passed.to_string().green(),
        failed.to_string().red()
    );
    
    if failed > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

#[derive(Debug)]
enum TestResult {
    Passed,     // Compiled and ran successfully
    Compiled,   // Compiled only (no_run)
}

fn extract_blocks(file: &PathBuf) -> Result<Vec<CodeBlock>> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read: {:?}", file))?;
    
    let parser = MdParser::new(&content);
    let mut blocks = Vec::new();
    let mut current_code = String::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();
    let mut current_info: Vec<String> = Vec::new();
    let current_start_line = 1;
    
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(lang)) => {
                in_code_block = true;
                current_code.clear();
                
                // Parse language and info string
                let lang_str = match lang {
                    CodeBlockKind::Fenced(info) => info.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let parts: Vec<&str> = lang_str.split_whitespace().collect();
                current_lang = parts.first().map(|s: &&str| s.to_string()).unwrap_or_default();
                current_info = parts.into_iter().skip(1).map(|s: &str| s.to_string()).collect();
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    blocks.push(CodeBlock {
                        file: file.clone(),
                        line: current_start_line,
                        lang: current_lang.clone(),
                        code: current_code.clone(),
                        info: current_info.clone(),
                    });
                }
                in_code_block = false;
            }
            Event::Text(text) => {
                if in_code_block {
                    current_code.push_str(&text);
                }
            }
            Event::Code(code) => {
                if in_code_block {
                    current_code.push('`');
                    current_code.push_str(&code);
                    current_code.push('`');
                }
            }
            _ => {}
        }
    }
    
    Ok(blocks)
}

fn test_block(block: &CodeBlock, temp_dir: &Path) -> Result<TestResult> {
    let test_file = temp_dir.join(format!("test_{}_{}.rs",
        sanitize_filename(&block.file.file_name().unwrap().to_string_lossy()),
        block.line
    ));
    
    // Wrap code if needed
    let code = if block.code.contains("fn main") || block.code.contains("#![") {
        block.code.clone()
    } else {
        // Wrap in main function
        format!("fn main() {{\n{}\n}}", block.code)
    };
    
    fs::write(&test_file, &code)
        .with_context(|| format!("Failed to write test file: {:?}", test_file))?;
    
    let binary_file = temp_dir.join(format!("test_{}_{}",
        sanitize_filename(&block.file.file_name().unwrap().to_string_lossy()),
        block.line
    ));
    
    // Compile
    let compile_output = Command::new("rustc")
        .arg("--edition")
        .arg("2024")
        .arg("--crate-type")
        .arg("bin")
        .arg("-o")
        .arg(&binary_file)
        .arg(&test_file)
        .output()
        .context("Failed to run rustc")?;
    
    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Compilation failed:\n{}", stderr));
    }
    
    if block.should_compile_only() {
        // Clean up and return
        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_file(&binary_file);
        return Ok(TestResult::Compiled);
    }
    
    // Run the binary
    let run_output = Command::new(&binary_file)
        .output()
        .context("Failed to run test binary")?;
    
    // Clean up
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&binary_file);
    
    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        return Err(anyhow::anyhow!("Test failed:\n{}", stderr));
    }
    
    Ok(TestResult::Passed)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_extract_rust_block() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Test\n\n```rust\nfn main() {{}}\n```\n").unwrap();
        
        let blocks = extract_blocks(&file.path().to_path_buf()).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lang, "rust");
        assert!(blocks[0].should_test());
    }
    
    #[test]
    fn test_extract_ignore_block() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "```rust ignore\nfn main() {{}}\n```\n").unwrap();
        
        let blocks = extract_blocks(&file.path().to_path_buf()).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].should_test());
    }
    
    #[test]
    fn test_extract_no_run_block() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "```rust no_run\nfn main() {{}}\n```\n").unwrap();
        
        let blocks = extract_blocks(&file.path().to_path_buf()).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].should_compile_only());
    }
}
