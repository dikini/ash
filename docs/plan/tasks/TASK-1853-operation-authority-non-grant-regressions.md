# TASK-1853: Prove rows do not grant authority

## Description

Add or strengthen regressions proving row metadata never registers operation authority or invokes providers/handlers during admission.

## Requirements

- Preserve existing Phase 179 non-authority checks.
- Add Phase 183 checks for operation authority discharge wording and no provider execution.
- Ensure parse/check/import of row-bearing callables remains metadata-only.

## Completion criteria

- [x] Regression tests pass.
- [x] Rows do not mutate provider/resource/role/policy/evidence/failure state.

## Evidence

- Preserved Phase 179 `row_admission_does_not_install_authority_or_call_host_hooks` regression and updated operation terminology to `authority`. Verified with `cargo test -p ash-engine --test task_1829_1830_1831_1832_1833_row_admission`.

## Depends on

- TASK-1851; TASK-1852.
