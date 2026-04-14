# SPEC-037: Type and Impl Context Quotations

**Status:** Draft  
**Date:** 2026-04-14  
**Version:** 0.2  

## 1. Overview

Add context-scoped quotations to the surface syntax. Quotations allow derive handlers (and eventually compile-time metaprograms) to construct AST fragments using natural surface syntax rather than verbose Rust struct constructors. Antiquotation `$(expr)` marks a position that will be substituted during quote resolution.

For the MVP, quotations are **only resolved inside the engine's built-in derive expansion pass** (SPEC-036). If a `quote` expression or `Splice` node appears in ordinary runtime user code, it is rejected during **lowering**.

## 2. Motivation

Without quotations, built-in derive handlers in Rust are verbose:

```rust
let body = Expr::And(
    Box::new(Expr::Eq(
        Box::new(Expr::FieldAccess(Box::new(Expr::Var("a".into())), "x".into())),
        Box::new(Expr::FieldAccess(Box::new(Expr::Var("b".into())), "x".into()))
    )),
    Box::new(Expr::Eq(
        Box::new(Expr::FieldAccess(Box::new(Expr::Var("a".into())), "y".into())),
        Box::new(Expr::FieldAccess(Box::new(Expr::Var("b".into())), "y".into()))
    ))
);
```

With quotations, the handler can parse an Ash expression fragment and substitute leaves:

```rust
// Rust handler code parsing Ash syntax
let quoted_body = parse_quote_expr("a.x == b.x && a.y == b.y")?;
```

And the surrounding impl block is equally clear:

```rust
// Rust handler code parsing Ash syntax
let quoted_impl = parse_quote_impl(
    "impl Eq<$(target.name)> { eq(a, b) = $(body) }"
)?;
```

Because `type` and `impl` are top-level, self-contained contexts, there are no hidden bindings and no gensym requirements.

## 3. Syntax

### 3.1 Quotation Expressions

```ash
quote type { TypeDefinition }
quote impl { ImplDefinition }
quote expr { Expression }
```

Each keyword introduces a quotation in the corresponding syntactic category. The body is parsed using the existing parser entry point for that category.

### 3.2 Antiquotation

Inside any quotation, `$(name)` or `$(expr)` may appear in **leaf positions** where a type, expression, or identifier would normally stand:

```ash
quote expr { $(left) + $(right) }
quote impl {
    impl Eq<$(type_name)> {
        eq(a, b) = $(body)
    }
}
```

The `$(...)` content is **not evaluated by the interpreter** in the MVP. It is parsed as a minimal placeholder expression (usually an identifier or path) and stored as `Expr::Splice`. During quote resolution in the derive expansion pass, the engine substitutes each splice with a pre-computed AST node supplied by the Rust handler.

### 3.3 Restrictions

- Splice expressions inside quotations are restricted to **simple identifiers and paths** in the MVP (e.g., `$(target.name)`, `$(body)`). The parser stores the inner text as an opaque placeholder.
- Nested quotations are rejected.
- Quotations and splices appearing outside derive expansion contexts are rejected during lowering.

## 4. Semantics

### 4.1 Parser State Migration

Current `ParseInput` is:

```rust
pub type ParseInput<'a> = Stateful<LocatingSlice<&'a str>, Position>;
```

To support splicing, `Position` must be expanded into a richer state struct:

```rust
pub struct ParseState {
    pub position: Position,
    pub allow_splices: bool,
}

pub type ParseInput<'a> = Stateful<LocatingSlice<&'a str>, ParseState>;
```

This requires updating every parser function that reads or writes `input.state` to use `input.state.position` for line/column tracking and `input.state.allow_splices` for splice dispatch.

### 4.2 Parsing Quotations

When the parser encounters `quote impl {`, it:

1. Consumes the opening `{`.
2. Sets `input.state.allow_splices = true`.
3. Delegates to the existing parser entry point (`parse_type_def`, `expr`, or a refactored `parse_impl_definition` inner helper that returns `ImplDef`).
4. When the inner parser sees `$(`, and `allow_splices` is true:
   - consumes `$(`,
   - parses a simple identifier or dot-path (e.g., `target.name`),
   - expects `)`,
   - emits `Expr::Splice { path: String, span: Span }` in expression contexts, or `Type::Splice { path: String, span: Span }` in type contexts.
5. When the inner parser finishes, the outer parser expects `}` and restores `allow_splices` to its previous value.

### 4.3 Quote Resolution

Quote resolution is performed by the engine's derive expansion pass, not by the interpreter. Because quotations can appear in expression, type, or impl contexts, resolution is defined per context:

