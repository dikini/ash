//! Module resolution integration tests.
//!
//! Tests verify that the module resolver correctly discovers and loads modules
//! from the local file tree, `ASH_LIBRARY_PATH`, and the built-in stdlib root,
//! with proper search precedence, cycle detection, and error reporting.

use ash_core::{Decision, Expr, Provenance, Value};
use ash_engine::Engine;
use ash_interp::act_env::ActEnv;
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::error::EvalError;
use ash_interp::eval::eval_expr;
use ash_interp::policy::{Policy, PolicyEvaluator};
use tempfile::TempDir;

/// Helper: write `contents` to `path`, creating parent directories as needed.
fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

/// Helper: build a default engine.
fn build_engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

/// Helper: attempt to force a returned Act value with a visible dummy argument.
/// Under honest hidden-ActEnv threading, arbitrary user-visible values should not
/// be accepted as the runtime carrier.
fn force_with_dummy_arg(value: Value) -> Result<Value, EvalError> {
    let mut ctx = Context::new();
    ctx.set("act".to_string(), value);
    let expr = Expr::Call {
        func: "act".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::Int(0))],
    };
    eval_expr(&expr, &ctx)
}

// ── 1. Local sibling module resolution ──────────────────────────────────

/// Two files sit next to each other in the same directory.
/// `main.ash` imports a type from `sibling.ash`.
///
/// Expected: the resolver locates `sibling.ash` relative to the entry file
/// root and the imported type is available for construction.
#[tokio::test]
async fn sibling_module_type_import_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("sibling.ash"),
        "pub type Point = Point { x: Int, y: Int };",
    );
    write(
        &dir.join("main.ash"),
        "\
        use sibling::{Point}\n\
        \n\
        workflow main() -> Point { ret Point { x: 1, y: 2 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    // Validates that the legacy loader resolves this today.
    assert!(
        result.is_ok(),
        "sibling module resolution: expected successful execution, got: {:?}",
        result.err()
    );
    let value = result.expect("checked above");
    assert!(
        matches!(
            &value,
            ash_core::Value::Variant { name, fields }
                if name == "Point"
                    && fields.len() == 2
                    && fields[0].0 == "x" && fields[0].1 == ash_core::Value::Int(1)
                    && fields[1].0 == "y" && fields[1].1 == ash_core::Value::Int(2)
        ),
        "expected Point {{ x: 1, y: 2 }}, got {value:?}",
    );
}

#[tokio::test]
async fn multiline_nested_ordinary_import_resolves_before_workflow_body() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("types.ash"),
        "\
        pub type A = A { value: Int };\n\
        pub type B = B { label: String };\n\
        ",
    );
    write(
        &dir.join("main.ash"),
        "\
        use types::{\n\
            A,\n\
            B\n\
        };\n\
        \n\
        workflow main() -> A { ret A { value: 7 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "multiline ordinary import should resolve before workflow parsing, got: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn semicolonless_ordinary_imports_do_not_swallow_workflow_body() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(&dir.join("types.ash"), "pub type A = A { value: Int };");
    write(&dir.join("extra.ash"), "pub type B = B { label: String };");
    write(
        &dir.join("main.ash"),
        "\
        use types::A\n\
        use extra::B\n\
        \n\
        workflow main() -> A { ret A { value: 7 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "semicolonless ordinary imports should remain line-delimited, got: {:?}",
        result.err()
    );
}

// ── 2. Nested multi-file modules ───────────────────────────────────────

/// Project layout:
///
/// ```text
/// main.ash
/// foo/mod.ash       -- declares `pub mod bar;`
/// foo/bar.ash       -- defines `pub type Inner`
/// ```
///
/// Expected: `main.ash` can import `foo::bar::Inner` through the nested
/// module tree.  The resolver must follow `pub mod` declarations and load
/// child modules recursively.
#[tokio::test]
async fn nested_pub_mod_module_resolution() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("foo").join("bar.ash"),
        "pub type Inner = Inner { val: Int };",
    );
    write(&dir.join("foo").join("mod.ash"), "pub mod bar;");
    write(
        &dir.join("main.ash"),
        "\
        use foo::bar::{Inner}\n\
        \n\
        workflow main() -> Inner { ret Inner { val: 99 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "nested module resolution: expected successful execution, got: {:?}",
        result.err()
    );
}

