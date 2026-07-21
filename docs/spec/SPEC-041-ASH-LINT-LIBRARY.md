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

[dependencies]
ash-parser = { path = "../ash-parser" }
serde = { workspace = true, optional = true }

# `walkdir` is primarily used by the CLI binary for directory traversal.
# It lives in [dependencies] (not [bin.dependencies]) because Cargo does not
# support [bin.dependencies], and the library may later expose a
# `lint_directory` helper that also needs it.
walkdir = { workspace = true }

[features]
serde = ["dep:serde"]
```

## 4. Library API

### 4.1 Core Types

```rust
use ash_parser::token::Span;
use std::collections::HashMap;

/// A single lint diagnostic.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintDiagnostic {
    pub span: Span,
    pub code: LintCode,
    pub message: String,
    pub severity: LintSeverity,
    pub category: LintCategory,
    pub fixes: Vec<LintFix>,
    /// Reserved for future LSP-style related locations.
    pub related_information: Vec<LintRelatedInformation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintCode(pub String);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintFix {
    pub span: Span,
    pub replacement: String,
    pub description: String,
}

/// Reserved for future LSP-style related diagnostic locations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintRelatedInformation {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LintSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LintCategory {
    Ooda,        // OODA loop violations
    Provenance,  // Placeholder: missing provenance annotations (syntax TBD)
    Policy,      // Policy-related lints
    Style,       // Placeholder: general style nits
}

/// Per-rule enforcement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RuleLevel {
    Allow,
    Warn,
    Deny,
}

/// Configuration for linting.
#[derive(Debug, Clone)]
pub struct LintConfig {
    pub require_provenance: bool,
    pub enable_policy_lints: bool,
    pub rules: HashMap<LintCode, RuleLevel>,
}

