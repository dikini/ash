//! TASK-2042 contracts for Engine-owned descriptor rejection terminals.

use ash_engine::{CanonicalTerminalEnvelopeV1, SubmittedDescriptorPreExecutionRejection};

#[test]
fn submitted_descriptor_pre_execution_rejections_have_engine_owned_terminals() {
    assert_eq!(
        SubmittedDescriptorPreExecutionRejection::InvalidDescriptor.canonical_terminal_envelope(),
        CanonicalTerminalEnvelopeV1::invalid_checked_artifact(),
    );
    assert_eq!(
        SubmittedDescriptorPreExecutionRejection::HostAdmissionRejected
            .canonical_terminal_envelope(),
        CanonicalTerminalEnvelopeV1::admission_rejected(),
    );
}
