//! Canonical Engine submission helpers for test-runner source execution.
//!
//! The test client owns source selection and result presentation only.  It
//! never evaluates an AST, Core expression, or CPS artifact itself.

use std::path::Path;
use std::time::Duration;

use ash_engine::{CanonicalTerminalEnvelopeV1, Engine};
use serde_json::{Value as JsonValue, json};

use crate::test_runner::types::Outcome;

/// Submit source through the admitted Engine boundary and return its canonical
/// terminal envelope.  This is the sole execution seam used by the test
/// client.
pub(crate) fn execute_admitted_source(
    engine: &Engine,
    path: &Path,
    source: &str,
    timeout: Duration,
) -> Result<CanonicalTerminalEnvelopeV1, String> {
    let mut entry = engine
        .parse_file_source(path, source)
        .map_err(|error| format!("parse error: {error}"))?;
    let admitted = engine
        .admit_program(&mut entry)
        .map_err(|error| format!("admission error: {error}"))?;
    let (request, _) = engine
        .new_admitted_program_request(&admitted, Some(timeout))
        .map_err(|error| format!("request error: {error}"))?;
    let execution = engine.execute_admitted_program(&request);

    std::thread::Builder::new()
        .name("ash-test-engine".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| format!("runtime error: {error}"))?;
            runtime
                .block_on(execution)
                .map_err(|error| format!("execution error: {error}"))
        })
        .map_err(|error| format!("execution worker error: {error}"))?
        .join()
        .map_err(|_| "execution worker panicked".to_string())?
}

/// Classify a canonical terminal result for ordinary authored source tests.
pub(crate) fn classify_authored_terminal(
    terminal: &CanonicalTerminalEnvelopeV1,
) -> (Outcome, Option<String>) {
    match terminal {
        CanonicalTerminalEnvelopeV1::Returned(ash_core::Value::Bool(false)) => {
            (Outcome::Fail, Some("test returned false".to_string()))
        }
        CanonicalTerminalEnvelopeV1::Returned(_) => (Outcome::Pass, None),
        CanonicalTerminalEnvelopeV1::Trapped(reason) => (Outcome::Fail, Some(reason.clone())),
        CanonicalTerminalEnvelopeV1::AdmissionRejected => (
            Outcome::Error,
            Some("admitted test source was rejected".to_string()),
        ),
        CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact => (
            Outcome::Error,
            Some("admitted test source produced an invalid checked artifact".to_string()),
        ),
        CanonicalTerminalEnvelopeV1::TimedOut => {
            (Outcome::Error, Some("test timed out".to_string()))
        }
        CanonicalTerminalEnvelopeV1::Cancelled => (
            Outcome::Error,
            Some("admitted test source was cancelled".to_string()),
        ),
    }
}

/// Serialize the stable terminal observation for a test repro record.
#[must_use]
pub(crate) fn terminal_envelope_json(terminal: &CanonicalTerminalEnvelopeV1) -> JsonValue {
    match terminal {
        CanonicalTerminalEnvelopeV1::Returned(ash_core::Value::Bool(value)) => {
            json!({ "returned": { "Bool": value } })
        }
        CanonicalTerminalEnvelopeV1::Returned(ash_core::Value::Int(value)) => {
            json!({ "returned": { "Int": value } })
        }
        CanonicalTerminalEnvelopeV1::Returned(value) => {
            json!({ "returned": { "display": value.to_string() } })
        }
        CanonicalTerminalEnvelopeV1::Trapped(reason) => json!({ "trapped": reason }),
        CanonicalTerminalEnvelopeV1::AdmissionRejected => json!({ "admission_rejected": true }),
        CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact => {
            json!({ "invalid_checked_artifact": true })
        }
        CanonicalTerminalEnvelopeV1::TimedOut => json!({ "timed_out": true }),
        CanonicalTerminalEnvelopeV1::Cancelled => json!({ "cancelled": true }),
    }
}
