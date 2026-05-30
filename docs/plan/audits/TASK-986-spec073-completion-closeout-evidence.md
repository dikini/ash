# TASK-986 SPEC-073 Completion Closeout Evidence

**Status:** Final TASK-986 closeout evidence; SPEC-073 promoted to Implemented MVP after broad gates, status reconciliation, and independent review.
**Date:** 2026-05-30
**Scope:** Acceptance-matrix command evidence for SPEC-073 A73-1 through A73-12 after TASK-985 integration proof.

## Evidence Matrix

| Row | Current evidence | TASK-986 closeout note |
| --- | --- | --- |
| A73-1 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_977_source_archive_release_metadata -- --nocapture` passed in TASK-977; `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture` passed 2 tests, including `task_985_source_archive_to_runtime_support_to_cleanup_flow_passes`. | Source install evidence is concrete for git checkout/source archive paths, source metadata, and reproducibility markers. |
| A73-2 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_979_release_index_tarball_url_policy -- --nocapture` passed in TASK-979; `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture` passed 2 tests, including explicit-digest `file://` tarball URL update evidence. | Binary tarball URL support remains explicit-digest only; no hosted registry or signed release-index-as-digest claim. |
| A73-3 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_978_runtime_support_payload_metadata -- --nocapture` and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine --test task_978_runtime_support_payload -- --nocapture` passed in TASK-978; TASK-985 Ashgrove target passed 2 tests and checks source archive runtime-support payload presence. | Source and tarball installs have equivalent required toolchain contents including runtime-support metadata. |
| A73-4 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_979_release_index_tarball_url_policy -- --nocapture` passed in TASK-979; TASK-985 Ashgrove target passed 2 tests and updates from old tarball toolchain to a new immutable explicit-digest tarball URL toolchain without publishing on unsigned release-index failure. | Bare release-index resolver remains out of scope until signed entries bind toolchain id, URL, and digest. |
| A73-5 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_980_packaged_dispatcher_lifecycle -- --nocapture` passed in TASK-980; TASK-985 verifies packaged dispatcher lifecycle metadata selects the new manager toolchain after update. | Default/dispatcher behavior is selector-only; project manifests are not rewritten. |
| A73-6 | Phase 127 `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_971_remove_cleanup -- --nocapture` passed active/default/live-daemon/running-manager removal-protection coverage; TASK-980 focused test passed packaged running-manager owner protection; TASK-985 tarball integration removes the old inactive toolchain after dispatcher update while preserving the new manager owner. | Active/default/live/running-manager refusal cases and the composed update/remove flow have concrete evidence. |
| A73-7 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_982_cleanup_reachability -- --nocapture` passed in TASK-982; TASK-985 source integration verifies cleanup dry-run reports reachable fetched cache and protects the selected source toolchain. | Cleanup remains conservative, dry-run visible, and project-local files are preserved. |
| A73-8 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_981_registry_metadata_substrate -- --nocapture` and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_984_trust_signing_remote_git_policy -- --nocapture` passed in TASK-981/TASK-984; TASK-985 CLI target passed 1 test and proves authenticated git dependency metadata is locked with credentials redacted before CLI use. | Hosted registry service and arbitrary SemVer solving remain non-goals. |
| A73-9 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine --test task_981_registry_metadata_lock_consumers -- --nocapture` and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_984_trust_signing_remote_git_policy -- --nocapture` passed focused drift/signature evidence; TASK-985 CLI target consumes the locked authenticated dependency through installed selected-toolchain dispatch. | Drift/check evidence is concrete for both ashgrove and ash-engine lock consumers. |
| A73-10 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine --test task_978_runtime_support_payload -- --nocapture` passed runtime-support identity evidence; TASK-985 CLI target passed 1 test and captures selected-toolchain `ASH_STDLIB_ROOT` plus `ASH_RUNTIME_SUPPORT_IDENTITY` from installed launcher dispatch before `ash check` and `ash run`. | Stdlib comes from the selected toolchain, not dependency fetch. |
| A73-11 | `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_983_manifest_rewrite_trust_preservation -- --nocapture` and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_984_trust_signing_remote_git_policy -- --nocapture` passed preservation and mandatory fail-closed trust/signing enforcement; TASK-985 tarball integration composes unsigned release-index rejection, explicit digest URL update, and required tarball signature sidecar evidence. | The implemented boundary is mandatory fail-closed evidence for source archives, tarballs, lock signatures, release-index rejection, and remote git policy; signed release-index-as-digest evidence is not implemented or claimed. |
| A73-12 | Phase 127 `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture` remains required regression evidence and passed in TASK-984; TASK-985 CLI target passed 1 test using installed `ash` to check and run a workflow importing a locked authenticated dependency from fetched cache. | Locked git dependencies are visible to `ash check` and `ash run` through module/dependency root integration. |

## TASK-985 Focused Command Evidence

```text
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase128_release_deployment_acceptance -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## TASK-986 Broad Gate Evidence

