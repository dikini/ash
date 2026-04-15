# SPEC-040: Diagnostic Infrastructure

## Status: Draft

## 1. Goal

Make all Ash compiler errors LSP-diagnostic-ready by adding source spans to every error variant and defining a uniform error trait that `ash-lsp-core` can consume.

## 2. Scope

This spec covers:
1. Adding `span: ash_parser::token::Span` to every variant of `TypeEnvError` and `NameError`.
2. Adding `span` to `ConstructorError::UnknownConstructor` and `ConstructorError::NonExhaustiveMatch`.
3. Adding `span` to `ResolutionError` and `PurityError` variants that lack it.
4. Adding `span` to `TypeError` variants that lack it.
5. Defining a new `AshLspError` trait with `span()`, `severity()`, `code()`, and `message()` methods.

## 3. Current State

### 3.1 `TypeEnvError`

```rust
pub enum TypeEnvError {
    DuplicateType(String),               // no span
    TypeNotFound(String),                // no span
    InvalidDefinition(String),           // no span
    DuplicateInterface(String),          // no span
    MissingInterface(String),            // no span
    DuplicateImpl { interface: String, ty: String }, // no span
    MissingImpl { interface: String, ty: String },   // no span
    MissingInterfaceMethod { interface: String, method: String }, // no span
}
```

### 3.2 `ConstructorError`

Most variants already carry `span`, but two do not:

```rust
pub enum ConstructorError {
    UnknownConstructor(String),          // no span
    // ... other variants with spans
    NonExhaustiveMatch {
        scrutinee_type: String,
        missing: String,
    },   // no span
}
```

> **Note:** The codebase also contains an unused `ExhaustivenessError` enum in `error.rs`. It is **not** emitted by the active type-checking pipeline and is **not** part of the `AshLspError` trait.

### 3.3 `NameError`

```rust
pub enum NameError {
    Unresolved { name: String },
    Private { name: String },
    WrongTargetCapabilityAsFn { name: String },
    WrongTargetFnAsCapability { name: String },
}
```

None of these variants carry a `span` today.

### 3.4 `ResolutionError`

Defined in `crates/ash-typeck/src/names.rs`:

```rust
pub enum ResolutionError {
    UnboundVariable(String),
    DuplicateBinding(String),
    UndefinedCapability(String),
    UnresolvedSymbolicCapability { capability: String },
    UndefinedPolicy(String),
    UndefinedRole(String),
}
```

None carry `span`.

### 3.5 `PurityError`

Defined in `crates/ash-typeck/src/purity.rs`:

```rust
pub struct PurityError {
    pub kind: PurityViolation,
    pub span: ash_parser::token::Span,
}
```

`PurityError` already carries a `span`, but it is not listed in the original `AshLspError` spec.

### 3.6 `TypeError`

Defined in `crates/ash-typeck/src/solver.rs`:

```rust
pub enum TypeError {
    Mismatch { expected: Box<Type>, found: Box<Type> },
    InfiniteType { var: TypeVar, typ: Box<Type> },
    ConstructorNameMismatch { expected: String, found: String },
    ConstructorArityMismatch { name: String, expected_arity: usize, found_arity: usize },
    UnboundVariable(String),
    EffectViolation { required: Effect, actual: Effect },
    MissingCapability(String),
    UnsatisfiedObligation(String),
    Obligation(ash_core::workflow_contract::ObligationError),
    UndischargedObligations { obligations: Vec<String> },
    UnknownObligation { name: String, span: Span },
    ObligationAlreadySatisfied { name: String, span: Span },
    // ...
}
```

Most variants lack `span`.

## 4. Required Changes

### 4.1 `TypeEnvError`

Every variant must include a `span` field. Example transformations:

```rust
// Before
DuplicateType(String)

// After
DuplicateType {
    name: String,
    span: ash_parser::token::Span,
}
```

Full list of variants to update:
- `DuplicateType`
- `TypeNotFound`
- `InvalidDefinition`
- `DuplicateInterface`
- `MissingInterface`
- `DuplicateImpl`
- `MissingImpl`
- `MissingInterfaceMethod`

### 4.2 `ConstructorError`

Update both spanless variants:

```rust
UnknownConstructor {
    name: String,
    span: ash_parser::token::Span,
}

NonExhaustiveMatch {
    scrutinee_type: String,
    missing: String,
    span: ash_parser::token::Span,
}
```

### 4.3 `NameError`

All four variants updated to include `span`:
- `Unresolved { name: String, span: Span }`
- `Private { name: String, span: Span }`
- `WrongTargetCapabilityAsFn { name: String, span: Span }`
- `WrongTargetFnAsCapability { name: String, span: Span }`

### 4.4 `ResolutionError`

All variants updated to include `span`:
- `UnboundVariable { name: String, span: Span }`
- `DuplicateBinding { name: String, span: Span }`
- `UndefinedCapability { name: String, span: Span }`
- `UnresolvedSymbolicCapability { capability: String, span: Span }`
- `UndefinedPolicy { name: String, span: Span }`
- `UndefinedRole { name: String, span: Span }`

