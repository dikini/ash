//! TASK-717: end-to-end conformance closeout for Phase 98.

use ash_core::runtime::{WorkflowBoundaryOutcome, WorkflowReportStatus};
use ash_core::{Expr, ProcessHandle, Value};
use ash_engine::{Engine, WorkflowAdmissionOutcome, WorkflowAdmissionRequest};
use ash_interp::{ChildEnvProjection, Context, RuntimeState, derive_child_env, eval_expr_async};
use tempfile::TempDir;

fn write_phase98_example(dir: &std::path::Path, relative: &str) -> std::path::PathBuf {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = std::fs::read_to_string(repo_root.join(relative))
        .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
    let path = dir.join(
        std::path::Path::new(relative)
            .file_name()
            .expect("example filename"),
    );
    std::fs::write(&path, source)
        .unwrap_or_else(|err| panic!("failed to write example copy: {err}"));
    path
}

fn workflow_example_path(relative: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(relative)
}

fn process_context(
    runtime_state: RuntimeState,
    process_id: ash_core::runtime::ProcessId,
) -> Context {
    derive_child_env(
        &Context::new().with_runtime_state(runtime_state),
        ChildEnvProjection::new(process_id, 0),
    )
    .expect("proc context projection should succeed")
}

async fn force_proc_value_with_context(
    ctx: Context,
    proc_value: Value,
) -> ash_interp::EvalResult<Value> {
    let mut call_ctx = ctx;
    call_ctx.set("p".to_string(), proc_value);
    eval_expr_async(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        &call_ctx,
    )
    .await
}

async fn force_proc_value(proc_value: Value) -> ash_interp::EvalResult<Value> {
    let runtime_state = RuntimeState::new();
    let root_process_id = ash_core::runtime::ProcessId::new();
    runtime_state
        .register_root_process(root_process_id)
        .await
        .expect("root process registers");

    force_proc_value_with_context(process_context(runtime_state, root_process_id), proc_value).await
}

fn record_field<'a>(fields: &'a std::collections::HashMap<String, Value>, name: &str) -> &'a Value {
    fields
        .get(name)
        .unwrap_or_else(|| panic!("missing field {name:?} in {fields:?}"))
}

fn extract_process_handles(value: Value) -> (ProcessHandle, ProcessHandle) {
    match value {
        Value::Record(fields) => {
            let left = fields
                .get("_0")
                .or_else(|| fields.get("0"))
                .unwrap_or_else(|| panic!("missing first handle field in {fields:?}"));
            let right = fields
                .get("_1")
                .or_else(|| fields.get("1"))
                .unwrap_or_else(|| panic!("missing second handle field in {fields:?}"));
            let Value::ProcessHandle(left) = left.clone() else {
                panic!("expected left process handle, got {left:?}");
            };
            let Value::ProcessHandle(right) = right.clone() else {
                panic!("expected right process handle, got {right:?}");
            };
            (left, right)
        }
        value if value.is_list() => {
            let items = value
                .list_to_vec()
                .expect("is_list only returns true for convertible lists");
            let [left, right]: [Value; 2] = items.try_into().unwrap_or_else(|items: Vec<Value>| {
                panic!("expected two process handles, got {items:?}")
            });
            let Value::ProcessHandle(left) = left else {
                panic!("expected left process handle, got {left:?}");
            };
            let Value::ProcessHandle(right) = right else {
                panic!("expected right process handle, got {right:?}");
            };
            (left, right)
        }
        other => panic!("expected process handles record or list, got {other:?}"),
    }
}

async fn wait_for_terminal_children(runtime_state: &RuntimeState, handles: &[ProcessHandle]) {
    for handle in handles {
        runtime_state
            .wait_for_process_terminal_state(handle.process_id)
            .await
            .unwrap_or_else(|| panic!("missing process record for {:?}", handle.process_id));
    }
}

#[tokio::test]
async fn phase98_fail_with_error_example_runs_end_to_end() {
    let engine = Engine::new().build().expect("engine builds");
    let example = workflow_example_path("05-phase98/01-fail-with-error.ash");

    let mut workflow = engine.parse_file(&example).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");

    assert_eq!(result, Value::Int(7));
}

