# TASK-990 Ashgrove Source Payload Local-State Closeout

## Status

Complete for Phase 129 closeout on 2026-05-31. SPEC-074 A74-1 through A74-8 have concrete implementation, focused regression, source-archive non-regression, broad ashgrove gate, and review evidence. This closeout does not expand scope beyond the SPEC-074 source-root payload/local-state amendment to the SPEC-073 Implemented MVP.

## Controller-run command evidence

| Command | Result | Evidence |
| --- | --- | --- |
| `git diff --check` | PASS | Exited 0 with no whitespace errors. |
| `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture` | PASS | Exited 0; 10 tests passed, 0 failed. |
| `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture` | PASS | Exited 0; filtered source-archive coverage passed, including TASK-977 release metadata, TASK-984 attestation/trust failures, TASK-985 release/deployment source-archive flow, and TASK-989 source-archive digest-policy regression. |
| `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove` | PASS | Exited 0; all default ashgrove unit, integration, and doctest targets passed, including TASK-989's 10 focused tests and the existing corrupt `.git` metadata regression. |
| `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --all-targets -- --nocapture` | PASS | Exited 0; all ashgrove unit/integration targets passed, including the TASK-989 target with 10 passing tests and the existing corrupt `.git` metadata regression. |
| `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings` | PASS | Exited 0; no clippy warnings. |
| `cargo fmt --all --check` | PASS | Exited 0 after closeout docs/status patches. |
| `python3 -c 'from pathlib import Path; audit=Path("docs/plan/audits/TASK-990-ashgrove-source-payload-local-state-closeout.md"); assert audit.exists(), audit; text=audit.read_text(); required=["A74-1","A74-8","independent review","cargo"]; missing=[s for s in required if s not in text]; assert not missing, missing; print("TASK-990 closeout artifact verified")'` | PASS | Exited 0 and printed `TASK-990 closeout artifact verified`. |

## Acceptance reconciliation

| ID | Closeout status | Evidence |
| --- | --- | --- |
| A74-1 | PASS | `task_989_gitignored_agents_state_can_change_during_source_install` mutates `.agents/status/dashboard.json` during fake cargo execution, observes the ignored path absent from the isolated copy, and succeeds without `--allow-dirty-source`. The focused TASK-989 command passed with 10 tests. |
| A74-2 | PASS | `task_989_gitignored_nested_target_is_excluded_from_digest_and_copy` mutates an ignored nested `crates/ash-bench/target/generated.txt`, observes it absent from the isolated copy, and succeeds. |
| A74-3 | PASS | `task_989_nonignored_payload_mutation_fails_before_publish` mutates `std/src/lib.ash` during build, fails with `source-payload-changed`, and asserts no final toolchain is published. |
| A74-4 | PASS | `task_989_nonignored_dirty_source_still_rejects_without_override` rejects a nonignored untracked file without `--allow-dirty-source`. Existing TASK-968 dirty/corrupt git tests also passed in the broad ashgrove run. |
| A74-5 | PASS | `task_989_source_archive_digest_policy_does_not_use_source_root_ignores` proves source-shaped archives keep `source_archive_digest` and do not receive source-root payload metadata. The `source_archive` filtered command also passed existing TASK-977/TASK-984/TASK-985 source-archive release metadata, attestation, and release/deployment coverage. |
| A74-6 | PASS | Implementation review found live source-root digest and copy share `SourceRootBuildPayload::LiveRoot { files, digest, ... }`: `SourceRootBuildPayload::inspect` computes `source_root_payload_files`, `digest_source_files` consumes that list, and `copy_for_build` passes the same `files` to `copy_source_payload_files_for_build`. |
| A74-7 | PASS | `task_989_update_from_source_uses_same_payload_policy_as_install` exercises `ashgrove update --from source --path ... --to ...` with ignored `.agents/` churn and verifies the same source-root payload metadata/copy exclusion behavior as install. |
| A74-8 | PASS | The reported local checkout failure mode is covered by deterministic equivalent regression: `.agents/status/dashboard.json` changes during source install while git status remains clean, the ignored file is excluded from the isolated source-build copy, and install succeeds. This is the same class as the observed `source cargo build dirtied source root ...` false abort. |

## Review status

Independent review: PASS. The closeout review checked SPEC-074 against the TASK-989 implementation and tests for source-root/source-archive policy separation, source-shaped archive handling, shared digest/copy membership, fail-closed nonignored mutation behavior, fake-cargo observation plumbing, update parity, and docs/status consistency.

Quality review blockers are resolved. The prior corrupt `.git` regression is specifically preserved by `task_968_source_install_rejects_corrupt_git_metadata_even_with_unidentified_override`, which passed in the broad ashgrove test gate. TASK-989 also added and passed direct regressions for git payload membership failure and git worktree classification failure, preventing fallback to non-git walking when git-like source roots cannot provide reliable membership.

No open blockers remain for Phase 129 closeout. SPEC-073 remains historical Implemented MVP; SPEC-074 owns only the post-MVP source-payload/local-state amendment.
