# Track D1: Stdlib Monomorphic Builtin Migration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `std/src/string.ash` and `std/src/record.ash` with `pub builtin fn` declarations, importable via the standard module system (`use string::{concat}`), executing through the existing runtime dispatch table.

**Architecture:** Thread `module: String` through `CallableKind::Builtin` so `build_imported_closures` can emit qualified vs. unqualified `Expr::Call` correctly — string ops need `module: Some("string")` to hit `"string::concat"` in the dispatch table, record ops need `module: None` to hit unqualified `"keys"`. The dispatch table already has all needed entries; no new runtime entries required.

**Tech Stack:** Rust, `ash-engine` (module_loader + lib.rs), `ash-interp` dispatch table, `.ash` surface syntax for builtin fn declarations.

---

## File Map

| File | Change |
|------|--------|
| `crates/ash-engine/src/module_loader.rs` | Add `module: String` to `CallableKind::Builtin`; populate at import time in `load_ordinary_file` |
| `crates/ash-engine/src/lib.rs` | Update `build_imported_closures` `Builtin` arm: dispatch-table check to pick qualified vs. unqualified |
| `crates/ash-engine/tests/builtin_fn_e2e_import.rs` | Fix two `matches!(…, CallableKind::Builtin)` patterns to `CallableKind::Builtin { .. }` |
| `std/src/string.ash` | Create: four `pub builtin fn` declarations |
| `std/src/record.ash` | Create: three `pub builtin fn` declarations |
| `crates/ash-engine/tests/string_stdlib_e2e.rs` | Create: e2e tests for string stdlib imports |
| `crates/ash-engine/tests/record_stdlib_e2e.rs` | Create: e2e tests for record stdlib imports |

---

### Task 1: Add `module: String` to `CallableKind::Builtin` (data model)

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs:41-48`

- [ ] **Step 1: Write the failing test**

Add this test to `crates/ash-engine/src/module_loader.rs` inside the existing `#[cfg(test)] mod tests` block (near line 1282):

```rust
#[test]
fn builtin_fn_callable_kind_carries_module_name() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(dir.join("string.ash"), "pub builtin fn concat(a: String, b: String) -> String;\n")
        .expect("write");
    std::fs::write(dir.join("caller.ash"), "use string::{concat}\nworkflow main { ret 0 }\n")
        .expect("write");

    let result = super::load_ordinary_file(&dir.join("caller.ash")).expect("load");
    let callable = result.imported_callables.get("concat").expect("concat callable");
    match &callable.kind {
        super::CallableKind::Builtin { module } => {
            assert_eq!(module, "string", "module name should be populated from import path");
        }
        other => panic!("expected Builtin {{ module }}, got: {:?}", other),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test -p ash-engine builtin_fn_callable_kind_carries_module_name 2>&1 | tail -20
```

Expected: compile error — `CallableKind::Builtin` has no `module` field yet.

- [ ] **Step 3: Add `module: String` field to `CallableKind::Builtin`**

In `crates/ash-engine/src/module_loader.rs`, change the enum (lines 41–48):

```rust
/// Whether a callable carries an Ash-level body or is bodyless (builtin).
#[derive(Debug, Clone)]
pub enum CallableKind {
    /// User-defined callable with an Ash expression body.
    User {
        /// The Ash expression constituting the callable body.
        body: Expr,
    },
    /// Bodyless builtin function resolved at link time.
    /// `module` is the module path joined by `::` (e.g. `"string"`, `"record"`).
    Builtin { module: String },
}
```

- [ ] **Step 4: Fix `parse_builtin_fn_callable` — populate module from a placeholder**

`parse_builtin_fn_callable` doesn't know the module name; it receives only the snippet. Change it to accept a `module: String` parameter and pass it through.

In `module_loader.rs`, update the function signature and the `kind` line (around line 776):

```rust
fn parse_builtin_fn_callable(
    snippet: &str,
    module: String,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    // … (rest unchanged) …
    Ok(Some(ImportedCallableExport {
        name: name.clone(),
        callable: InlineCallable {
            exported_name: name,
            params,
            kind: CallableKind::Builtin { module },
        },
    }))
}
```

- [ ] **Step 5: Fix all call sites of `parse_builtin_fn_callable`**

Search for all uses:

```
grep -n "parse_builtin_fn_callable" crates/ash-engine/src/module_loader.rs
```

For each call site, pass `String::new()` as a temporary placeholder for `module`. (This keeps the code compiling; Task 2 will populate the real value.)

- [ ] **Step 6: Fix existing `CallableKind::Builtin` patterns**

Two test patterns in `builtin_fn_e2e_import.rs` match `CallableKind::Builtin` (lines 59 and 285). Update both:

