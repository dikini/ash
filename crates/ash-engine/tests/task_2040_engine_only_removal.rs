//! TASK-2040 removal contracts for the Engine-only execution boundary.
//!
//! The manifest is a frozen entry inventory, so its sole shared-file exception
//! names the exact legacy module declaration to remove. All other TASK-2040
//! delete records require their repository paths to disappear. The property
//! ranges only over the source identities declared by TASK-2035.

use ash_core::Value;
use ash_engine::{CanonicalTerminalEnvelopeV1, Engine};
use proptest::prelude::*;
use serde_json::Value as JsonValue;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const AUDIT_204_MANIFEST: &str =
    include_str!("../../../docs/plan/audits/AUDIT-204-direct-ast-retirement.json");
const DIRECT_AST_CLIENT_CARGO_TOML: &str =
    include_str!("fixtures/task_2040_direct_ast_client/Cargo.toml");
const DIRECT_AST_CLIENT_MAIN_RS: &str =
    include_str!("fixtures/task_2040_direct_ast_client/src/main.rs");
const DIFFERENTIAL_CLIENT_CARGO_TOML: &str =
    include_str!("fixtures/task_2040_differential_client/Cargo.toml");
const DIFFERENTIAL_CLIENT_MAIN_RS: &str =
    include_str!("fixtures/task_2040_differential_client/src/main.rs");
const EXTERNAL_CPS_CLIENT_CARGO_TOML: &str =
    include_str!("fixtures/task_2040_external_cps_client/Cargo.toml");
const EXTERNAL_CPS_CLIENT_MAIN_RS: &str =
    include_str!("fixtures/task_2040_external_cps_client/src/main.rs");

const TASK_2040_OWNER: &str = "TASK-2040";
const RUNTIME_PACKAGE_NAME: &str = "ash-runtime";
const RUNTIME_CRATE_IDENTIFIER: &str = "ash_runtime";
const RUNTIME_DIRECTORY_NAME: &str = "ash-runtime";

const TASK_2035_SYNTH_WRAPPER_ID: &str = "TASK-2035-SYNTH-WRAPPER-001";
const TASK_2035_SYNTH_WRAPPER_SOURCE: &str =
    "fn contract_target_zero() -> Int { 0 }\nfn main() -> Bool { contract_target_zero() == 0 }\n";
const TASK_2035_REPL_INT_ID: &str = "TASK-2035-REPL-ROUTE-001";
const TASK_2035_REPL_INT_SOURCE: &str = "fn main() -> Int { 42 }\n";
const TASK_2035_REPL_BOOL_ID: &str = "TASK-2035-REPL-ROUTE-002";
const TASK_2035_REPL_BOOL_SOURCE: &str = "fn main() -> Bool { 1 == 1 }\n";
const TASK_2035_SHARED_ROUTE_ID: &str = "TASK-2035-SHARED-ROUTE-001";

/// The frozen manifest records source locations as entries, not broad deletion
/// permissions. This is the only TASK-2040 delete record whose source file is
/// shared with the retained Engine; the module declaration is its retirement
/// target. No other old-root path is allowed through this exception.
const SHARED_FILE_SYMBOL_REMOVALS: [(&str, &str, &str); 1] = [(
    "AUDIT-204-AST-010",
    "crates/ash-engine/src/lib.rs",
    "mod differential;",
)];

/// TASK-2040 preserves exactly these records while the required package rename
/// moves their source root from `ash-interp` to `ash-runtime`.
const REPLACED_RUNTIME_PATHS: [(&str, &str); 4] = [
    (
        "crates/ash-interp/src/eval/builtins.rs",
        "crates/ash-runtime/src/builtin_catalog.rs",
    ),
    (
        "crates/ash-interp/tests/builtin_dispatch/host_hook_metadata.rs",
        "crates/ash-runtime/tests/builtin_dispatch/host_hook_metadata.rs",
    ),
    (
        "crates/ash-interp/tests/builtin_dispatch/markdown.rs",
        "crates/ash-runtime/tests/builtin_dispatch/markdown.rs",
    ),
    (
        "crates/ash-interp/tests/task_1932_host_boundary_cross_boundary_fixtures.rs",
        "crates/ash-runtime/tests/task_1932_host_boundary_cross_boundary_fixtures.rs",
    ),
];

#[derive(Clone, Copy)]
struct DeclaredSourceContract {
    id: &'static str,
    source: &'static str,
}

