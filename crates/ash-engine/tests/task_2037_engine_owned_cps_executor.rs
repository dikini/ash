//! TASK-2037 RED contracts for the Engine-owned checked-CPS executor boundary.
//!
//! These tests cover only TASK-2037's migration-owned CPS paths. They do not
//! claim the TASK-2040 direct-AST or differential deletion work is complete.

use ash_core::{Expr, Value};
use ash_engine::{
    CanonicalTerminalEnvelopeV1, Engine, ProductionTerminalClassification,
    standard_profiles::StandardProviderProfile,
};
use proptest::prelude::*;
use std::{path::Path, process::Command, time::Duration};

const RUNTIME_LIBRARY_SOURCE: &str = include_str!("../../ash-runtime/src/lib.rs");
const PRODUCTION_DRIVER_SOURCE: &str = include_str!("../src/production_cps_driver.rs");
const CHECKED_ADMISSION_SOURCE: &str = include_str!("../src/checked_cps_admission.rs");
const RUNTIME_CARGO_MANIFEST: &str = include_str!("../../ash-runtime/Cargo.toml");
const AUDIT_204_MANIFEST: &str =
    include_str!("../../../docs/plan/audits/AUDIT-204-direct-ast-retirement.json");
const EXTERNAL_CLIENT_CARGO_TOML: &str =
    include_str!("fixtures/task_2037_external_client/Cargo.toml");
const EXTERNAL_CLIENT_MAIN_RS: &str =
    include_str!("fixtures/task_2037_external_client/src/main.rs");

const TASK_2035_SHARED_SOURCE: &str = "fn main() -> Int { 42 }\n";
const TASK_2035_SHARED_VALUE: i64 = 42;
const TASK_2032_SLEEP_SOURCE: &str = "fn main() -> Null { time::sleep(1) }";
const TASK_2032_TRAP_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with trap_sleep }
";

fn audit_record(audit_id: &str) -> &str {
    let record_start = AUDIT_204_MANIFEST
        .find(&format!("\"id\": \"{audit_id}\""))
        .unwrap_or_else(|| panic!("AUDIT-204 retains migration record {audit_id}"));
    let record_end = AUDIT_204_MANIFEST[record_start..]
        .find("\n    },")
        .map(|offset| record_start + offset)
        .expect("the AUDIT-204 migration record remains delimited");
    &AUDIT_204_MANIFEST[record_start..record_end]
}

async fn install_time_sleep_profile(engine: &Engine) {
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2037-engine-owned-cps-executor",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("the curated time.sleep source installs its standard provider profile");
    engine
        .register_time_sleep_provider_binding()
        .expect("the Engine seals the curated time.sleep provider binding before admission");
}

fn access_failure_mentions(stderr: &str, module: &str) -> bool {
    stderr.contains(&format!("module `{module}` is private"))
        || stderr.contains(&format!("could not find `{module}` in"))
}

#[test]
fn task_2037_owned_cps_paths_do_not_depend_on_the_non_engine_executor() {
    let migration_scope = [
        ("AUDIT-204-CPS-001", "crates/ash-interp/src/cps/mod.rs"),
        (
            "AUDIT-204-CPS-002",
            "crates/ash-engine/src/production_cps_driver.rs",
        ),
        (
            "AUDIT-204-CPS-003",
            "crates/ash-engine/src/checked_cps_admission.rs",
        ),
    ];

    for (audit_id, path) in migration_scope {
        let record = audit_record(audit_id);
        assert!(
            record.contains(&format!("\"path\": \"{path}\""))
                && record.contains("\"owner_or_external_handoff\": \"TASK-2037\""),
            "the frozen audit must retain the exact TASK-2037 migration path {path}"
        );
    }

    assert!(
        !RUNTIME_LIBRARY_SOURCE.contains("pub mod cps;"),
        "a public `ash_runtime::cps` module gives non-Engine consumers access to checked-CPS \
         validation and evaluation"
    );
    assert!(
        !RUNTIME_LIBRARY_SOURCE.contains("pub use cps::"),
        "the residual runtime-support crate must not re-export a checked-CPS execution API"
    );

    for (path, source) in [
        (
            "crates/ash-engine/src/production_cps_driver.rs",
            PRODUCTION_DRIVER_SOURCE,
        ),
        (
            "crates/ash-engine/src/checked_cps_admission.rs",
            CHECKED_ADMISSION_SOURCE,
        ),
    ] {
        assert!(
            !source.contains("ash_runtime::cps::"),
            "{path} must use the Engine-private CPS executor and validation boundary, not \
             `ash_runtime::cps`"
        );
    }
}