impl Default for LintConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert(LintCode("L001".to_string()), RuleLevel::Warn);
        rules.insert(LintCode("L002".to_string()), RuleLevel::Warn);
        rules.insert(LintCode("L004".to_string()), RuleLevel::Warn);
        Self {
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

/// Lints a single definition, appending diagnostics to the provided vector.
pub fn lint_definition(
    def: &ash_parser::surface::Definition,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    match def {
        ash_parser::surface::Definition::Capability(_) => {
            // no lint
        }
        ash_parser::surface::Definition::Policy(_) => {
            // no lint
        }
        ash_parser::surface::Definition::Role(_) => {
            // no lint
        }
        ash_parser::surface::Definition::Interface(_) => {
            // no lint
        }
        ash_parser::surface::Definition::Impl(_) => {
            // no lint
        }
        ash_parser::surface::Definition::Function(_) => {
            // Function-level lints are out of scope for the MVP.
        }
    }
}
```

### 4.2 Lint Rules

The existing CLI lints must be reimplemented as AST visitors instead of string searches:

| Rule ID | Legacy Alias | Description | Category | Implementation |
|---------|--------------|-------------|----------|----------------|
| `L001` | `ooda-missing-decide` | Workflow lacks `observe` or `act` call | Ooda | Walk `ModuleFile` definitions; flag workflows with no effectful operation |
| `L002` | `ooda-missing-orient` | `act` without preceding `orient` | Ooda | Track statement sequence within workflow bodies |
| `L003` | — | Missing `provenance` block | Provenance | **Deferred.** The `Definition` enum has no provenance construct; syntax must be defined in a future spec before this rule can be implemented. |
| `L004` | — | Policy conflict not checked | Policy | After any `Workflow::Decide` or `Expr::Policy`, every control-flow path must contain a `Workflow::Check { target: CheckTarget, .. }` before a terminal node. |

> **Compatibility:** The CLI binary must accept the legacy rule IDs `ooda-missing-decide` and `ooda-missing-orient` as aliases for `L001` and `L002` respectively when parsing `--allow` / `--deny` arguments.
> These OODA lints are library/template compatibility guidance for historical examples. They should point users toward the visible tower algebra and explicit `Act`, `Proc`, and `Workflow` operations; they are not primitive alpha execution semantics.
>
> **Legacy alias semantic drift:** The aliases are **name-only** mappings. The original CLI performed primitive string searches, whereas the library rules operate on the AST, so behaviour may differ slightly (e.g., span precision and exact matching). These legacy IDs are **deprecated**; new code should use `L001` and `L002`.

> **L004 helper predicate:** Define `ContainsPolicy(expr)` recursively over all `Expr` variants. It returns `true` iff the expression tree rooted at `expr` contains any `Expr::Policy` node.
>
> ```rust
> fn ContainsPolicy(expr: &Expr) -> bool {
>     match expr {
>         Expr::Policy(_) => true,
>         Expr::FieldAccess { base, .. } => ContainsPolicy(base),
>         Expr::IndexAccess { base, index } => ContainsPolicy(base) || ContainsPolicy(index),
>         Expr::Unary { operand, .. } => ContainsPolicy(operand),
>         Expr::Binary { left, right, .. } => ContainsPolicy(left) || ContainsPolicy(right),
>         Expr::Call { args, .. } => args.iter().any(ContainsPolicy),
>         Expr::Match { scrutinee, arms, .. } => {
>             ContainsPolicy(scrutinee) || arms.iter().any(|a| ContainsPolicy(&a.body))
>         }
>         Expr::IfLet { expr, then_branch, else_branch, .. } => {
>             ContainsPolicy(expr) || ContainsPolicy(then_branch) || ContainsPolicy(else_branch)
>         }
>         Expr::Constructor { fields, payload, .. } => {
>             fields.iter().any(|(_, e)| ContainsPolicy(e))
>                 || match payload {
>                     ConstructorPayload::Unit => false,
>                     ConstructorPayload::Record(fs) => fs.iter().any(|(_, e)| ContainsPolicy(e)),
>                     ConstructorPayload::Tuple(es) => es.iter().any(ContainsPolicy),
>                 }
>         }
>         Expr::If { condition, then_branch, else_branch, .. } => {
>             let else_has = else_branch.as_ref().map_or(false, |e| ContainsPolicy(e));
>             ContainsPolicy(condition) || ContainsPolicy(then_branch) || else_has
>         }
>         Expr::Block { statements, tail_expr, .. } => {
>             statements.iter().any(|s| match s {
>                 BlockStmt::Let { expr, .. } => ContainsPolicy(expr),
>             }) || tail_expr.as_ref().map_or(false, |e| ContainsPolicy(e))
>         }
>         Expr::FnDef { body, .. } => ContainsPolicy(body),
>         Expr::FnApply { func, args, .. } => {
>             ContainsPolicy(func) || args.iter().any(ContainsPolicy)
>         }
>         Expr::Literal(_) | Expr::Variable(_) | Expr::CheckObligation { .. } | Expr::Panic { .. } => false,
>     }
> }
> ```
>
> **L004 formal condition (recursive CPS):** Define `Safe(w, pending)` where `w` is a `Workflow` and `pending ∈ {true, false}` indicates whether an unmatched `Workflow::Decide` or `Expr::Policy` precedes `w`:
>
> 1. `Safe(Workflow::Done {..}, pending) = ¬pending`
> 2. `Safe(Workflow::Ret { expr, .. }, pending) = ¬pending ∧ ¬ContainsPolicy(expr)` — a `Ret` carrying a policy expression is also unsafe.
> 3. `Safe(Workflow::Check { continuation: Some(c), .. }, _) = Safe(c, false)` — a `Check` satisfies any pending requirement; the path continues through its continuation with `pending` reset.
> 4. `Safe(Workflow::Check { continuation: None, .. }, pending) = ¬pending`
> 5. `Safe(Workflow::Decide { then_branch, else_branch: Some(else_branch), .. }, pending) = Safe(then_branch, true) ∧ Safe(else_branch, true)` — both branches inherit `pending = true`.
> 5a. `Safe(Workflow::Decide { then_branch, else_branch: None, .. }, pending) = Safe(then_branch, true)` — with no else branch, the then-branch alone must satisfy the condition.
> 6. `Safe(Workflow::If { then_branch, else_branch: Some(else_branch), .. }, pending) = Safe(then_branch, pending) ∧ Safe(else_branch, pending)` — `If` propagates the pending state.
> 6a. `Safe(Workflow::If { then_branch, else_branch: None, .. }, pending) = Safe(then_branch, pending)` — with no else branch, the then-branch alone must satisfy the condition.
> 7. `Safe(Workflow::Seq { left, right, .. }, pending) = Safe(left, pending) ∧ Safe(right, pending)` — both branches inherit `pending`.
> 8. For any other Workflow variant `w` (including but not limited to `Oblige`, `For`, `With`, `Maybe`, `Must`, `Receive`, `Yield`, `Resume`) that can contain expressions:
>    - Let `has_policy = true` iff `ContainsPolicy(expr)` for any expression `expr` appearing in `w`.
>    - If `w` has a continuation field `continuation: Some(c)`, then `Safe(w, pending) = Safe(c, pending ∨ has_policy)`.
>    - Otherwise, `Safe(w, pending) = ¬(pending ∨ has_policy)`.
>
> A workflow definition satisfies L004 iff `Safe(body, false)` holds for its body.

### 4.3 CLI Binary Refactor

`src/main.rs` becomes a CLI wrapper that preserves existing flags (`--allow`, `--deny-warnings`, `--format`) by mapping them to `LintConfig`:

```rust
fn main() {
    let args = parse_args();
    let mut config = LintConfig::default();

    // Map legacy CLI flags to LintConfig rules
    for rule in &args.allow {
        let code = match rule.as_str() {
            "ooda-missing-decide" => "L001",
            "ooda-missing-orient" => "L002",
            other => other,
        };
        config.rules.insert(LintCode(code.to_string()), RuleLevel::Allow);
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

`LintDiagnostic` implements `AshLspError` so it feeds directly into the SPEC-040 diagnostic pipeline:

```rust
use ash_lsp_core::diagnostics::AshLspError;

impl AshLspError for LintDiagnostic {
    fn message(&self) -> String {
        format!("[{}] {}", self.code.0, self.message)
    }

    fn severity(&self) -> lsp_types::DiagnosticSeverity {
        match self.severity {
            LintSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
            LintSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
            LintSeverity::Information => lsp_types::DiagnosticSeverity::INFORMATION,
            LintSeverity::Hint => lsp_types::DiagnosticSeverity::HINT,
        }
    }

    fn span(&self) -> &ash_parser::token::Span {
        &self.span
    }

    fn code(&self) -> Option<String> {
        Some(self.code.0.clone())
    }
}
```

## 6. Testing Strategy

1. **Unit tests:** Each lint rule tested against minimal source inputs.
2. **Property tests:**
   - `lint_module(parse_surface_file(src), &cfg)` is idempotent when run twice.
   - Every emitted `LintDiagnostic` has a non-default span (`line > 0`).
3. **Integration tests:** Run `ash-lint` binary on `examples/` and assert expected diagnostics.
4. **LSP integration tests:** Verify that `ash-lsp-core` correctly aggregates lint diagnostics with parser and type-checker diagnostics.
5. **CLI regression tests:** Verify that `--allow L001`, `--allow ooda-missing-decide`, `--deny-warnings`, and `--format` continue to work after the refactor.

## 7. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (lint diagnostics in the pipeline)
- **Blocked by:** SPEC-039 must deliver `parse_surface_file()` and a stable `ModuleFile` AST as an explicit, gated deliverable. This spec **cannot proceed to implementation** until SPEC-039 §4.6 acceptance criteria are met: the `ModuleFile` AST is frozen, all parser tests pass, and `parse_surface_file()` is published as a public, documented API.
- **Parallelizable with:** SPEC-040 (Diagnostic Infrastructure) after `TASK-570` binding-span changes are complete