// ── 3. Stdlib import resolution ────────────────────────────────────────

/// `main.ash` imports a type from the standard library (e.g. `option`).
///
/// Expected: the resolver falls through to the builtin stdlib root and
/// resolves the module without the user specifying a path.
#[tokio::test]
async fn stdlib_module_resolution() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use option::{Option, Some}\n\
        \n\
        workflow main() -> Option<Int> { ret Some { value: 42 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib module resolution: expected successful execution, got: {:?}",
        result.err()
    );
    let value = result.expect("checked above");
    assert!(
        matches!(
            &value,
            ash_core::Value::Variant { name, .. } if name == "Some"
        ),
        "expected Some variant, got {value:?}",
    );
}

/// `std::act` should expose ordinary helper signatures over an opaque `Act`
/// identity rather than exporting `unit` as a public builtin.
#[test]
fn stdlib_act_helpers_import_as_ordinary_functions_over_opaque_act() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use act::{Act, unit, bind, then, guard}\n\
        \n\
        workflow main(x: Act<Int>) -> Act<Int> { ret x }\n\
        ",
    );

    let engine = build_engine();
    let workflow = engine
        .parse_file(dir.join("main.ash"))
        .expect("opaque act helpers should import cleanly");

    for name in ["unit", "bind", "then", "guard"] {
        assert!(
            workflow.imported_fn_signatures.contains_key(name),
            "expected ordinary helper signature for {name}, found {:?}",
            workflow.imported_fn_signatures.keys().collect::<Vec<_>>()
        );
        assert!(
            !workflow.imported_builtin_signatures.contains_key(name),
            "{name} should no longer be imported as a public builtin signature"
        );
    }
}

/// `std::act::unit` should now be exposed through an ordinary helper while
/// still evaluating to the opaque runtime `Act` value.
#[tokio::test]
async fn stdlib_act_unit_returns_opaque_value_via_ordinary_helper() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use act::{Act, unit}\n\
        \n\
        workflow main() -> Act<Int> { ret unit(1); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib act ordinary helper: expected successful execution, got: {:?}",
        result.err()
    );
    assert!(
        matches!(
            result.expect("checked above"),
            ash_core::Value::Closure { .. }
        ),
        "ordinary std::act unit should still return an opaque closure-shaped Act value"
    );
}

/// If `Act` really threads a hidden `ActEnv` parameter, forcing the returned
/// value with zero arguments should reject with `WrongArity` instead of behaving
/// like a zero-arg closure.
#[tokio::test]
async fn stdlib_act_unit_zero_arg_force_requires_hidden_actenv() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use act::{Act, unit}\n\
        \n\
        workflow main() -> Act<Int> { ret unit(1); }\n\
        ",
    );

    let engine = build_engine();
    let act_value = engine
        .run_file(dir.join("main.ash"))
        .await
        .expect("unit should produce an Act value");

    let forced = force_with_dummy_arg(act_value);
    assert!(
        forced.is_err(),
        "A-path runtime contract should reject arbitrary visible arguments as a fake ActEnv carrier, got {forced:?}"
    );
}

/// Sequenced helpers should preserve the same hidden-ActEnv requirement when a
/// composed Act value is forced.
#[tokio::test]
async fn stdlib_act_then_dummy_arg_force_rejects_fake_actenv() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use act::{Act, unit, then}\n\
        \n\
        workflow main() -> Act<Int> { ret then(unit(1), unit(2)); }\n\
        ",
    );

    let engine = build_engine();
    let act_value = engine
        .run_file(dir.join("main.ash"))
        .await
        .expect("then should produce an Act value");

    let forced = force_with_dummy_arg(act_value);
    assert!(
        forced.is_err(),
        "sequenced Act values should also reject arbitrary visible arguments as fake ActEnv carriers, got {forced:?}"
    );
}
fn force_act_with_policy(
    value: Value,
    policy_name: &str,
    decision: Decision,
) -> Result<Value, EvalError> {
    let mut policies = PolicyEvaluator::new();
    policies.register(Policy::new(policy_name).with_default(decision));
    let act_env = ActEnv::new(
        CapabilityContext::new(),
        policies.clone(),
        Provenance::new(),
    );
    let ctx = Context::new()
        .with_policy_evaluator(policies)
        .with_act_env(act_env);

    eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::Literal(value)),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &ctx,
    )
}