const TASK_2035_SOURCE_CONTRACTS: [DeclaredSourceContract; 4] = [
    DeclaredSourceContract {
        id: TASK_2035_SYNTH_WRAPPER_ID,
        source: TASK_2035_SYNTH_WRAPPER_SOURCE,
    },
    DeclaredSourceContract {
        id: TASK_2035_REPL_INT_ID,
        source: TASK_2035_REPL_INT_SOURCE,
    },
    DeclaredSourceContract {
        id: TASK_2035_REPL_BOOL_ID,
        source: TASK_2035_REPL_BOOL_SOURCE,
    },
    DeclaredSourceContract {
        id: TASK_2035_SHARED_ROUTE_ID,
        source: TASK_2035_REPL_INT_SOURCE,
    },
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("ash-engine remains under the workspace crates directory")
        .to_path_buf()
}

fn audit_entries() -> Vec<serde_json::Map<String, JsonValue>> {
    let manifest: JsonValue =
        serde_json::from_str(AUDIT_204_MANIFEST).expect("the frozen AUDIT-204 manifest is JSON");
    manifest["entries"]
        .as_array()
        .expect("AUDIT-204 has an entries array")
        .iter()
        .map(|entry| {
            entry
                .as_object()
                .expect("every audit entry is an object")
                .clone()
        })
        .collect()
}

fn entry_text<'a>(entry: &'a serde_json::Map<String, JsonValue>, field: &str) -> &'a str {
    entry[field]
        .as_str()
        .unwrap_or_else(|| panic!("AUDIT-204 entry has string {field}"))
}

fn task_2040_delete_entries() -> Vec<serde_json::Map<String, JsonValue>> {
    audit_entries()
        .into_iter()
        .filter(|entry| {
            entry_text(entry, "owner_or_external_handoff") == TASK_2040_OWNER
                && entry_text(entry, "disposition") == "delete"
        })
        .collect()
}

fn shared_symbol_removal(entry: &serde_json::Map<String, JsonValue>) -> Option<&'static str> {
    let id = entry_text(entry, "id");
    let path = entry_text(entry, "path");
    SHARED_FILE_SYMBOL_REMOVALS
        .iter()
        .find_map(|(expected_id, expected_path, symbol)| {
            (id == *expected_id && path == *expected_path).then_some(*symbol)
        })
}

fn assert_no_fixture_setup_failure(output: &Output, stderr: &str) {
    assert!(
        !stderr.contains("failed to load source for dependency")
            && !stderr.contains("failed to parse manifest")
            && !stderr.contains("no matching package named"),
        "the static fixture must reach API checking rather than fail setup: {stderr}"
    );
    assert!(
        output.status.code().is_some(),
        "cargo must return a process status"
    );
}

fn inaccessible_module(stderr: &str, module: &str) -> bool {
    stderr.contains(&format!("module `{module}` is private"))
        || stderr.contains(&format!("could not find `{module}` in"))
}

fn absent_engine_module(stderr: &str, module: &str) -> bool {
    stderr
        .replace('`', "")
        .contains(&format!("could not find {module} in ash_engine"))
}

fn compile_fixture(fixture_name: &str, cargo_toml: String, main_rs: String) -> (Output, String) {
    let engine_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dir = workspace_root()
        .join("target")
        .join("task-2040-external-fixtures")
        .join(fixture_name);
    fs::create_dir_all(&target_dir).expect("create an isolated static-client target directory");
    let fixture = tempfile::Builder::new()
        .prefix(&format!(".task-2040-{fixture_name}-"))
        .tempdir_in(engine_root)
        .expect("create an isolated static client fixture inside ash-engine");
    fs::write(fixture.path().join("Cargo.toml"), cargo_toml)
        .expect("materialize the static client Cargo manifest");
    fs::create_dir(fixture.path().join("src")).expect("create the static client source directory");
    fs::write(fixture.path().join("src/main.rs"), main_rs)
        .expect("materialize the static client source");

    let output = Command::new("cargo")
        .args(["check", "--offline", "--quiet", "--target-dir"])
        .arg(target_dir)
        .current_dir(fixture.path())
        .output()
        .expect("run Cargo against the static client fixture");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output, stderr)
}

fn expected_terminal(contract_id: &str) -> CanonicalTerminalEnvelopeV1 {
    match contract_id {
        TASK_2035_SYNTH_WRAPPER_ID | TASK_2035_REPL_BOOL_ID => {
            CanonicalTerminalEnvelopeV1::returned(Value::Bool(true))
        }
        TASK_2035_REPL_INT_ID | TASK_2035_SHARED_ROUTE_ID => {
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        }
        _ => panic!("test helper accepts only declared TASK-2035 identities"),
    }
}

