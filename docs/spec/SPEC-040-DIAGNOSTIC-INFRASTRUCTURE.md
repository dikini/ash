# SPEC-040: Diagnostic Infrastructure

## Status: Draft

## 1. Goal

Make all Ash compiler errors LSP-diagnostic-ready by adding source spans to every error variant and defining a uniform error trait that `ash-lsp-core` can consume.

## 2. Scope

This spec covers:
1. Adding `span: ash_parser::token::Span` to every variant of `TypeEnvError` and `NameError`.
2. Adding `span` to **all** spanless variants of `ConstructorError` (`UnknownConstructor`, `MissingField`, `UnknownField`, `FieldTypeMismatch`, `TupleFieldTypeMismatch`, `TupleArityMismatch`, `NonExhaustiveMatch`) and to `TypeError::NotAConstructor`.
3. Adding `span` to `ResolutionError` and `PurityError` variants that lack it.
4. Adding `span` to `TypeError` variants that lack it.
5. Defining a new `AshLspError` trait, a `Severity` enum, and a `DiagnosticCode` newtype in a new `crates/ash-diagnostic` crate.

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

`ConstructorError` contains both spanless and span-bearing variants. The following variants currently lack a `span`:

```rust
pub enum ConstructorError {
    // Spanless variants
    UnknownConstructor(String),          // no span
    MissingField { constructor: String, field: String }, // no span
    UnknownField { constructor: String, field: String }, // no span
    FieldTypeMismatch { constructor: String, field: String, expected: String, actual: String }, // no span
    TupleFieldTypeMismatch { constructor: String, position: usize, expected: String, actual: String }, // no span
    TupleArityMismatch { constructor: String, expected: usize, actual: usize }, // no span
    NonExhaustiveMatch {
        scrutinee_type: String,
        missing: String,
    },   // no span

    // Span-bearing variants (already present in the codebase)
    UnboundVariable {
        name: String,
        span: ash_parser::token::Span,
    },
    NotIterable {
        ty: crate::types::Type,
        span: ash_parser::token::Span,
    },
    UnsupportedExpression {
        kind: String,
        span: ash_parser::token::Span,
    },
    UnknownTypeAnnotation {
        name: String,
        context: String,
        span: ash_parser::token::Span,
    },
    InvalidInterfaceMethodCall {
        interface: String,
        method: String,
        reason: String,
        span: ash_parser::token::Span,
    },
}
```

> **Note:** `TypeError::NotAConstructor(String)` also lacks a span and is treated as part of the constructor-error scope for this spec.

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