/// `std::act::Policy` should remain coherent with policy-facing helpers when
/// imported across the stdlib module boundary.
#[tokio::test]
async fn stdlib_act_policy_alias_can_flow_into_guard() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        r"
        use act::{Act, Policy, guard, unit}

        workflow main(p: Policy) -> Act<Int> { ret guard(p, unit(1)); }
        ",
    );

    let engine = build_engine();
    let mut workflow = engine
        .parse_file(dir.join("main.ash"))
        .expect("imported Policy alias should parse cleanly");

    engine
        .check(&mut workflow)
        .expect("imported Policy values should type-check when passed to guard");
}

/// `std::act::guard` should import as an ordinary helper with its opaque `Act`
/// signature and execute policy-allowed actions through the library boundary.
#[tokio::test]
async fn stdlib_act_guard_permits_allowed_policy() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        r#"
        use act::{Act, guard, unit}

        workflow main() -> Act<Int> { ret guard("allowed", unit(7)) }
        "#,
    );

    let engine = build_engine();
    let act_value = engine
        .run_file(dir.join("main.ash"))
        .await
        .expect("guard over allowed policy should produce an Act value");

    let forced = force_act_with_policy(act_value, "allowed", Decision::Permit)
        .expect("allowed guard should force successfully");
    assert_eq!(
        forced,
        Value::List(Box::new(vec![Value::ActEnvToken, Value::Int(7)]))
    );
}

/// Denied policies should execute `std::act::guard`'s ordinary-library failure path
/// rather than bypassing the helper through a public runtime builtin.
#[tokio::test]
async fn stdlib_act_guard_denies_rejected_policy() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        r#"
        use act::{Act, guard, unit}

        workflow main() -> Act<Int> { ret guard("denied", unit(7)) }
        "#,
    );

    let engine = build_engine();
    let act_value = engine
        .run_file(dir.join("main.ash"))
        .await
        .expect("guard over denied policy should still produce an Act value");

    let forced = force_act_with_policy(act_value, "denied", Decision::Deny)
        .expect("denied guard should force its failure Act");
    assert_eq!(
        forced,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("policy denied".to_string())
        ]))
    );
}

/// `main.ash` can import the Phase 97 Act stdlib helpers from the builtin
/// stdlib root.
#[tokio::test]
async fn stdlib_act_module_resolution() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use act::{unit, bind, then, guard}\n\
        \n\
        workflow main() -> Int { ret 1; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib act module resolution: expected successful execution, got: {:?}",
        result.err()
    );
}

// ── 4. Missing module error ────────────────────────────────────────────

/// `main.ash` imports from a module `nonexistent` that does not exist on
/// disk or in the stdlib.
///
/// Expected: the resolver returns a clear, actionable error indicating
/// that the module was not found and where it searched.
#[tokio::test]
async fn missing_module_produces_clear_error() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use nonexistent::{Foo}\n\
        \n\
        workflow main() -> Int { ret 1; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(result.is_err(), "missing module should produce an error");
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("not found") || msg.contains("module"),
        "error should mention the missing module, got: {msg}",
    );
}

// ── 5. Circular import detection ───────────────────────────────────────