### 4.5 `TypeError`

All variants updated to include `span`. For variants that already carry `span` (`UnknownObligation`, `ObligationAlreadySatisfied`), verify that every construction site passes a real span.

## 5. The `AshLspError` Trait

Define the trait in `ash-typeck` (e.g., `ash_typeck::diagnostic::AshLspError`) because `ash-lsp-core` does not exist yet. It can be moved to `ash-lsp-core` once that crate is created in SPEC-038.

```rust
use ash_parser::token::Span;

pub trait AshLspError: std::fmt::Display + std::error::Error {
    fn span(&self) -> Option<Span>;
    fn severity(&self) -> Severity;
    fn code(&self) -> Option<String>;
    fn message(&self) -> String { self.to_string() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}
```

### 5.1 Error Codes

The `code()` method returns an `Option<String>` using a lightweight taxonomy:

| Prefix | Meaning |
|--------|---------|
| `E001` – `E099` | Parser errors |
| `E100` – `E199` | Type-checker errors (`TypeError`, `TypeEnvError`, `ConstructorError`) |
| `E200` – `E299` | Name / resolution errors (`NameError`, `ResolutionError`) |
| `E300` – `E399` | Purity / effect errors (`PurityError`) |
| `W001` – `W099` | Lint diagnostics (future) |

> **Note:** This taxonomy can start sparse; codes are assigned on an as-needed basis during implementation.

### 5.2 Implementations

Implement `AshLspError` for:
- `ash_parser::error::ParseError`
- `ash_typeck::error::ConstructorError`
- `ash_typeck::error::TypeEnvError`
- `ash_typeck::solver::TypeError`
- `ash_typeck::name_binding::NameError`
- `ash_typeck::names::ResolutionError`
- `ash_typeck::purity::PurityError`

Each implementation maps the error's internal severity logic:
- Parse errors → `Severity::Error`
- Type errors → `Severity::Error`
- Name errors → `Severity::Error`
- Resolution errors → `Severity::Error`
- Purity errors → `Severity::Error`
- Future lint diagnostics → `Severity::Warning`

> **Not included:** `ExhaustivenessError` is unused in the active pipeline and does not receive an implementation.

### 5.3 Diagnostic Conversion

With the trait in place, converting any error to an LSP `Diagnostic` becomes mechanical:

```rust
fn ash_error_to_diagnostic(err: &dyn AshLspError, source: &str) -> Option<Diagnostic> {
    let span = err.span()?;
    let range = span_to_lsp_range(span, source);
    Some(Diagnostic {
        range,
        severity: Some(err.severity().into()),
        code: err.code().map(NumberOrString::String),
        source: Some("ash".into()),
        message: err.message(),
        ..Default::default()
    })
}
```

## 6. Call Sites to Update

Every location that constructs these errors must be updated to pass a `Span`:

- `crates/ash-typeck/src/type_env.rs` — all `TypeEnvError` construction sites
- `crates/ash-typeck/src/check_expr.rs` — `ConstructorError::UnknownConstructor`, `ConstructorError::NonExhaustiveMatch`
- `crates/ash-typeck/src/name_binding.rs` — all `NameError` construction sites
- `crates/ash-typeck/src/names.rs` — all `ResolutionError` construction sites
- `crates/ash-typeck/src/solver.rs` — all `TypeError` construction sites
- `crates/ash-typeck/src/purity.rs` — `PurityError` already has `span`; verify all construction sites populate it correctly
- All test files that construct these errors directly

## 7. Migration Strategy

Because the changes are mechanical but widespread, the migration should be done **per error type** to keep diffs reviewable:
1. `TypeEnvError` + all call sites
2. `NameError` + all call sites
3. `ResolutionError` + all call sites
4. `TypeError` + all call sites
5. `ConstructorError::UnknownConstructor` + `ConstructorError::NonExhaustiveMatch` + all call sites
6. Define `AshLspError` trait and implement it for all error types.

## 8. Testing Strategy

1. **Unit tests:** Assert that every error variant carries a span equal to the input location.
2. **Integration tests:** Parse/type-check invalid programs and verify that every emitted diagnostic has a non-zero span.
3. **LSP bridge tests:** Verify that `ash_error_to_diagnostic` produces valid LSP ranges for a sample of each error type.
4. **Proptest:** Generate random invalid programs and assert that every produced diagnostic satisfies `span.line > 0 && span.column > 0`.

## 9. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (unified diagnostics)
- **Blocked by:** SPEC-039 `TASK-570` (binding-span changes must land first so that `TypeError::UnboundVariable` and similar errors can capture accurate spans)
- **Parallelizable with:** SPEC-039 `TASK-571` (comment trivia) and SPEC-041 (Lint Library) after `TASK-570` is complete
