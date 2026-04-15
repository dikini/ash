# SPEC-041: Ash Lint Library Extraction

## Status: Draft

## 1. Goal

Convert `crates/ash-lint` from a CLI-only binary into a reusable library crate that `ash-lsp-core` can depend on for lint diagnostics.

## 2. Current State

`crates/ash-lint` currently has:
- No `[lib]` section in `Cargo.toml`
- No `src/lib.rs`
- `src/main.rs` containing ~200 lines of trivial string-matching lints (e.g., `content.contains("observe")`) with hardcoded `PathBuf` I/O

## 3. Target State

`crates/ash-lint` becomes a dual crate (library + binary) with:

```toml
[package]
name = "ash-lint"
# ... existing metadata

[lib]
name = "ash_lint"
path = "src/lib.rs"

[[bin]]
name = "ash-lint"
path = "src/main.rs"
```

## 4. Library API

### 4.1 Core Types

```rust
use ash_parser::token::Span;

/// A single lint diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub struct LintDiagnostic {
    pub span: Span,
    pub code: LintCode,
    pub message: String,
    pub severity: LintSeverity,
    pub category: LintCategory,
    pub fixes: Vec<LintFix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintCode(pub &'static str);

#[derive(Debug, Clone, PartialEq)]
pub struct LintFix {
    pub span: Span,
    pub replacement: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintCategory {
    Ooda,        // OODA loop violations
    Provenance,  // Missing provenance annotations
    Policy,      // Policy-related lints
    Style,       // General style nits
}

/// Configuration for linting.
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    pub max_lines_per_workflow: Option<usize>,
    pub require_provenance: bool,
    pub enable_policy_lints: bool,
}

/// Lints a single source file (raw string).
pub fn lint_source(source: &str, config: &LintConfig) -> Vec<LintDiagnostic> {
    let module = match ash_parser::parse_module(source) {
        Ok(m) => m,
        Err(_) => return Vec::new(), // parse errors are handled by the parser
    };
    lint_module(&module, config)
}

/// Lints a parsed module.
pub fn lint_module(module: &ash_parser::surface::ModuleFile, config: &LintConfig) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    for def in &module.definitions {
        lint_definition(def, config, &mut diagnostics);
    }
    diagnostics
}
```

### 4.2 Lint Rules

The existing CLI lints must be reimplemented as AST visitors instead of string searches:

| Rule ID | Description | Category | Implementation |
|---------|-------------|----------|----------------|
| `L001` | Workflow lacks `observe` or `act` call | Ooda | Walk `ModuleFile` definitions; flag workflows with no effectful operation |
| `L002` | `act` without preceding `orient` | Ooda | Track statement sequence within workflow bodies |
| `L003` | Missing `provenance` block | Provenance | Check for `Provenance` metadata in module-level definitions |
| `L004` | Policy conflict not checked | Policy | Check for `check` calls after `decide` |

### 4.3 CLI Binary Refactor

`src/main.rs` becomes a thin CLI wrapper:

```rust
fn main() {
    let args = parse_args();
    let source = std::fs::read_to_string(&args.file).expect("read file");
    let config = LintConfig::default();
    let diagnostics = ash_lint::lint_source(&source, &config);
    for d in diagnostics {
        println!("{}:{}:{}: {}: {}", args.file, d.span.line, d.span.column, d.code, d.message);
    }
}
```

## 5. Integration with `ash-lsp-core`

`ash-lsp-core` will call:

```rust
let lint_diagnostics = ash_lint::lint_module(module_file, &LintConfig::default());
```

And convert each `LintDiagnostic` to an LSP `Diagnostic` using the same `AshLspError` conversion pipeline defined in SPEC-040.

## 6. Testing Strategy

1. **Unit tests:** Each lint rule tested against minimal source inputs.
2. **Integration tests:** Run `ash-lint` binary on `examples/` and assert expected diagnostics.
3. **LSP integration tests:** Verify that `ash-lsp-core` correctly aggregates lint diagnostics with parser and type-checker diagnostics.

## 7. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (lint diagnostics in the pipeline)
- **Blocked by:** SPEC-039 (must provide `parse_module()` and `ModuleFile` with stable AST)
- **Parallelizable with:** SPEC-040 (Diagnostic Infrastructure) after `TASK-570` binding-span changes are complete