/// Two modules import each other:
///
/// - `a.ash` does `use b::{X}`
/// - `b.ash` does `use a::{Y}`
///
/// Expected: the resolver detects the cycle and returns a deterministic
/// error rather than recursing indefinitely or panicking.
#[tokio::test]
async fn circular_import_detection() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("a.ash"),
        "\
        use b::{X}\n\
        \n\
        pub type Y = Y { v: Int };\n\
        \n\
        workflow main() -> Int { ret 1; }\n\
        ",
    );
    write(
        &dir.join("b.ash"),
        "\
        use a::{Y}\n\
        \n\
        pub type X = X { v: Int };\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("a.ash")).await;

    // The ModuleResolver must detect circular imports and report them
    // with a clear "cyclic" or "circular" message.
    match result {
        Err(ref e) => {
            let msg = format!("{e}");
            assert!(
                msg.to_lowercase().contains("cyclic") || msg.to_lowercase().contains("circular"),
                "circular import should report a cycle, got: {msg}",
            );
        }
        Ok(_) => {
            panic!(
                "circular import should be detected and reported as an error, \
                 but execution succeeded"
            );
        }
    }
}

// ── 6. ASH_LIBRARY_PATH search order ───────────────────────────────────

/// Verify that the resolver searches paths in the following order:
///
///   1. Entry file root tree (local files)
///   2. Directories listed in `ASH_LIBRARY_PATH`
///   3. Built-in stdlib root
///
/// A module with the same name exists both locally and in the library path;
/// the local one must win.
#[tokio::test]
async fn ash_library_path_search_order_local_wins_over_lib() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    // Local module -- value 10
    write(
        &dir.join("mymod.ash"),
        "pub type LocalResult = LocalResult { val: Int };",
    );

    // Library-path module -- value 20 (should NOT be used)
    let lib_dir = temp.path().join("libs");
    write(
        &lib_dir.join("mymod.ash"),
        "pub type LocalResult = LocalResult { val: Int };",
    );

    write(
        &dir.join("main.ash"),
        "\
        use mymod::{LocalResult}\n\
        \n\
        workflow main() -> LocalResult { ret LocalResult { val: 10 }; }\
        ",
    );

    let lib_dir_clone = lib_dir.clone();
    let result = temp_env::async_with_vars(
        [("ASH_LIBRARY_PATH", Some(lib_dir_clone.as_os_str()))],
        async {
            let engine = build_engine();
            engine.run_file(dir.join("main.ash")).await
        },
    )
    .await;

    assert!(
        result.is_ok(),
        "ASH_LIBRARY_PATH search order: local should win, got: {:?}",
        result.err()
    );
}

/// When no local module exists, the resolver falls through to
/// `ASH_LIBRARY_PATH`.
#[tokio::test]
async fn ash_library_path_falls_through_to_lib_dir() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();
    let lib_dir = temp.path().join("libs");

    // Only in library path -- NOT local.
    write(
        &lib_dir.join("extmod.ash"),
        "pub type ExtVal = ExtVal { n: Int };",
    );

    write(
        &dir.join("main.ash"),
        "\
        use extmod::{ExtVal}\n\
        \n\
        workflow main() -> ExtVal { ret ExtVal { n: 5 }; }\n\
        ",
    );

    let lib_dir_clone = lib_dir.clone();
    let result = temp_env::async_with_vars(
        [("ASH_LIBRARY_PATH", Some(lib_dir_clone.as_os_str()))],
        async {
            let engine = build_engine();
            engine.run_file(dir.join("main.ash")).await
        },
    )
    .await;

    assert!(
        result.is_ok(),
        "ASH_LIBRARY_PATH fallback: should resolve from lib dir, got: {:?}",
        result.err()
    );
}

/// When neither a local module nor an `ASH_LIBRARY_PATH` entry exists, the
/// resolver should fall through to the builtin stdlib root.
#[tokio::test]
async fn ash_library_path_falls_through_to_stdlib() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use option::{Option, Some}\n\
        \n\
        workflow main() -> Option<Int> { ret Some { value: 1 }; }\n\
        ",
    );

    let result =
        temp_env::async_with_vars([("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>)], async {
            let engine = build_engine();
            engine.run_file(dir.join("main.ash")).await
        })
        .await;

    assert!(
        result.is_ok(),
        "stdlib fallback: should resolve `option` from builtin root, got: {:?}",
        result.err()
    );
}