fn engine_terminal_for_declared_source(source: &str) -> CanonicalTerminalEnvelopeV1 {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("task-2040 test runtime builds");
    runtime.block_on(async {
        let engine = Engine::new().build().expect("test Engine builds");
        let mut entry = engine
            .parse_file_source(Path::new("task-2040-declared-source.ash"), source)
            .expect("declared TASK-2035 source parses through Engine");
        let admitted = engine
            .admit_program(&mut entry)
            .expect("declared TASK-2035 source admits through Engine");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("the issuing Engine mints the request");
        engine
            .execute_admitted_program(&request)
            .await
            .expect("Engine terminalizes the admitted declared source")
    })
}

#[test]
fn task_2040_manifest_delete_entries_are_absent_with_one_shared_file_symbol_exception() {
    let root = workspace_root();
    let delete_entries = task_2040_delete_entries();
    assert_eq!(
        delete_entries.len(),
        165,
        "the frozen manifest retains every TASK-2040 delete record"
    );

    let exception_ids = delete_entries
        .iter()
        .filter_map(|entry| shared_symbol_removal(entry).map(|_| entry_text(entry, "id")))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        exception_ids,
        BTreeSet::from(["AUDIT-204-AST-010"]),
        "only Engine's shared library-file entry may be checked by its exact module symbol"
    );

    for entry in delete_entries {
        let path = entry_text(&entry, "path");
        if let Some(symbol) = shared_symbol_removal(&entry) {
            let source = fs::read_to_string(root.join(path))
                .unwrap_or_else(|error| panic!("read retained shared file {path}: {error}"));
            assert!(
                !source.contains(symbol),
                "{path} must remove the manifest-owned `{symbol}` entry"
            );
            continue;
        }
        assert!(
            !root.join(path).exists(),
            "{} ({path}) must be absent after TASK-2040 retirement",
            entry_text(&entry, "id")
        );
    }
}

#[test]
fn retained_engine_library_has_no_differential_module_or_public_export() {
    let source = fs::read_to_string(workspace_root().join("crates/ash-engine/src/lib.rs"))
        .expect("read retained Engine library source");
    assert!(
        !source.contains("mod differential;"),
        "the retained Engine library must not compile the retired comparison route"
    );
    assert!(
        !source.contains("pub mod differential;"),
        "the retained Engine library must not export a differential execution route"
    );
}

#[test]
fn named_runtime_replacements_survive_only_at_the_renamed_runtime_root() {
    let root = workspace_root();
    let replacements = audit_entries()
        .into_iter()
        .filter(|entry| {
            entry_text(entry, "owner_or_external_handoff") == TASK_2040_OWNER
                && entry_text(entry, "disposition") == "replace"
        })
        .map(|entry| entry_text(&entry, "path").to_string())
        .collect::<BTreeSet<_>>();
    let expected_old_paths = REPLACED_RUNTIME_PATHS
        .iter()
        .map(|(old_path, _)| (*old_path).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        replacements, expected_old_paths,
        "only the four frozen TASK-2040 replacement records receive the rename mapping"
    );

    for (old_path, renamed_path) in REPLACED_RUNTIME_PATHS {
        assert!(
            !root.join(old_path).exists(),
            "replaced runtime source {old_path} must be absent after moving to {renamed_path}"
        );
        assert!(
            root.join(renamed_path).is_file(),
            "required replacement {old_path} must survive at its named runtime target {renamed_path}"
        );
    }
}

#[test]
fn deferred_lean_records_remain_separate_project_material_without_current_authority() {
    let root = workspace_root();
    let deferred = audit_entries()
        .into_iter()
        .filter(|entry| entry_text(entry, "disposition") == "deferred_separate_project")
        .collect::<Vec<_>>();
    assert_eq!(
        deferred.len(),
        43,
        "the frozen Lean handoff remains complete"
    );

    for entry in deferred {
        let path = entry_text(&entry, "path");
        assert!(
            root.join(path).is_file(),
            "deferred Lean material remains at {path}"
        );
        assert_eq!(
            entry_text(&entry, "owner_or_external_handoff"),
            "external:lean-reference-project",
            "{path} remains owned by its separate project"
        );
        assert_eq!(
            entry_text(&entry, "external_project"),
            "lean-reference-project",
            "{path} retains its separate-project handoff"
        );
        assert!(
            entry_text(&entry, "prohibited_current_authority")
                .contains("not a current Ash execution route"),
            "{path} cannot gain current Ash execution authority"
        );
    }
}

