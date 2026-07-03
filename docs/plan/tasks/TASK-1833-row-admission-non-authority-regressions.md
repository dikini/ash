# TASK-1833: Prove row admission does not install authority

## Description

Strengthen negative tests that prove parse, check, import, and admission of row-bearing callables do not register providers, select resources, install runtime modules, admit roles, or grant policies.

## Owner decision gate

D7: What proves Phase 179 did not regress Phase 178 authority-neutrality?

## Requirements

- Extend existing Phase 178 authority-neutrality tests in `crates/ash-engine/tests/task_1822_row_authority_neutrality.rs` or add new tests.
- Verify that admission checks with row requirements do not mutate engine provider registry, resource initializer selections, operation implementation selections, or runtime module registry.
- Verify that rejection outcomes do not leave any authority residue.
- Verify that imported callables preserve the same non-authority invariant.

## Completion criteria

- [x] Negative tests cover parse, check, import, and admission phases.
- [x] Tests assert counts/lookups are zero before and after admission attempts.
- [x] Rejection tests assert no authority residue.
- [x] `cargo fmt --check`, `cargo clippy`, and `cargo test -p ash-engine` pass.

## Depends on

- TASK-1829, TASK-1830, TASK-1831, TASK-1832.
