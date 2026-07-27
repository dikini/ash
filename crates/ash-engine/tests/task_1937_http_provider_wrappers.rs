//! TASK-1937 HTTP stdlib wrapper/profile tests under strict closed admission.
//!
//! The wrapper declarations, imports, request shapes, and profile registration remain checked at
//! the source boundary. Positive host behavior deliberately awaits authorized frame installation
//! and the async CPS host driver; generic source execution must not revive direct dispatch.

use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};
use ash_interp::ExecError;
use std::io::ErrorKind;
use std::net::TcpListener as StdTcpListener;
use std::sync::OnceLock;
use tokio::net::TcpListener;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The host's ability to bind the loopback address Wiremock and this target use.
///
/// The result is cached because this is a host capability, rather than a behavior
/// exercised independently by each test.
#[derive(Debug)]
enum LoopbackTcpBindCapability {
    Available,
    PermissionDenied(String),
    UnexpectedFailure(String),
}

fn loopback_tcp_bind_capability() -> &'static LoopbackTcpBindCapability {
    static CAPABILITY: OnceLock<LoopbackTcpBindCapability> = OnceLock::new();

    CAPABILITY.get_or_init(|| match StdTcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            drop(listener);
            LoopbackTcpBindCapability::Available
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            LoopbackTcpBindCapability::PermissionDenied(error.to_string())
        }
        Err(error) => LoopbackTcpBindCapability::UnexpectedFailure(format!(
            "kind={:?}, error={error}",
            error.kind()
        )),
    })
}

macro_rules! require_loopback_tcp_bind {
    () => {
        match loopback_tcp_bind_capability() {
            LoopbackTcpBindCapability::Available => {}
            LoopbackTcpBindCapability::PermissionDenied(error) => {
                eprintln!(
                    "skipping HTTP wrapper integration test: host denied loopback TCP binding \\
                     (127.0.0.1:0, PermissionDenied): {error}"
                );
                return;
            }
            LoopbackTcpBindCapability::UnexpectedFailure(error) => {
                panic!(
                    "HTTP wrapper integration test setup failed while checking loopback TCP \\
                     bind capability (127.0.0.1:0): {error}"
                );
            }
        }
    };
}

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

async fn parse_check_execute(
    engine: &Engine,
    fixture: &str,
    source: &str,
) -> Result<ExecError, Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(fixture);
    let mut application = engine.parse_file_source(path, source)?;
    engine.check(&mut application)?;
    let error = engine
        .execute(&application)
        .await
        .expect_err("generic source execution must reject without checked Core/CPS admission");
    Ok(error)
}

fn assert_closed_admission(error: ExecError) {
    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "generic source execution must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn stdlib_http_wrappers_parse_check_then_reject_before_provider_execution() {
    require_loopback_tcp_bind!();
    let server = MockServer::start().await;
    for (verb, route, body) in [
        ("GET", "/get", "get-ok"),
        ("POST", "/post", "post-ok"),
        ("PUT", "/put", "put-ok"),
        ("DELETE", "/delete", "delete-ok"),
    ] {
        Mock::given(method(verb))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
    }

    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::sandboxed_http(
            "task-1937-http",
            [server.address().ip().to_string()],
        ))
        .await
        .expect("http profile installs");

    let base = server.uri();
    let source = format!(
        r#"
        use http::{{get, post, put, delete}}

        fn main() -> Int {{
            do {{
                let get_response = get("{base}/get");
                let post_response = post("{base}/post", "body");
                let put_response = put("{base}/put", "body");
                let delete_response = delete("{base}/delete");
                return get_response.status + post_response.status + put_response.status + delete_response.status
            }}
        }}
    "#
    );

    let error = parse_check_execute(
        &engine,
        "task_1937_http_provider_wrappers_success.ash",
        &source,
    )
    .await
    .expect("HTTP stdlib wrappers should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent HTTP provider execution and host evidence: {evidence:?}"
    );
}

#[tokio::test]
async fn stdlib_http_wrapper_blocked_host_shape_rejects_before_provider_execution() {
    require_loopback_tcp_bind!();
    let server = MockServer::start().await;
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::sandboxed_http(
            "task-1937-http",
            ["api.example.invalid"],
        ))
        .await
        .expect("http profile installs");

    let blocked_url = format!("{}/blocked", server.uri());
    let source = format!(
        r#"
        use http::{{get}}

        fn main() -> Int {{
            do {{
                get("{blocked_url}");
                return 0
            }}
        }}
    "#
    );

    let error = parse_check_execute(
        &engine,
        "task_1937_http_provider_wrappers_denied.ash",
        &source,
    )
    .await
    .expect("blocked HTTP request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the blocked HTTP request from reaching a provider: {evidence:?}"
    );
}

#[tokio::test]
async fn stdlib_http_wrapper_provider_failure_shape_rejects_before_provider_execution() {
    require_loopback_tcp_bind!();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind unused local port");
    let address = listener.local_addr().expect("local addr");
    drop(listener);

    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::sandboxed_http(
            "task-1937-http",
            [address.ip().to_string()],
        ))
        .await
        .expect("http profile installs");

    let failing_url = format!("http://{address}/unreachable");
    let source = format!(
        r#"
        use http::{{get}}

        fn main() -> Int {{
            do {{
                get("{failing_url}");
                return 0
            }}
        }}
    "#
    );

    let error = parse_check_execute(
        &engine,
        "task_1937_http_provider_wrappers_failure.ash",
        &source,
    )
    .await
    .expect("provider failure request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the failing HTTP request from reaching a provider: {evidence:?}"
    );
}