```rust
// line 59 — was: CallableKind::Builtin
ash_engine::module_loader::CallableKind::Builtin { .. }

// line 285 — was: CallableKind::Builtin
ash_engine::module_loader::CallableKind::Builtin { .. }
```

Also fix the internal module_loader test at line 1285:

```rust
// was: matches!(triple.kind, CallableKind::Builtin)
matches!(triple.kind, CallableKind::Builtin { .. })
```

- [ ] **Step 7: Run all tests to verify compile + new test fails for the right reason**

```
cargo test -p ash-engine 2>&1 | tail -30
```

Expected: the new test `builtin_fn_callable_kind_carries_module_name` fails with `module == ""` (empty placeholder). All other tests pass.

- [ ] **Step 8: Commit**

```bash
git add crates/ash-engine/src/module_loader.rs crates/ash-engine/tests/builtin_fn_e2e_import.rs
git commit -m "feat(ash-engine): add module field to CallableKind::Builtin"
```

---

### Task 2: Populate `module` at import resolution time

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs` — `load_ordinary_file`, `collect_module_exports`, or the call chain where `parse_builtin_fn_callable` is invoked

The key issue: `parse_builtin_fn_callable` is called inside `collect_module_exports`, which doesn't know the import module path. The module name is available in `load_ordinary_file` where `import.module_segments` lives.

The cleanest fix: propagate module name through the callables after they are collected — set `callable.kind.module` in the `Glob` and `Named` branches of `load_ordinary_file`, where `import.module_segments` is available.

- [ ] **Step 1: Update `load_ordinary_file` `Glob` branch** (around line 208–211)

```rust
ImportSelection::Glob => {
    imported_type_defs.extend(exports.type_defs.values().cloned());
    let module_name = import.module_segments.join("::");
    for (k, mut v) in exports.callables.clone() {
        if let CallableKind::Builtin { ref mut module } = v.kind {
            *module = module_name.clone();
        }
        imported_callables.insert(k, v);
    }
}
```

- [ ] **Step 2: Update `load_ordinary_file` `Named` branch** (around line 212–225)

```rust
ImportSelection::Named { name, alias } => {
    let exported_name = alias.unwrap_or_else(|| name.clone());
    if let Some(type_def) = exports.type_defs.get(&name) {
        imported_type_defs.push(type_def.clone());
    } else if let Some(callable) = exports.callables.get(&name) {
        let mut callable = callable.clone();
        callable.exported_name.clone_from(&exported_name);
        if let CallableKind::Builtin { ref mut module } = callable.kind {
            *module = import.module_segments.join("::");
        }
        imported_callables.insert(exported_name, callable);
    } else {
        return Err(EngineError::Parse(format!(
            "item '{name}' not found in module '{}'",
            import.module_segments.join("::")
        )));
    }
}
```

- [ ] **Step 3: Run tests**

```
cargo test -p ash-engine 2>&1 | tail -30
```

Expected: `builtin_fn_callable_kind_carries_module_name` now passes. All other tests remain green.

- [ ] **Step 4: Commit**

```bash
git add crates/ash-engine/src/module_loader.rs
git commit -m "feat(ash-engine): populate module name in CallableKind::Builtin at import resolution"
```

---

### Task 3: Update `build_imported_closures` — dispatch-table check for qualified call form

**Files:**
- Modify: `crates/ash-engine/src/lib.rs:1289`

This is the core fix that makes string builtins dispatch correctly. The `Builtin` arm currently emits `module: None`, routing `concat` to LIST concat. After this task it checks the dispatch table.

- [ ] **Step 1: Write the failing test**

Add a new integration test to `crates/ash-engine/tests/builtin_fn_e2e_import.rs`:

```rust
// ---------------------------------------------------------------------------
// Test 7: Builtin fn closure dispatches via correct qualified/unqualified form
// ---------------------------------------------------------------------------