Final TASK-986 verification commands:

```text
bash scripts/check-rust-format.sh
RUSTC_WRAPPER= bash scripts/check-rust-clippy.sh
RUSTC_WRAPPER= bash scripts/check-rust-tests.sh --workspace --all-targets
RUSTC_WRAPPER= bash scripts/check-doc-tests.sh
git diff --check
python3 -c "from pathlib import Path; files=[Path('docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md'),Path('docs/spec/README.md'),Path('docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md'),Path('docs/plan/PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md'),Path('docs/plan/PLAN-INDEX.md'),*sorted(Path('docs/plan/tasks').glob('TASK-97[5-9]-*.md')),*sorted(Path('docs/plan/tasks').glob('TASK-98[0-6]-*.md')),Path('docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md'),Path('docs/plan/audits/TASK-986-spec073-completion-closeout-evidence.md')]; missing=[str(p) for p in files if not p.exists()]; assert not missing, missing; spec=files[0].read_text(); closeout=Path('docs/plan/audits/TASK-986-spec073-completion-closeout-evidence.md').read_text(); assert all(f'A73-{n}' in spec and f'A73-{n}' in closeout for n in range(1,13)); print(f'checked {len(files)} SPEC-073/Phase128 status files')"
```

Results are recorded in the TASK-986 final report. The broad gates passed before the TASK-986 checklist was marked complete.

## Independent Review Evidence

An independent closeout review agent inspected the TASK-986 evidence artifact and status surfaces on 2026-05-30. Findings:

- A73-1 through A73-12 had current evidence or an explicit non-MVP boundary.
- A73-6 needed stronger citation of Phase 127/TASK-971 active/default/live/running-manager removal-protection evidence plus TASK-980 running-manager evidence; TASK-986 incorporated that citation in this artifact.
- Promotion required reconciling SPEC-073, docs/spec/README.md, PLAN-123, PLAN-INDEX, TASK-986, the TASK-986 evidence artifact, and CHANGELOG; TASK-986 reconciled those surfaces.
- The review identified stale SPEC-073 trust/signing text that conflicted with A73-11 mandatory enforcement; TASK-986 amended §4.1/§4.2/§18 and kept the release-index digest boundary explicit.
- The review confirmed hosted registry, global/system roots, OS package-manager integration, arbitrary SemVer solving, and Phase 127 historical partial language were the key overclaim risks to preserve.

## Boundaries Preserved

- SPEC-073 is Implemented MVP after TASK-986 closeout.
- No hosted registry service is implemented or claimed.
- No arbitrary SemVer dependency solver is implemented or claimed.
- No global/system install roots are added.
- No OS package-manager integration is implemented or claimed.
- Release-index signature metadata is not accepted as tarball digest evidence; URL install/update remains explicit-digest only until a later resolver binds signed entries to toolchain id, tarball URL, and digest.
- Phase 127 remains the historical partial closeout; Phase 128 owns the successor completion evidence.
