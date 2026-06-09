#![allow(unused_imports)]

pub use ash_core::{Expr, Value};
pub use ash_interp::context::Context;
pub use ash_interp::error::EvalError;
pub use ash_interp::eval::{
    BuiltinEntry, builtin_dispatch_table, dispatch_builtin, eval_expr, is_known_builtin,
};
pub use std::path::{Path, PathBuf};

pub fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path
}

pub fn stdlib_root() -> PathBuf {
    workspace_root().join("std/src")
}

pub fn builtin_module_prefix(path: &Path) -> String {
    let root = stdlib_root();
    let relative = path.strip_prefix(&root).unwrap_or(path);
    let without_ext = relative.with_extension("");
    let mut parts: Vec<String> = without_ext
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();
    if parts.last().is_some_and(|part| part == "mod") {
        parts.pop();
    }
    parts.join("::")
}

pub fn collect_stdlib_builtin_declarations(dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let entry = entry.expect("stdlib dir entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_stdlib_builtin_declarations(&path, out);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "ash") {
            continue;
        }

        let module = builtin_module_prefix(&path);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for line in source.lines() {
            let Some(rest) = line.trim_start().strip_prefix("pub builtin fn ") else {
                continue;
            };
            let Some(before_args) = rest.split('(').next() else {
                continue;
            };
            let name = before_args.split('<').next().unwrap_or(before_args).trim();
            out.push(format!("{module}::{name}"));
        }
    }
}

// ── TASK-621: Dispatch table structure ────────────────────────────

pub fn assert_process_run_record(result: Value, stdout: &str, stderr: &str, exit_code: i64) {
    match result {
        Value::Record(fields) => {
            assert_eq!(
                fields.get("stdout"),
                Some(&Value::String(stdout.to_string()))
            );
            assert_eq!(
                fields.get("stderr"),
                Some(&Value::String(stderr.to_string()))
            );
            assert_eq!(fields.get("exit_code"), Some(&Value::Int(exit_code)));
        }
        other => panic!("expected process::run result record, got {other:?}"),
    }
}