/// Verify that a builtin fn imported from a module named "string" generates
/// a closure whose body uses `module: Some("string")` when the dispatch table
/// has a "string::func" entry, and `module: None` otherwise.
///
/// This test runs the full string::concat path end-to-end to confirm correct
/// dispatch (not list concat).
#[tokio::test]
async fn builtin_fn_string_concat_dispatches_via_qualified_name() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("string.ash"),
        "pub builtin fn concat(a: String, b: String) -> String;\n",
    )
    .expect("write string.ash");

    std::fs::write(
        dir.join("caller.ash"),
        "use string::{concat}\nworkflow main { ret concat(\"hello \", \"world\") }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("caller.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("hello world".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test -p ash-engine builtin_fn_string_concat_dispatches_via_qualified_name 2>&1 | tail -20
```

Expected: test fails — `concat` dispatches to list concat or returns wrong value (not `"hello world"`).

- [ ] **Step 3: Update `build_imported_closures` `Builtin` arm**

In `crates/ash-engine/src/lib.rs`, replace the `CallableKind::Builtin` arm (lines 1289–1311):

```rust
CallableKind::Builtin { module } => {
    // Check the dispatch table to decide whether this builtin needs a
    // qualified call (e.g. "string::concat") or unqualified (e.g. "keys").
    // String builtins live under "string::*" in the table; record builtins
    // live under unqualified names.
    let qualified = format!("{module}::{}", callable.exported_name);
    let call_module =
        if ash_interp::eval::builtin_dispatch_table().contains_key(qualified.as_str()) {
            Some(module.clone())
        } else {
            None
        };
    let param_exprs: Vec<ash_core::Expr> = callable
        .params
        .iter()
        .map(|p| ash_core::Expr::Variable {
            name: p.clone(),
            span: ash_core::ast::Span::default(),
        })
        .collect();
    ash_core::Expr::Call {
        func: callable.exported_name.clone(),
        module: call_module,
        arguments: param_exprs,
    }
}
```

- [ ] **Step 4: Run all tests**

```
cargo test -p ash-engine 2>&1 | tail -30
```

Expected: new test passes. All prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/ash-engine/src/lib.rs crates/ash-engine/tests/builtin_fn_e2e_import.rs
git commit -m "feat(ash-engine): route builtin closures via qualified dispatch for string ops"
```

---

### Task 4: Create `std/src/string.ash` (TASK-623)

**Files:**
- Create: `std/src/string.ash`
- Create: `crates/ash-engine/tests/string_stdlib_e2e.rs`

- [ ] **Step 1: Write the failing test (RED)**

Create `crates/ash-engine/tests/string_stdlib_e2e.rs`:

```rust
//! TASK-623: End-to-end tests for std/src/string.ash stdlib import.

/// Verify that `use string::{concat}` resolves against `std/src/string.ash`
/// and executes correctly.
#[tokio::test]
async fn string_stdlib_concat_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use string::{concat}\nworkflow main { ret concat(\"foo\", \"bar\") }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("foobar".to_string()));
}

#[tokio::test]
async fn string_stdlib_starts_with_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use string::{starts_with}\nworkflow main { ret starts_with(\"hello\", \"he\") }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

#[tokio::test]
async fn string_stdlib_ends_with_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use string::{ends_with}\nworkflow main { ret ends_with(\"hello\", \"lo\") }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

#[tokio::test]
async fn string_stdlib_is_empty_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use string::{is_empty}\nworkflow main { ret is_empty(\"\") }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

/// Verify all four string stdlib functions can be imported together.
#[tokio::test]
async fn string_stdlib_all_four_functions_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use string::{concat, starts_with, ends_with, is_empty}\nworkflow main { ret is_empty(\"\") }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```
cargo test -p ash-engine string_stdlib 2>&1 | tail -20
```

Expected: all five tests fail with module-not-found error for `"string"`.

- [ ] **Step 3: Create `std/src/string.ash` (GREEN)**

```
std/src/string.ash
```

```ash
pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;
```

- [ ] **Step 4: Run tests**

```
cargo test -p ash-engine string_stdlib 2>&1 | tail -20
```

Expected: all five tests pass.

- [ ] **Step 5: Run full test suite for regressions**

```
cargo test -p ash-engine -p ash-interp 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add std/src/string.ash crates/ash-engine/tests/string_stdlib_e2e.rs
git commit -m "feat(std): add string.ash stdlib module with concat/starts_with/ends_with/is_empty (TASK-623)"
```

---

### Task 5: Create `std/src/record.ash` (TASK-626)

**Files:**
- Create: `std/src/record.ash`
- Create: `crates/ash-engine/tests/record_stdlib_e2e.rs`

Note: `keys`, `values`, `record` dispatch via **unqualified** table entries (no `"record::keys"` entry exists). The prerequisite fix in Task 3 handles this automatically: `"record::keys"` not in table → `call_module = None` → unqualified dispatch to `"keys"`.

The type signature caveat from the spec applies: `List<String>` and `Record` may not parse cleanly. The `keys` and `values` return type will be `List<String>` which should parse; `Record` as a parameter type will be tested during implementation. If needed, fall back to omitting type annotations.

- [ ] **Step 1: Write the failing test (RED)**

Create `crates/ash-engine/tests/record_stdlib_e2e.rs`:

```rust
//! TASK-626: End-to-end tests for std/src/record.ash stdlib import.

/// Verify that `use record::{keys}` resolves against `std/src/record.ash`
/// and executes correctly.
#[tokio::test]
async fn record_stdlib_keys_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use record::{keys}\nworkflow main { let r = record(\"a\", 1, \"b\", 2)\nret keys(r) }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    // keys returns a list; just verify it's a non-empty list
    assert!(
        matches!(result, ash_core::Value::List(_)),
        "expected List from keys, got: {result:?}"
    );
}

#[tokio::test]
async fn record_stdlib_values_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use record::{values}\nworkflow main { let r = record(\"a\", 1, \"b\", 2)\nret values(r) }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::List(_)),
        "expected List from values, got: {result:?}"
    );
}

#[tokio::test]
async fn record_stdlib_record_constructor_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use record::{record}\nworkflow main { ret record(\"x\", 42) }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::Record(_)),
        "expected Record from record(), got: {result:?}"
    );
}

/// Verify all three record stdlib functions can be imported together.
#[tokio::test]
async fn record_stdlib_all_three_functions_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use record::{keys, values, record}\nworkflow main { ret record(\"k\", 1) }\n",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::Record(_)),
        "expected Record, got: {result:?}"
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

```
cargo test -p ash-engine record_stdlib 2>&1 | tail -20
```

Expected: all four tests fail with module-not-found for `"record"`.

- [ ] **Step 3: Create `std/src/record.ash` (GREEN)**

Try this form first:

```ash
pub builtin fn keys(r: Record) -> List<String>;
pub builtin fn values(r: Record) -> List<String>;
pub builtin fn record() -> Record;
```

If `Record` or `List<String>` fails to parse, fall back to the minimal parseable form without type annotations for problematic types. Run `cargo test -p ash-engine record_stdlib` after writing the file to determine which form is needed.

- [ ] **Step 4: Run tests**

```
cargo test -p ash-engine record_stdlib 2>&1 | tail -20
```

Expected: all four tests pass.

- [ ] **Step 5: Run full test suite for regressions**

```
cargo test -p ash-engine -p ash-interp 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add std/src/record.ash crates/ash-engine/tests/record_stdlib_e2e.rs
git commit -m "feat(std): add record.ash stdlib module with keys/values/record (TASK-626)"
```

---

### Task 6: Final regression check and CHANGELOG update

**Files:**
- Modify: `CHANGELOG.md` or `docs/PLAN-INDEX.md` per project convention

- [ ] **Step 1: Run complete test suite**

```
cargo test --workspace 2>&1 | tail -40
```

Expected: all tests pass across all crates.

- [ ] **Step 2: Update CHANGELOG**

Check the existing CHANGELOG format:

```
head -40 CHANGELOG.md
```

Add an entry for Track D1 (Tasks 623 and 626) following the existing format. Include:
- `feat(std): string.ash — concat, starts_with, ends_with, is_empty importable via module system`
- `feat(std): record.ash — keys, values, record importable via module system`
- `feat(ash-engine): CallableKind::Builtin carries module name for correct dispatch routing`

- [ ] **Step 3: Update PLAN-INDEX**

Check whether `docs/PLAN-INDEX.md` exists and add a Track D1 entry following the pattern for Track C (TASK-621/622):

```
head -60 docs/PLAN-INDEX.md
```

Mark TASK-623 and TASK-626 as complete.

- [ ] **Step 4: Commit**

```bash
git add CHANGELOG.md docs/PLAN-INDEX.md
git commit -m "docs: update CHANGELOG and PLAN-INDEX for Track D1 completion (TASK-623, TASK-626)"
```

---

## Self-Review

**Spec coverage check:**

| Spec requirement | Covered by |
|-----------------|-----------|
| Prerequisite: `CallableKind::Builtin { module: String }` | Task 1 |
| Populate module at import time (Named + Glob branches) | Task 2 |
| `build_imported_closures` dispatch-table check | Task 3 |
| `std/src/string.ash` with four functions | Task 4 |
| `std/src/record.ash` with three functions | Task 5 |
| `string_stdlib_e2e.rs` tests | Task 4 |
| `record_stdlib_e2e.rs` tests | Task 5 |
| Existing tests remain green | Tasks 3, 4, 5, 6 |
| TASK-631A out of scope | not covered — correct |
| Track D2/D1.5 out of scope | not covered — correct |

**Type consistency check:** `CallableKind::Builtin { module }` field is `String` throughout. `build_imported_closures` clones `module` for `format!`. Pattern `{ .. }` used where field value is not needed. All consistent.

**No placeholders:** No TBD or TODO in implementation steps. Type-annotation caveat in Task 5 Step 3 is explicit with a fallback instruction and a test command to determine which path is needed.
