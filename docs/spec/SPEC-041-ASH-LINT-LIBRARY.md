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
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Per-rule enforcement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleLevel {
    Allow,
    Warn,
    Deny,
}

/// Configuration for linting.
#[derive(Debug, Clone)]
pub struct LintConfig {
    pub max_lines_per_workflow: Option<usize>,
    pub require_provenance: bool,
    pub enable_policy_lints: bool,
    pub rules: HashMap<LintCode, RuleLevel>,
}

impl Default for LintConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert(LintCode("L001"), RuleLevel::Warn);
        rules.insert(LintCode("L002"), RuleLevel::Warn);
        rules.insert(LintCode("L004"), RuleLevel::Warn);
        Self {
            max_lines_per_workflow: None,
            require_provenance: false,
            enable_policy_lints: true,
            rules,
        }
    }
}

/// Lints a single source file (raw string).
pub fn lint_source(source: &str, config: &LintConfig) -> Vec<LintDiagnostic> {
    let module = match ash_parser::parse_surface_file(source) {
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
| `L003` | Missing `provenance` block | Provenance | **Deferred.** The `Definition` enum has no provenance construct; syntax must be defined in a future spec before this rule can be implemented. |
| `L004` | Policy conflict not checked | Policy | In any block, after a `Decide` expression or `PolicyExpr` statement, there must be a `CheckObligation` or `CheckContract` expression before the block ends or before a terminal/return expression. |

> **L004 formal condition:** For each block `B`, let `decides(B)` be the set of `Decide` / `PolicyExpr` nodes in `B`. For each such node `d`, there must exist a `CheckObligation` or `CheckContract` node `c` in the same block such that `c` appears after `d` in statement order and before the end of `B`.

### 4.3 CLI Binary Refactor

`src/main.rs` becomes a CLI wrapper that preserves existing flags (`--allow`, `--deny-warnings`, `--format`) by mapping them to `LintConfig`:

```rust
fn main() {
    let args = parse_args();
    let mut config = LintConfig::default();

    // Map legacy CLI flags to LintConfig rules
    for rule in &args.allow {
        config.rules.insert(LintCode(rule), RuleLevel::Allow);
    }
    if args.deny_warnings {
        for (_, level) in config.rules.iter_mut() {
            if *level == RuleLevel::Warn {
                *level = RuleLevel::Deny;
            }
        }
    }

    let source = std::fs::read_to_string(&args.file).expect("read file");
    let diagnostics = ash_lint::lint_source(&source, &config);
    emit_diagnostics(&diagnostics, args.format);
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
2. **Property tests:**
   - `lint_module(parse_surface_file(src), &cfg)` is idempotent when run twice.
   - Every emitted `LintDiagnostic` has a non-default span (`line > 0`).
3. **Integration tests:** Run `ash-lint` binary on `examples/` and assert expected diagnostics.
4. **LSP integration tests:** Verify that `ash-lsp-core` correctly aggregates lint diagnostics with parser and type-checker diagnostics.
5. **CLI regression tests:** Verify that `--allow L001`, `--deny-warnings`, and `--format` continue to work after the refactor.

## 7. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (lint diagnostics in the pipeline)
- **Blocked by:** SPEC-039 (must provide `parse_surface_file()` and `ModuleFile` with stable AST)
- **Parallelizable with:** SPEC-040 (Diagnostic Infrastructure) after `TASK-570` binding-span changes are complete