#[test]
fn external_client_cannot_compile_against_direct_ast_evaluation() {
    // CodeDev changes all three runtime constants to `ash-runtime`,
    // `ash_runtime`, and `ash-runtime` when performing the required crate
    // rename. Before that removal, this direct-AST fixture deliberately
    // compiles and makes the test RED for the intended reason.
    let root = workspace_root();
    let direct_ast_manifest = DIRECT_AST_CLIENT_CARGO_TOML
        .replace(
            "__ASH_CORE_PATH__",
            &root.join("crates/ash-core").display().to_string(),
        )
        .replace("__TASK_2040_RUNTIME_PACKAGE__", RUNTIME_PACKAGE_NAME)
        .replace(
            "__TASK_2040_RUNTIME_PATH__",
            &root
                .join("crates")
                .join(RUNTIME_DIRECTORY_NAME)
                .display()
                .to_string(),
        );
    let direct_ast_api = ["eval", "_expr"].concat();
    let direct_ast_main = DIRECT_AST_CLIENT_MAIN_RS
        .replace("__TASK_2040_RUNTIME_CRATE__", RUNTIME_CRATE_IDENTIFIER)
        .replace("__TASK_2040_DIRECT_AST_API__", &direct_ast_api);
    let (direct_ast, direct_ast_stderr) =
        compile_fixture("direct-ast", direct_ast_manifest, direct_ast_main);
    assert!(
        !direct_ast.status.success(),
        "a client can still import and call the direct AST evaluator; TASK-2040 must remove it"
    );
    assert_no_fixture_setup_failure(&direct_ast, &direct_ast_stderr);
    assert!(
        direct_ast_stderr.contains(&direct_ast_api),
        "the direct-AST probe must fail because its requested API is unavailable: {direct_ast_stderr}"
    );
}

#[test]
fn external_client_cannot_compile_against_retired_comparison_route() {
    let engine_path = workspace_root()
        .join("crates/ash-engine")
        .display()
        .to_string();
    let differential_module = ["differ", "ential"].concat();
    let differential_type = ["Differential", "Harness"].concat();
    let differential_manifest =
        DIFFERENTIAL_CLIENT_CARGO_TOML.replace("__ASH_ENGINE_PATH__", &engine_path);
    let differential_main = DIFFERENTIAL_CLIENT_MAIN_RS
        .replace("__TASK_2040_DIFFERENTIAL_MODULE__", &differential_module)
        .replace("__TASK_2040_DIFFERENTIAL_TYPE__", &differential_type);
    let (differential, differential_stderr) =
        compile_fixture("differential", differential_manifest, differential_main);
    assert!(
        !differential.status.success(),
        "an external client must not compile against the retired comparison route"
    );
    assert_no_fixture_setup_failure(&differential, &differential_stderr);
    assert!(
        absent_engine_module(&differential_stderr, &differential_module),
        "the comparison-route probe must fail because ash_engine has no requested module: {differential_stderr}"
    );
}

#[test]
fn external_client_cannot_compile_against_non_engine_cps_execution() {
    let engine_path = workspace_root()
        .join("crates/ash-engine")
        .display()
        .to_string();
    let cps_manifest = EXTERNAL_CPS_CLIENT_CARGO_TOML.replace("__ASH_ENGINE_PATH__", &engine_path);
    let (external_cps, external_cps_stderr) = compile_fixture(
        "external-cps",
        cps_manifest,
        EXTERNAL_CPS_CLIENT_MAIN_RS.to_string(),
    );
    assert!(
        !external_cps.status.success(),
        "an external client must not compile against non-Engine CPS execution"
    );
    assert_no_fixture_setup_failure(&external_cps, &external_cps_stderr);
    assert!(
        inaccessible_module(&external_cps_stderr, "private_cps"),
        "the CPS probe must fail at the unavailable Engine module: {external_cps_stderr}"
    );
}

#[test]
fn declared_task_2035_contract_terminalizes_through_engine() {
    let contract = TASK_2035_SOURCE_CONTRACTS[0];
    let terminal = engine_terminal_for_declared_source(contract.source);

    assert_eq!(
        terminal,
        expected_terminal(contract.id),
        "{} must terminalize through Engine without a legacy evaluator",
        contract.id,
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn declared_task_2035_source_contracts_terminalize_only_through_engine(
        contract_index in prop_oneof![Just(0_usize), Just(1_usize), Just(2_usize), Just(3_usize)],
    ) {
        let contract = TASK_2035_SOURCE_CONTRACTS[contract_index];
        let terminal = engine_terminal_for_declared_source(contract.source);

        prop_assert_eq!(
            terminal,
            expected_terminal(contract.id),
            "{} must retain its admitted Engine terminal without another evaluator",
            contract.id,
        );
    }
}
