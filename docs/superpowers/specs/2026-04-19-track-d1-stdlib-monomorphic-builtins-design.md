# Track D1: Stdlib Monomorphic Builtin Migration

**Date:** 2026-04-19  
**Phase:** 92 — `builtin fn` Declaration Form  
**Tasks:** TASK-623, TASK-626 (+ prerequisite module-name propagation fix)  
**Status:** Design approved

---

## Goal

Create `std/src/string.ash` and `std/src/record.ash` with `pub builtin fn` declarations for the four string operations and three record operations currently hardcoded in the evaluator. After Track D1, these functions are importable via the standard module system (`use string::{concat}`) and execute through the runtime dispatch table.

Track D2 (polymorphic list ops), D1.5 (type predicates), and Track F cleanup (TASK-631A) are out of scope.

---

## Prerequisite: Module Name Propagation

### Problem

`build_imported_closures` currently generates synthetic closure bodies with `module: None`:

```
Expr::Call { func: "concat", module: None, arguments: [...] }
```

For record ops (`keys`, `values`) this is correct — they are unqualified in the dispatch table. For string ops it is wrong: the dispatch table key is `"string::concat"`, not `"concat"`. The unqualified `"concat"` dispatches to *list* concatenation.

### Fix

**`CallableKind::Builtin` gains a `module: String` field.**

```rust
// crates/ash-engine/src/module_loader.rs
pub enum CallableKind {
    User { body: Expr },
    Builtin { module: String },
}
```

The module name is populated at import resolution time in `load_ordinary_file`, using `import.module_segments.join("::")`, in both the `Named` and `Glob` import branches when a `Builtin` callable is resolved.

**`build_imported_closures` uses a dispatch-table check** to pick qualified vs. unqualified call form:

```rust
CallableKind::Builtin { module } => {
    let qualified = format!("{module}::{}", callable.exported_name);
    let call_module = if builtin_dispatch_table().contains_key(qualified.as_str()) {
        Some(module.clone())
    } else {
        None
    };
    let param_exprs = ...; // Variable exprs for each param
    Expr::Call {
        func: callable.exported_name.clone(),
        module: call_module,
        arguments: param_exprs,
    }
}
```

Result:
- `string::concat` → `"string::concat"` in table → `module: Some("string")` → correct dispatch
- `record::keys` → `"record::keys"` NOT in table → `module: None` → dispatches to unqualified `"keys"` → correct dispatch

No new dispatch table entries are needed.

---

## TASK-623: `std/src/string.ash`

### File

**Create `std/src/string.ash`:**

```ash
pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;
```

### Dispatch

All four qualified dispatch entries already exist in `ash-interp`'s `builtin_dispatch_table()`:
`"string::concat"`, `"string::starts_with"`, `"string::ends_with"`, `"string::is_empty"`.

### Discovery

The stdlib root is `crates/ash-engine/../../std/src` (resolved at compile time via `env!("CARGO_MANIFEST_DIR")`). A file `use string::{concat}` will resolve to `std/src/string.ash` via the existing `search_roots` mechanism — no module-loader changes needed beyond the prerequisite fix.

### TDD Steps

1. **Red:** Write integration test importing `string::concat` from `std/src/string.ash` and calling it. Expect failure (file not yet created / module not found).
2. **Green:** Create the file. Implement prerequisite module-name fix so the closure dispatches correctly.
3. **Verify:** Test passes. `use string::{concat, starts_with, ends_with, is_empty}` all resolve and execute.

---

## TASK-626: `std/src/record.ash`

### File

**Create `std/src/record.ash`:**

```ash
pub builtin fn keys(r: Record) -> List<String>;
pub builtin fn values(r: Record) -> List<String>;
pub builtin fn record() -> Record;
```

**Type-signature caveat:** `Record` and `List<String>` may not be cleanly expressible in the current surface parser. During implementation, the exact syntax will be validated. If necessary, fallback to the best parseable form (e.g., placeholder types); strict typechecking integration for these is deferred to Track D1.5/D2 which handle polymorphism.

`record` is variadic at the runtime level (accepts key-value pairs) but declared with 0 parameters; the dispatch table entry for `"record"` has `variadic: true` and handles the variadic case at eval time.

### Dispatch

The dispatch table has unqualified entries `"keys"` (arity 1), `"values"` (arity 1), `"record"` (arity 0, variadic). Because `"record::keys"` is not in the table, `build_imported_closures` generates `module: None` closures, which dispatch to the unqualified implementations.

### TDD Steps

1. **Red:** Write integration test importing `record::keys` from `std/src/record.ash`. Expect failure.
2. **Green:** Create the file. Prerequisite fix already in place.
3. **Verify:** `use record::{keys, values, record}` resolve and execute correctly.

---

## Testing Strategy

### New tests

| File | What it tests |
|------|---------------|
| `crates/ash-engine/tests/string_stdlib_e2e.rs` | Import string builtins, call each, verify results |
| `crates/ash-engine/tests/record_stdlib_e2e.rs` | Import record builtins, call each, verify results |

### Regression baseline

All existing tests (`builtin_fn_e2e_import`, `builtin_dispatch`, `regex_capability`) must remain green.

---

## Files Changed

| File | Change |
|------|--------|
| `std/src/string.ash` | Create |
| `std/src/record.ash` | Create |
| `crates/ash-engine/src/module_loader.rs` | `CallableKind::Builtin { module: String }` + populate at import time |
| `crates/ash-engine/src/lib.rs` | Update `build_imported_closures` for `Builtin { module }` arm |
| `crates/ash-engine/tests/string_stdlib_e2e.rs` | Create |
| `crates/ash-engine/tests/record_stdlib_e2e.rs` | Create |

---

## Out of Scope

- **TASK-631A**: Remove hardcoded `add_builtin_functions` entries in `type_env.rs` (Track F cleanup)
- **Track D1.5**: Type predicate builtins (blocked on ad-hoc polymorphism)
- **Track D2**: Polymorphic list ops (blocked on generic builtin semantics)
- **Track E**: Regex migration