`PurityError` already carries a `span`, but it does **not** currently implement `std::error::Error`. It must be updated to do so (e.g., by deriving `thiserror::Error` or a manual impl) before it can satisfy the `AshLspError` trait bounds.

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
    Obligation(#[from] ash_core::workflow_contract::ObligationError),
    UndischargedObligations { obligations: Vec<String> },
    UnknownObligation { name: String, span: Span },
    ObligationAlreadySatisfied { name: String, span: Span },
    UnsatisfiedObligations { obligations: Vec<String> },
    PatternMismatch { expected: Box<Type>, actual: Box<Type> },
    UnknownVariant(String),
    PatternArityMismatch { expected: usize, actual: usize },
    InvalidPattern { message: String },
    NotAConstructor(String),                                 // no span
    UnknownCapability { name: String, span: Span },          // already has span
    InvalidConstraintField { capability: String, field: String, span: Span }, // already has span
    ConstraintTypeMismatch { field: String, expected: String, found: String },
}
```

Most variants lack `span`. `UnknownCapability` and `InvalidConstraintField` already carry spans and must be verified at all construction sites.

#### TypeError variants that cannot structurally receive a span

The following variants wrap aggregate or forwarded errors and therefore **cannot** mechanically carry a single `span` field in their structure:

- `Obligation(#[from] ObligationError)` — delegates to an external error type
- `UndischargedObligations { obligations: Vec<String> }` — aggregates multiple outstanding obligations
- `UnsatisfiedObligations { obligations: Vec<String> }` — aggregates multiple unsatisfied obligations

> **Note:** `UndischargedObligations` and `UnsatisfiedObligations` appear to be functionally overlapping in `solver.rs`. When implementing span support, verify whether one of them is duplicate or dead code and can be removed.

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

Update **all** spanless variants:

```rust
UnknownConstructor {
    name: String,
    span: ash_parser::token::Span,
}

MissingField {
    constructor: String,
    field: String,
    span: ash_parser::token::Span,
}

UnknownField {
    constructor: String,
    field: String,
    span: ash_parser::token::Span,
}

FieldTypeMismatch {
    constructor: String,
    field: String,
    expected: String,
    actual: String,
    span: ash_parser::token::Span,
}

TupleFieldTypeMismatch {
    constructor: String,
    position: usize,
    expected: String,
    actual: String,
    span: ash_parser::token::Span,
}

TupleArityMismatch {
    constructor: String,
    expected: usize,
    actual: usize,
    span: ash_parser::token::Span,
}

NonExhaustiveMatch {
    scrutinee_type: String,
    missing: String,
    span: ash_parser::token::Span,
}
```

Also update `TypeError::NotAConstructor`:

```rust
NotAConstructor {
    name: String,
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

All variants updated to include `span`. For variants that already carry `span` (`UnknownObligation`, `ObligationAlreadySatisfied`, `UnknownCapability`, `InvalidConstraintField`), verify that every construction site passes a real span.

## 5. The `AshLspError` Trait

Define the trait, `Severity`, and a `DiagnosticCode` newtype in a **new crate** `crates/ash-diagnostic` (e.g., `ash_diagnostic::AshLspError`) to break the circular dependency between `ash-typeck` and future `ash-lsp-core`.

```rust
use ash_parser::token::Span;

/// Lightweight newtype for diagnostic codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCode(pub String);

pub trait AshLspError: std::fmt::Display + std::error::Error {
    fn span(&self) -> Option<Span>;
    fn severity(&self) -> Severity;
    fn code(&self) -> Option<DiagnosticCode>;
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

The `code()` method returns an `Option<DiagnosticCode>` using a lightweight taxonomy:

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

> **Prerequisite:** `PurityError` must implement `std::error::Error` before it can satisfy the `AshLspError` super-trait bounds.

### 5.3 Diagnostic Conversion

With the trait in place, converting any error to an LSP `Diagnostic` becomes mechanical. `span_to_lsp_range` is defined in SPEC-038; the helper below assumes it is available:

```rust
fn ash_error_to_diagnostic(err: &dyn AshLspError, source: &str) -> Option<Diagnostic> {
    let span = err.span()?;
    let range = span_to_lsp_range(span, source); // see SPEC-038
    Some(Diagnostic {
        range,
        severity: Some(err.severity().into()),
        code: err.code().map(|c| NumberOrString::String(c.0)),
        source: Some("ash".into()),
        message: err.message(),
        ..Default::default()
    })
}
```

### 5.4 ash-diagnostic Crate Scaffolding

Create a new crate at `crates/ash-diagnostic` with the following layout.

**`crates/ash-diagnostic/Cargo.toml`**

```toml
[package]
name = "ash-diagnostic"
version = "0.1.0"
edition = "2021"

[dependencies]
ash-parser = { path = "../ash-parser" }
# Explicit constraint: must NOT depend on ash-typeck
```

**`crates/ash-diagnostic/src/lib.rs`**

```rust
pub use ash_parser::token::Span;

/// Lightweight newtype for diagnostic codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCode(pub String);

/// Diagnostic severity levels aligned with LSP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Uniform trait for all Ash compiler errors that can be surfaced as LSP diagnostics.
pub trait AshLspError: std::fmt::Display + std::error::Error {
    fn span(&self) -> Option<Span>;
    fn severity(&self) -> Severity;
    fn code(&self) -> Option<DiagnosticCode>;
    fn message(&self) -> String { self.to_string() }
}
```

**Dependency constraints:**
- `ash-diagnostic` **may** depend on `ash-parser` (to use `Span`).
- `ash-diagnostic` **must NOT** depend on `ash-typeck` (to avoid a circular dependency; `ash-typeck` will depend on `ash-diagnostic` to implement `AshLspError` for its error types).

## 6. Call Sites to Update

Every location that constructs these errors must be updated to pass a `Span`:

- `crates/ash-typeck/src/type_env.rs` — all `TypeEnvError` construction sites
- `crates/ash-typeck/src/check_expr.rs` — all spanless `ConstructorError` construction sites
- `crates/ash-typeck/src/name_binding.rs` — all `NameError` construction sites
- `crates/ash-typeck/src/names.rs` — all `ResolutionError` construction sites
- `crates/ash-typeck/src/solver.rs` — all `TypeError` construction sites (including `NotAConstructor`)
- `crates/ash-typeck/src/purity.rs` — `PurityError` already has `span`; verify all construction sites populate it correctly and add `std::error::Error` implementation
- All test files that construct these errors directly

## 7. Migration Strategy

Because the changes are mechanical but widespread, the migration should be done **per error type** to keep diffs reviewable:
1. `TypeEnvError` + all call sites
2. `NameError` + all call sites
3. `ResolutionError` + all call sites
4. `TypeError` + all call sites
5. All spanless `ConstructorError` variants + `TypeError::NotAConstructor` + all call sites
6. Create `crates/ash-diagnostic`, define `AshLspError`, `Severity`, and `DiagnosticCode`, and implement for all error types.

## 8. Testing Strategy

1. **Unit tests:** Assert that every error variant carries a span equal to the input location.
2. **Integration tests:** Parse/type-check invalid programs and verify that every emitted diagnostic has a non-zero span.
3. **LSP bridge tests:** Verify that `ash_error_to_diagnostic` produces valid LSP ranges for a sample of each error type.
4. **Proptest:** Generate random invalid programs and assert that the span of every produced diagnostic is **not** `Span::default()` (i.e., a real span was attached).

## 9. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (unified diagnostics)
- **Blocked by:** SPEC-039 `TASK-570` (binding-span changes must land first so that `TypeError::UnboundVariable` and similar errors can capture accurate spans)
- **Parallelizable with:** SPEC-039 `TASK-571` (comment trivia) and SPEC-041 (Lint Library) after `TASK-570` is complete