// ── 7. Comprehensive stdlib resolution ─────────────────────────────────

/// Verify that multiple stdlib modules resolve together: `result` type,
/// `string` builtin functions, and `list` builtin functions all work
/// when imported through the resolver.
#[tokio::test]
async fn stdlib_result_type_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use result::{Result, Ok, Err}\n\
        \n\
        workflow main() -> Result<Int, String> { ret Ok { value: 42 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib result resolution: expected successful execution, got: {:?}",
        result.err()
    );
    let value = result.expect("checked above");
    assert!(
        matches!(
            &value,
            ash_core::Value::Variant { name, .. } if name == "Ok"
        ),
        "expected Ok variant, got {value:?}",
    );
}

/// Verify that `string` builtin functions resolve through the stdlib root.
#[tokio::test]
async fn stdlib_string_builtin_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use string::{concat}\n\
        \n\
        workflow main() -> String { ret concat(\"hello\", \" world\"); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib string builtin: expected successful execution, got: {:?}",
        result.err()
    );
    assert_eq!(
        result.expect("checked above"),
        ash_core::Value::String("hello world".to_string()),
    );
}

/// Verify that `list` builtin functions resolve through the stdlib root.
#[tokio::test]
async fn stdlib_list_builtin_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use list::{len}\n\
        \n\
        workflow main() -> Int { ret len([1, 2, 3]); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib list builtin: expected successful execution, got: {:?}",
        result.err()
    );
    assert_eq!(result.expect("checked above"), ash_core::Value::Int(3));
}

/// Verify that `predicate` builtin functions resolve through the stdlib root.
#[tokio::test]
async fn stdlib_predicate_builtin_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use predicate::{is_int}\n\
        \n\
        workflow main() -> Bool { ret is_int(42); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib predicate builtin: expected successful execution, got: {:?}",
        result.err()
    );
    assert_eq!(result.expect("checked above"), ash_core::Value::Bool(true));
}

/// A public callable must not expose a private ordinary type in its signature;
/// Phase 109 imports type identities through semantic summaries rather than
/// allowing private ordinary type leakage through public APIs.
#[test]
fn public_callable_private_plain_type_signature_is_rejected() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("lib.ash"),
        "\
        type Hidden = Hidden { value: Int };\n\
        pub builtin fn passthrough(x: Hidden) -> Hidden;\n\
        ",
    );
    write(
        &dir.join("main.ash"),
        "\
        use lib::{passthrough}\n\
        workflow main { ret 0 }\n\
        ",
    );

    let engine = build_engine();
    let err = engine
        .parse_file(dir.join("main.ash"))
        .expect_err("private ordinary type leak should be rejected at module boundary");
    assert!(
        err.to_string().contains("public callable 'passthrough'")
            && err.to_string().contains("private ordinary type 'Hidden'"),
        "diagnostic should identify the public callable private type leak: {err}"
    );
}

/// A plain `type` should not automatically expose its representation for
/// construction in importing modules.
#[tokio::test]
async fn plain_type_does_not_export_constructor_representation() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("lib.ash"),
        "\
        type Hidden = Hidden { value: Int };\n\
        pub builtin fn passthrough(x: Hidden) -> Hidden;\n\
        ",
    );
    write(
        &dir.join("main.ash"),
        "\
        use lib::{Hidden}\n\
        workflow main() -> Hidden { ret Hidden { value: 1 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_err(),
        "plain type constructor should remain hidden"
    );
}

/// A `pub type` should continue to expose its representation for importing
/// modules.
#[tokio::test]
async fn pub_type_still_exports_constructor_representation() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("lib.ash"),
        "pub type Visible = Visible { value: Int };\n",
    );
    write(
        &dir.join("main.ash"),
        "\
        use lib::{Visible}\n\
        workflow main() -> Visible { ret Visible { value: 1 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "pub type constructor should remain visible, got: {:?}",
        result.err()
    );
}
