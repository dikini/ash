# TASK-986 SPEC-073 Completion Closeout Evidence

**Status:** Draft evidence artifact seeded by TASK-985; TASK-986 owns final closeout, broad gates, independent review, and any SPEC-073 promotion.
**Date:** 2026-05-30
**Scope:** Acceptance-matrix command evidence for SPEC-073 A73-1 through A73-12 after TASK-985 integration proof.

## Evidence Matrix

| Row | Current evidence | TASK-986 closeout note |
| --- | --- | --- |
| A73-1 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture` passed 2 tests, including `task_985_source_archive_to_runtime_support_to_cleanup_flow_passes`; TASK-977 source archive target remains a focused row command. | Source install evidence is concrete; TASK-986 must reconcile status surfaces before promotion. |
| A73-2 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture` passed 2 tests, including explicit-digest `file://` tarball URL update evidence; TASK-979 remains focused policy evidence. | Binary tarball URL support remains explicit-digest only; no hosted registry or unbound release-index digest claim. |
| A73-3 | TASK-985 Ashgrove target passed 2 tests and checks source archive runtime-support payload presence; TASK-978 remains equivalence evidence for source and tarball manifests. | Runtime-support payload metadata is concrete. |
| A73-4 | TASK-985 Ashgrove target passed 2 tests and updates from old tarball toolchain to new immutable tarball URL toolchain without publishing on unsigned release-index failure. | Bare release-index resolver remains out of scope until signed entries bind toolchain id, URL, and digest. |
| A73-5 | TASK-985 Ashgrove target passed 2 tests and verifies packaged dispatcher lifecycle metadata selects the new manager toolchain after update. | Default/dispatcher behavior is selector-only; project manifests are not rewritten. |
| A73-6 | TASK-980 remains focused remove/running-manager evidence; TASK-985 tarball integration additionally removes the old inactive toolchain after dispatcher update while preserving the new manager owner. | TASK-986 should cite TASK-980 for refusal cases and TASK-985 for composed update/remove flow. |
| A73-7 | TASK-985 source integration verifies cleanup dry-run reports reachable fetched cache and protects the selected source toolchain; TASK-982 remains focused destructive/dry-run reachability evidence. | Cleanup remains conservative and project-local files are preserved. |
| A73-8 | TASK-985 CLI target passed 1 test and proves authenticated git dependency metadata is locked with credentials redacted before CLI use; TASK-981/TASK-984 remain focused metadata and remote policy evidence. | Hosted registry service and arbitrary SemVer solving remain non-goals. |
| A73-9 | TASK-984 lock signature/check evidence and TASK-981 metadata drift evidence remain focused row commands; TASK-985 CLI target consumes the locked authenticated dependency through installed selected-toolchain dispatch. | TASK-986 should keep drift/check evidence tied to focused commands. |
| A73-10 | TASK-985 CLI target passed 1 test and captures selected-toolchain `ASH_STDLIB_ROOT` plus `ASH_RUNTIME_SUPPORT_IDENTITY` from installed launcher dispatch before `ash check` and `ash run`. | Stdlib comes from selected toolchain, not dependency fetch. |
| A73-11 | TASK-984 trust/signing enforcement target remains focused security evidence; TASK-985 tarball integration composes unsigned release-index rejection, explicit digest URL update, and required tarball signature sidecar evidence. | SPEC-073 remains Draft; TASK-986 must avoid claiming signed release-index digest evidence beyond current explicit-digest boundary. |
| A73-12 | TASK-985 CLI target passed 1 test using installed `ash` to check and run a workflow importing a locked authenticated dependency from fetched cache; Phase 127 dependency-resolution target remains required. | TASK-986 must keep Phase 127 alpha dependency resolution green. |

## TASK-985 Focused Command Evidence

```text
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase128_release_deployment_acceptance -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Boundaries Preserved

- SPEC-073 remains Draft until TASK-986 completes broad gates, independent review, and status reconciliation.
- No hosted registry service is implemented or claimed.
- No arbitrary SemVer dependency solver is implemented or claimed.
- No global/system install roots are added.
- Release-index signature metadata is not accepted as tarball digest evidence; URL install/update remains explicit-digest only until a later resolver binds signed entries to toolchain id, tarball URL, and digest.