```rust
/// Resolve splices inside an expression quotation.
pub fn resolve_quote_expr(expr: &surface::Expr, subst: &HashMap<String, surface::Expr>) -> surface::Expr {
    match expr {
        surface::Expr::Splice { path, .. } => {
            subst.get(path)
                .cloned()
                .unwrap_or_else(|| panic!("unresolved splice: {}", path))
        }
        // ... recurse through all other Expr variants
        _ => recurse_children(expr, |child| resolve_quote_expr(child, subst)),
    }
}

/// Resolve splices inside a type expression.
pub fn resolve_quote_type(ty: &surface::Type, subst: &HashMap<String, surface::Type>) -> surface::Type {
    // analogous recursion; surface::Type gains a Splice variant for the MVP
}

/// Resolve splices inside an impl definition.
pub fn resolve_quote_impl(impl_def: &surface::ImplDef, expr_subst: &HashMap<String, surface::Expr>, type_subst: &HashMap<String, surface::Type>) -> surface::ImplDef {
    // Walk the impl head (type_subst) and method bodies (expr_subst)
}
```

Built-in derive handlers construct substitution maps in Rust and call the appropriate resolver:

```rust
let mut type_subst = HashMap::new();
type_subst.insert("target.name".to_string(), surface::Type::Constructor { name: target.name.clone(), args: vec![] });

let mut expr_subst = HashMap::new();
expr_subst.insert("body".to_string(), body_expr);

let resolved_impl = resolve_quote_impl(&quoted_impl, &expr_subst, &type_subst);
```

If a `Splice` path is missing from the relevant map, resolution panics or returns a `DeriveError::UnresolvedSplice`.

**Cross-context recursion:** When `resolve_quote_expr` encounters a `Type` node inside an expression (e.g., a type annotation or cast), it delegates to `resolve_quote_type` for that subtree. This ensures that type splices like `$(target.name)` inside a `quote expr` context are resolved correctly.

### 4.4 Hygiene Guarantees

Because `quote impl` generates a complete `impl` block, all bindings introduced inside it are **explicitly visible** in the template text. There is no hidden alpha-renaming.

- Generated `impl` methods introduce exactly the parameter names written in the quotation.
- Name resolution in generated code occurs at the **splice site** (the module where the `derive` clause appears), not at the built-in handler's definition site.
- If two generated methods use the same local variable name, they shadow each other exactly as hand-written code would.

## 5. IR Changes

### 5.1 Surface AST

**`crates/ash-parser/src/surface.rs`**

```rust
pub enum Expr {
    // ... existing variants

    /// Quotation of a type definition
    QuoteType {
        body: TypeDef,
        span: Span,
    },
    /// Quotation of an impl definition
    QuoteImpl {
        body: ImplDef,
        span: Span,
    },
    /// Quotation of an expression
    QuoteExpr {
        body: Box<Expr>,
        span: Span,
    },
    /// Antiquotation splice inside a quotation
    Splice {
        path: String,   -- e.g., "target.name" or "body"
        span: Span,
    },
}
```

Additionally, `surface::Type` gains a `Splice` variant so that type contexts (e.g., `impl Eq<$(target.name)>`) can contain antiquotations:

```rust
pub enum Type {
    // ... existing variants

    /// Antiquotation splice inside a type quotation
    Splice {
        path: String,
        span: Span,
    },
}
```

### 5.2 Parser State

**`crates/ash-parser/src/input.rs`**

Replace `Position`-only state with `ParseState`:

```rust
pub struct ParseState {
    pub position: Position,
    pub allow_splices: bool,
}

pub type ParseInput<'a> = Stateful<LocatingSlice<&'a str>, ParseState>;
```

Update `new_input` to initialize `ParseState { position: Position::new(), allow_splices: false }`.

### 5.3 Expression Parser

**`crates/ash-parser/src/parse_expr.rs`**

In leaf-position parsing, check `input.state.allow_splices`:

```rust
if input.state.allow_splices && input.input.starts_with("$(") {
    parse_splice(input)
} else {
    parse_leaf(input)
}
```

### 5.4 Type and Impl Parsers

**`crates/ash-parser/src/parse_module.rs`** and **`crates/ash-parser/src/parse_type_def.rs`**

The following parser functions must check `input.state.allow_splices` and delegate to `parse_splice` when true:

- `parse_surface_type` (`parse_module.rs`) — parses type expressions in signatures and annotations.
- `parse_type_def` (`parse_type_def.rs`) — parses the `type` keyword and body; type expressions inside struct fields and variant payloads must allow splices when called from `quote type`.
- `parse_impl_definition` (`parse_module.rs`) — parses `impl` heads; the interface type argument list must allow splices when called from `quote impl`.
- Any helper that parses parameter or return-type annotations (e.g., inside `parse_fn_definition`) must also respect the flag.

All of the above are called with `allow_splices = true` only when inside a quotation context.

### 5.5 Engine Quote Resolution

**`crates/ash-engine/src/derive.rs`**

Add `resolve_quote_expr`, `resolve_quote_type`, and `resolve_quote_impl` helpers used by built-in derive handlers. Add a new `DeriveError` variant:

```rust
pub enum DeriveError {
    // ... existing variants
    UnresolvedSplice(String),
}
```

## 6. Integration with Derive (SPEC-036)

Built-in derive handlers may use quotations freely:

```rust
// Inside EqDerive::expand (Rust code)
let quoted_impl = parse_quote_impl("impl Eq<$(target.name)> { eq(a, b) = $(body) }")?;

let mut type_subst = HashMap::new();
type_subst.insert(
    "target.name".to_string(),
    surface::Type::Constructor { name: target.name.clone(), args: vec![] }
);

let mut expr_subst = HashMap::new();
expr_subst.insert("body".to_string(), build_eq_body(target)?);

let resolved_impl = resolve_quote_impl(&quoted_impl, &expr_subst, &type_subst);
```

Note: For the MVP, built-in handlers may either:
- Parse quotation strings at compile time (engine startup) and cache the AST, or
- Construct `Expr::QuoteImpl` nodes directly in Rust.

Either approach is an implementation detail; the spec only requires that the final resolved AST contains no `Splice` nodes before injection into the module.

## 7. Migration Path

1. Replace `Position` with `ParseState` in `input.rs` and update all `input.state` usages across the parser crate.
2. Add `QuoteType`, `QuoteImpl`, `QuoteExpr`, and `Splice` to surface `Expr`.
3. Implement `parse_quote_expr` in `parse_expr.rs`; refactor `parse_impl_definition` into an inner `parse_impl_def -> ImplDef` helper (wrapping it with `Definition::Impl` at the existing call site) so `parse_quote_impl` can reuse it.
4. Implement `parse_splice` and thread `allow_splices` through `parse_expr`, `parse_surface_type`, the refactored impl parser, and `parse_type_def`.
5. Add `resolve_quote_expr`, `resolve_quote_type`, and `resolve_quote_impl` helpers in `ash-engine/src/derive.rs`.
6. Update built-in derive handlers to optionally use quotations.
7. Add a lowering rejection rule for `quote` / `Splice` nodes appearing outside derive contexts.
8. Test:
   - `quote expr { 1 + 2 }` resolved by a built-in handler
   - `quote impl { impl Eq<Point> { eq(a, b) = a.x == b.x } }` resolved by a built-in handler
   - Antiquotation substitution inside each context
   - Rejection of quotations in ordinary user functions

## 8. Conformance

An implementation conforming to SPEC-037 must:

- Parse `quote type { ... }`, `quote impl { ... }`, and `quote expr { ... }` as expressions.
- Support antiquotation `$(path)` in leaf positions inside each quotation context.
- Store unresolved splices as `Expr::Splice { path }` in the surface AST.
- Resolve splices during the derive expansion pass by substituting pre-computed AST nodes supplied by built-in Rust handlers.
- Reject `quote` expressions and `Splice` nodes that appear outside derive expansion contexts during lowering.
- Ensure generated code resolves names in the module where the quotation is spliced.

## 9. Files Affected

| File | Change |
|------|--------|
| `crates/ash-parser/src/input.rs` | Replace `Position`-only state with `ParseState { position, allow_splices }` |
| `crates/ash-parser/src/surface.rs` | Add `QuoteType`, `QuoteImpl`, `QuoteExpr`, `Splice` to `Expr` |
| `crates/ash-parser/src/parse_expr.rs` | Parse `quote ... { ... }` and `$(...)` splice syntax |
| `crates/ash-parser/src/parse_module.rs` | Respect `allow_splices` when parsing types and impls |
| `crates/ash-parser/src/parse_type_def.rs` | Respect `allow_splices` when parsing type expressions |
| `crates/ash-parser/src/lower.rs` | Reject `quote` / `Splice` in ordinary runtime code |
| `crates/ash-engine/src/derive.rs` | Add `resolve_quote_expr`, `resolve_quote_type`, `resolve_quote_impl` helpers and `UnresolvedSplice` error |