#[test]
fn external_clients_cannot_compile_against_retired_execution_routes() {
    let fixture = tempfile::Builder::new()
        .prefix(".task-2037-external-client-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .expect("create isolated external-client fixture directory inside the crate");
    let engine_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_root = engine_root
        .parent()
        .expect("ash-engine remains below the crates directory")
        .join("ash-runtime");
    let fixture_target = engine_root
        .parent()
        .and_then(Path::parent)
        .expect("ash-engine remains below the workspace root")
        .join("target/task-2037-external-client");
    let cargo_toml = EXTERNAL_CLIENT_CARGO_TOML
        .replace("__ASH_ENGINE_PATH__", &engine_root.display().to_string())
        .replace("__ASH_RUNTIME_PATH__", &runtime_root.display().to_string());
    let client_main = EXTERNAL_CLIENT_MAIN_RS
        .replace("__TASK_2037_MODULE__", "differential")
        .replace("__TASK_2037_TYPE__", concat!("Differential", "Harness"))
        .replace(
            "__TASK_2037_RUNTIME_ACCESS__",
            &format!("ash_runtime::{}{}", "eval_", "expr"),
        );
    std::fs::write(fixture.path().join("Cargo.toml"), cargo_toml)
        .expect("materialize the static external-client Cargo fixture");
    std::fs::create_dir(fixture.path().join("src"))
        .expect("create the static external-client fixture source directory");
    std::fs::write(fixture.path().join("src/main.rs"), client_main)
        .expect("materialize the static external-client Rust fixture");

    let output = Command::new("cargo")
        .args(["check", "--offline", "--quiet", "--target-dir"])
        .arg(fixture_target)
        .current_dir(fixture.path())
        .output()
        .expect("run Cargo against the external-client fixture");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "an external client must not compile against retired evaluator or differential routes"
    );
    assert!(
        access_failure_mentions(&stderr, "cps"),
        "the fixture must fail at the inaccessible ash_runtime::cps route, not dependency setup:\n{stderr}"
    );
    assert!(
        access_failure_mentions(&stderr, "differential"),
        "the fixture must fail at the inaccessible ash_engine::differential route, not dependency setup:\n{stderr}"
    );
    assert!(
        stderr.contains(&["eval", "_expr"].concat()),
        "the fixture must fail at the unavailable direct-AST API, not dependency setup:\n{stderr}"
    );
    assert!(
        !stderr.contains("failed to load source for dependency")
            && !stderr.contains("failed to parse manifest")
            && !stderr.contains("no matching package named"),
        "the static fixture must reach API access checking rather than fail dependency setup:\n{stderr}"
    );
}

proptest! {
    #[test]
    fn task_2035_shared_literal_route_terminalizes_through_an_admitted_engine_request(
        source in Just(TASK_2035_SHARED_SOURCE),
        expected_value in Just(TASK_2035_SHARED_VALUE),
    ) {
        let engine = Engine::new().build().expect("Engine builds for the exact shared route");
        let mut entry = engine
            .parse(source)
            .expect("TASK-2035's exact shared source parses");
        let program = engine
            .admit_program(&mut entry)
            .expect("the exact shared source receives Engine admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&program, None)
            .expect("only the issuing Engine mints the shared-route request");
        let terminal = tokio_test::block_on(engine.execute_admitted_program(&request))
            .expect("the Engine terminalizes its own admitted shared-route request");

        prop_assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(expected_value)),
        );
    }
}

