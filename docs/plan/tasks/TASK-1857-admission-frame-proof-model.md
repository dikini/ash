# TASK-1857: Add admission frame proof model

## Description

Add an admission-side proof model for operation row requirements discharged by handler/provider frames.

## Requirements

- Add tests first for provider-frame proof, handler-frame proof, frame-order shadowing, and missing operation discharge.
- Represent handler and provider frames as operation-discharge evidence.
- Preserve row non-grant behavior: frames prove authority only when supplied by the admission/runtime environment.

## Completion criteria

- [x] Tests fail before implementation and pass after.
- [x] Admission can prove an operation requirement from handler/provider frames.
- [x] Missing frame/provider authority produces a fail-closed diagnostic.

## Evidence

- RED: `cargo test -p ash-engine --test task_1857_admission_frame_proof_model` failed because `OperationAdmissionFrame`, `RowAdmissionEnvironment`, `RowAdmissionProof`, and `check_with_environment` did not exist. GREEN: the same command passes with 4 tests covering provider proof, handler proof, handler-over-provider frame shadowing, and missing-discharge diagnostics.

## Depends on

- TASK-1856.