#[tokio::test]
async fn phase98_proc_par_await_join_example_builds_source_level_observers_that_force_honestly() {
    let temp = TempDir::new().expect("tempdir");
    let main = write_phase98_example(
        temp.path(),
        "examples/05-phase98/02-proc-par-await-join.ash",
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&main).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let proc_value = engine.execute(&workflow).await.expect("execute");

    let runtime_state = RuntimeState::new();
    let root_process_id = ash_core::runtime::ProcessId::new();
    runtime_state
        .register_root_process(root_process_id)
        .await
        .expect("root process registers");
    let proc_ctx = process_context(runtime_state.clone(), root_process_id);

    let forced = force_proc_value_with_context(proc_ctx.clone(), proc_value)
        .await
        .expect("proc example should force successfully");
    let Value::Record(fields) = forced else {
        panic!("expected record payload from proc example, got {forced:?}");
    };

    let (await_left, await_right) =
        extract_process_handles(record_field(&fields, "await_handles").clone());
    let (join_left, join_right) =
        extract_process_handles(record_field(&fields, "join_handles").clone());
    wait_for_terminal_children(&runtime_state, &[await_left.clone(), await_right.clone()]).await;
    wait_for_terminal_children(&runtime_state, &[join_left.clone(), join_right.clone()]).await;

    let awaited = force_proc_value_with_context(
        proc_ctx.clone(),
        record_field(&fields, "await_observer").clone(),
    )
    .await
    .expect("source-level await observer should force after children terminate");
    let Value::Record(await_values) = awaited else {
        panic!("expected await observer record, got {awaited:?}");
    };
    assert_eq!(record_field(&await_values, "left"), &Value::Int(41));
    assert_eq!(record_field(&await_values, "right"), &Value::Int(1));

    let joined =
        force_proc_value_with_context(proc_ctx, record_field(&fields, "join_observer").clone())
            .await
            .expect("source-level join observer should force after children terminate");
    match joined {
        Value::Record(pair) => {
            assert_eq!(
                pair.get("_0").or_else(|| pair.get("0")),
                Some(&Value::Int(41))
            );
            assert_eq!(
                pair.get("_1").or_else(|| pair.get("1")),
                Some(&Value::Int(1))
            );
        }
        value if value.is_list() => {
            let items = value
                .list_to_vec()
                .expect("is_list only returns true for convertible lists");
            assert_eq!(items, vec![Value::Int(41), Value::Int(1)]);
        }
        other => panic!("expected join observer pair payload, got {other:?}"),
    }
}

#[tokio::test]
async fn phase98_proc_scatter_gather_example_forces_to_ordered_values() {
    let temp = TempDir::new().expect("tempdir");
    let main = write_phase98_example(
        temp.path(),
        "examples/05-phase98/03-proc-scatter-gather.ash",
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&main).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let proc_value = engine.execute(&workflow).await.expect("execute");

    let forced = force_proc_value(proc_value)
        .await
        .expect("scatter/gather proc should force successfully");
    assert_eq!(
        forced,
        Value::list_from_vec(vec![Value::Int(2), Value::Int(3), Value::Int(4)])
    );
}

#[tokio::test]
async fn workflow_boundary_reporting_remains_api_level_and_compatibility_wrappers_stay_honest() {
    let temp = TempDir::new().expect("tempdir");
    let main = write_phase98_example(
        temp.path(),
        "examples/05-phase98/04-workflow-boundary-reporting.ash",
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&main).expect("parse");
    engine.check(&mut workflow).expect("typecheck");

    let legacy = engine
        .execute_core_workflow(&workflow.core)
        .await
        .expect("legacy workflow execution should succeed");
    assert_eq!(legacy, Value::Int(9));

    let admitted = engine
        .admit_workflow(WorkflowAdmissionRequest {
            workflow_name: "task_717_boundary".to_string(),
            workflow: workflow.core.clone(),
            workflow_id: None,
            run_id: None,
            active_role: None,
            admitted_role: None,
            required_capabilities: vec![],
            requires: vec![],
            ensures: vec![],
        })
        .await;

    match admitted {
        WorkflowAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            WorkflowBoundaryOutcome::WorkflowSucceeded { value, report } => {
                assert_eq!(value, &legacy);
                assert_eq!(report.result.as_ref(), Some(&legacy));
                assert_eq!(report.status, WorkflowReportStatus::Succeeded);
            }
            other @ WorkflowBoundaryOutcome::WorkflowFailed { .. } => {
                panic!("expected succeeded workflow boundary outcome, got {other:?}")
            }
        },
        other @ WorkflowAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted workflow boundary outcome, got {other:?}")
        }
    }

    let readme =
        std::fs::read_to_string(workflow_example_path("README.md")).expect("read examples README");
    assert!(
        readme.contains("workflow boundary reporting currently requires the engine admission API"),
        "cross-layer docs must state the honest boundary-reporting limitation"
    );
    assert!(
        readme.contains("04-workflow-boundary-reporting.ash"),
        "examples README must reference the boundary-reporting source file"
    );
}