#[tokio::test]
async fn malformed_forged_checked_core_cps_artifact_rejects_before_an_engine_request_is_minted() {
    let engine = Engine::new().build().expect("Engine builds");
    let mut entry = engine
        .parse(TASK_2035_SHARED_SOURCE)
        .expect("the exact shared source parses before forged evidence is introduced");
    engine
        .check(&mut entry)
        .expect("the exact shared source type-checks before the checked artifact is forged");
    entry.core = Expr::Literal(Value::Null);

    let rejection = engine
        .admit_program(&mut entry)
        .expect_err("a forged checked artifact must reject before request construction");

    assert_eq!(
        rejection.classification(),
        ProductionTerminalClassification::InvalidCheckedCoreCps,
        "a forged artifact must not become execution or frame authority"
    );
    assert_eq!(
        rejection
            .canonical_terminal_envelope()
            .expect("the pre-execution rejection has a canonical terminal projection"),
        CanonicalTerminalEnvelopeV1::invalid_checked_artifact(),
    );
    assert!(
        engine.host_boundary_evidence().await.is_empty(),
        "rejection before request construction must not dispatch a terminal execution"
    );
}

#[tokio::test]
async fn admitted_language_trap_projects_the_canonical_terminal_envelope() {
    let engine = Engine::new().build().expect("Engine builds");
    let mut entry = engine
        .parse(TASK_2032_TRAP_SLEEP_SOURCE)
        .expect("the curated TASK-2032 language-trap source parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the curated language-trap source");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("only the issuing Engine creates the admitted language-trap request");
    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("the Engine terminalizes the admitted language-trap request");

    assert_eq!(
        envelope,
        CanonicalTerminalEnvelopeV1::trapped("division by zero"),
        "the language trap projects only through the canonical Engine terminal envelope"
    );
}

#[tokio::test(start_paused = true)]
async fn admitted_zero_timeout_projects_the_canonical_terminal_envelope() {
    let engine = Engine::new().build().expect("Engine builds");
    install_time_sleep_profile(&engine).await;
    let mut entry = engine
        .parse(TASK_2032_SLEEP_SOURCE)
        .expect("the curated TASK-2032 timeout source parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the curated timeout source");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, Some(Duration::ZERO))
        .expect("the issuing Engine creates a zero-timeout admitted request");
    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("the Engine projects the zero timeout as a canonical terminal result");

    assert_eq!(envelope, CanonicalTerminalEnvelopeV1::timed_out());
}

#[tokio::test(start_paused = true)]
async fn admitted_precancelled_request_projects_the_canonical_terminal_envelope() {
    let engine = Engine::new().build().expect("Engine builds");
    install_time_sleep_profile(&engine).await;
    let mut entry = engine
        .parse(TASK_2032_SLEEP_SOURCE)
        .expect("the curated TASK-2032 cancellation source parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the curated cancellation source");
    let (request, cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("the issuing Engine creates the admitted cancellation request");
    cancellation.cancel();
    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("the Engine projects pre-cancellation as a canonical terminal result");

    assert_eq!(envelope, CanonicalTerminalEnvelopeV1::cancelled());
}

#[test]
fn task_2040_renames_runtime_and_removes_direct_ast_exports() {
    assert!(
        RUNTIME_CARGO_MANIFEST.contains("name = \"ash-runtime\""),
        "TASK-2040 renames the residual support crate to ash-runtime"
    );
    assert!(
        !RUNTIME_CARGO_MANIFEST.contains("name = \"ash-interp\""),
        "the renamed support crate cannot retain its interpreter package identity"
    );

    let ast_record = audit_record("AUDIT-204-AST-001");
    assert!(
        ast_record.contains("\"path\": \"crates/ash-interp/src/eval.rs\"")
            && ast_record.contains("\"disposition\": \"delete\"")
            && ast_record.contains("\"owner_or_external_handoff\": \"TASK-2040\""),
        "AUDIT-204 assigns the retained direct-AST evaluator deletion to TASK-2040"
    );
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ash-interp/src/eval.rs")
            .exists(),
        "the TASK-2040-owned direct-AST legacy is removed"
    );
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ash-interp/Cargo.toml")
            .exists(),
        "the old interpreter package root is removed"
    );
    for legacy_export in ["pub mod eval;", "pub mod guard;", "pub mod policy;"] {
        assert!(
            !RUNTIME_LIBRARY_SOURCE.contains(legacy_export),
            "ash-runtime may not export the retired {legacy_export} route"
        );
    }
}
