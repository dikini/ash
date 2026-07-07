//! TASK-1937 HTTP stdlib wrapper/profile tests.

use ash_core::Value;
use ash_core::runtime::HostBoundaryOutcome;
use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};
use tokio::net::TcpListener;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

#[tokio::test]
async fn stdlib_http_wrappers_execute_through_sandboxed_profile_and_record_evidence() {
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
        fn main() -> Int {{
            do {{
                get_response <- http::get("{base}/get");
                post_response <- http::post("{base}/post", "body");
                put_response <- http::put("{base}/put", "body");
                delete_response <- http::delete("{base}/delete");
                return get_response.status + post_response.status + put_response.status + delete_response.status
            }}
        }}
    "#
    );

    let result = engine
        .run(&source)
        .await
        .expect("HTTP stdlib wrappers should execute");
    assert_eq!(result, Value::Int(800));

    let evidence = engine.host_boundary_evidence().await;
    for operation in ["get", "post", "put", "delete"] {
        assert!(
            evidence.iter().any(|record| record.provider_name == "http"
                && record.operation_name == operation
                && record.outcome == HostBoundaryOutcome::Succeeded
                && record.authority_neutral),
            "{operation} should record authority-neutral success evidence: {evidence:?}"
        );
        assert!(
            evidence
                .iter()
                .all(|record| !record.redacted_subject.contains(&base)),
            "HTTP evidence must redact raw URL arguments: {evidence:?}"
        );
    }
}

#[tokio::test]
async fn stdlib_http_wrapper_denies_blocked_host_before_provider_execution() {
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
        fn main() -> Int {{
            do {{
                http::get("{blocked_url}");
                return 0
            }}
        }}
    "#
    );

    let error = engine
        .run(&source)
        .await
        .expect_err("blocked HTTP host should fail closed");
    assert!(error.to_string().contains("denied http.get"), "{error}");

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "http"
            && record.operation_name == "get"
            && record.outcome == HostBoundaryOutcome::Denied),
        "blocked host should record denied sandbox evidence: {evidence:?}"
    );
    assert!(
        evidence
            .iter()
            .all(|record| !record.redacted_subject.contains(&blocked_url)),
        "HTTP denial evidence must redact raw URL arguments: {evidence:?}"
    );
}

#[tokio::test]
async fn stdlib_http_wrapper_preserves_provider_failure_taxonomy() {
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
        fn main() -> Int {{
            do {{
                http::get("{failing_url}");
                return 0
            }}
        }}
    "#
    );

    let error = engine
        .run(&source)
        .await
        .expect_err("allowed host with no server should be provider failure");
    assert!(error.to_string().contains("HTTP GET failed"), "{error}");

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "http"
            && record.operation_name == "get"
            && record.outcome == HostBoundaryOutcome::Failed),
        "provider failure should record failed host-boundary evidence: {evidence:?}"
    );
    assert!(
        evidence
            .iter()
            .all(|record| !record.redacted_subject.contains(&failing_url)),
        "HTTP failure evidence must redact raw URL arguments: {evidence:?}"
    );
}
