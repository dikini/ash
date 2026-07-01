//! End-to-end integration test for pipe operator with statement lifting.
//!
//! This test verifies the full pipeline: parse -> lower -> lift -> typecheck -> execute
//! for workflows using the pipe operator syntax.

use std::sync::Arc;

use ash_core::{Effect, EnvFrame, Expr, Value};
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::execute::execute_workflow_with_behaviour_in_state;
use ash_interp::policy::PolicyEvaluator;
use ash_interp::runtime_state::RuntimeState;
use ash_parser::input::new_input;
use ash_parser::lower::lower_workflow;
use ash_parser::parse_workflow::workflow_def;
use ash_typeck::type_check_workflow_def_in_env;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;

/// Creates a mock `read_dir` closure that returns a fixed list of files.
/// This simulates reading a directory without performing actual I/O.
fn mock_read_dir_closure() -> Value {
    Value::Closure {
        params: vec![("_path".to_string(), None)],
        body: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::String("readme.md".to_string()),
            Value::String("main.rs".to_string()),
            Value::String("docs".to_string()),
            Value::String("guide.md".to_string()),
            Value::String("Cargo.toml".to_string()),
        ]))),
        env: Arc::new(EnvFrame::new()),
    }
}

/// Sets up the type environment with builtin function types for the test.
fn setup_type_env() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();

    // Bind filter: Fun(List<String>, Fn(String) -> Bool) -> List<String>.
    // SPEC-072 pure closures typecheck as Type::Fn even inside workflows.
    let predicate_type = Type::Fn(vec![Type::String], Box::new(Type::Bool));
    let filter_type = Type::Fun(
        vec![Type::List(Box::new(Type::String)), predicate_type.clone()],
        Box::new(Type::List(Box::new(Type::String))),
        Effect::Operational,
    );
    env.bind_variable("filter", filter_type);

    // Bind ends_with: Fun(String, String) -> Bool. SPEC-072 callable
    // application is exact-arity, so callers use an explicit closure when a
    // unary predicate is needed by filter.
    let ends_with_type = Type::Fun(
        vec![Type::String, Type::String],
        Box::new(Type::Bool),
        Effect::Operational,
    );
    env.bind_variable("ends_with", ends_with_type);

    // Bind read_dir: Fun(String) -> List<String> (impure function - does I/O)
    let read_dir_type = Type::Fun(
        vec![Type::String],
        Box::new(Type::List(Box::new(Type::String))),
        Effect::Operational,
    );
    env.bind_variable("read_dir", read_dir_type);

    env
}

/// End-to-end test: parse, lower, lift, typecheck, and execute a workflow
/// that uses the pipe operator to filter .md files from a directory.
///
/// The workflow under test:
/// ```
/// workflow main(path: String) -> List<String> {
///     let md_files = read_dir(path) |> filter(|file| -> ends_with(file, ".md"));
///     ret md_files
/// }
/// ```
#[tokio::test]
async fn pipe_operator_e2e_read_dir_filter_md_files() {
    let source = r#"workflow main(path: String) -> List<String> { let md_files = read_dir(path) |> filter(|file| -> ends_with(file, ".md")); ret md_files }"#;

    // Step 1: Parse
    let mut input = new_input(source);
    let parsed = workflow_def(&mut input).expect("workflow should parse");

    // Step 2 & 3: Lower (which internally calls lift_workflow)
    let lowered = lower_workflow(&parsed).expect("lowering should succeed");

    // Step 4: Typecheck with builtin bindings
    let env = setup_type_env();
    let type_result = type_check_workflow_def_in_env(&env, &parsed);
    assert!(
        type_result.is_ok(),
        "typecheck failed: {:?}",
        type_result.err()
    );

    // Step 5: Execute with mock read_dir and path parameter in context
    let mut ctx = Context::new();
    ctx.set("read_dir".to_string(), mock_read_dir_closure());
    ctx.set("path".to_string(), Value::String("/test/dir".to_string()));

    let runtime_state = RuntimeState::new();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    let result = execute_workflow_with_behaviour_in_state(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("execution should succeed");

    // Verify: only .md files should remain
    let files = result
        .list_to_vec()
        .unwrap_or_else(|| panic!("expected List, got {:?}", result));
    assert_eq!(files.len(), 2, "expected 2 .md files, got {:?}", files);
    assert_eq!(files[0], Value::String("readme.md".to_string()));
    assert_eq!(files[1], Value::String("guide.md".to_string()));
}

/// Test that pipe operator examples use explicit closures rather than
/// implicit partial application when a unary predicate is needed.
#[tokio::test]
async fn pipe_operator_explicit_closure_ordering() {
    // Test starts_with in a similar pattern.
    let source = r#"workflow main(paths: List<String>) -> List<String> { let rs_files = paths |> filter(|path| -> starts_with(path, "src/")); ret rs_files }"#;

    let mut input = new_input(source);
    let parsed = workflow_def(&mut input).expect("workflow should parse");

    // Set up env with starts_with
    let mut env = TypeEnv::with_builtin_types();
    let predicate_type = Type::Fn(vec![Type::String], Box::new(Type::Bool));
    let filter_type = Type::Fun(
        vec![Type::List(Box::new(Type::String)), predicate_type],
        Box::new(Type::List(Box::new(Type::String))),
        Effect::Operational,
    );
    env.bind_variable("filter", filter_type);
    let starts_with_type = Type::Fun(
        vec![Type::String, Type::String],
        Box::new(Type::Bool),
        Effect::Operational,
    );
    env.bind_variable("starts_with", starts_with_type);

    let type_result = type_check_workflow_def_in_env(&env, &parsed);
    assert!(
        type_result.is_ok(),
        "typecheck failed: {:?}",
        type_result.err()
    );

    let lowered = lower_workflow(&parsed).expect("lowering should succeed");

    // Execute with mock paths
    let mut ctx = Context::new();
    ctx.set(
        "paths".to_string(),
        Value::list_from_vec(vec![
            Value::String("src/main.rs".to_string()),
            Value::String("tests/test.rs".to_string()),
            Value::String("src/lib.rs".to_string()),
            Value::String("Cargo.toml".to_string()),
        ]),
    );

    let runtime_state = RuntimeState::new();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    let result = execute_workflow_with_behaviour_in_state(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("execution should succeed");

    let files = result
        .list_to_vec()
        .unwrap_or_else(|| panic!("expected List, got {:?}", result));
    assert_eq!(files.len(), 2, "expected 2 src/ files, got {:?}", files);
    assert_eq!(files[0], Value::String("src/main.rs".to_string()));
    assert_eq!(files[1], Value::String("src/lib.rs".to_string()));
}
