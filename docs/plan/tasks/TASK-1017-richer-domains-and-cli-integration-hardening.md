# TASK-1017: Richer Domains and CLI Integration Hardening

## Status: Complete

## Description

Add richer finite domain families and harden synthesized/small-world CLI controls across ordinary source, structured snapshot, generated property, and world execution paths.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Add bounded product and bounded list domains with explicit size caps.
2. Add role/capability inclusion-set worlds where metadata exposes finite roles/capabilities.
3. Add policy-context and obligation lifecycle state-machine descriptors where metadata is stable.
4. Preserve fail-closed behavior for uncapped or open domains.
5. Verify filters, source selection, fail-fast, seed, max-cases, max-worlds, timeout, human output, and JSON output across synthesized paths.

## TDD Steps

- RED: Add failing tests for each new finite domain and CLI integration behavior.
- GREEN: Implement domain enumeration and CLI behavior incrementally.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused domain and CLI tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`
- `git diff --check`

## Evidence

### RED

- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli smallworld -- --nocapture` failed before implementation with 29 compile errors proving the new tests named missing richer-domain API surface: `SmallWorldProductAxis`, `SmallWorldListDescriptor`, `SmallWorldInclusionSetDescriptor`, `SmallWorldPolicyContextDescriptor`, `SmallWorldPolicyContext`, `SmallWorldLifecycleDescriptor`, `SmallWorldLifecycleStateDescriptor`, `SmallWorldDomainKind::{Product,List,RoleCapabilityInclusionSet,PolicyContext,ObligationLifecycle}`, and the corresponding `SmallWorldDomain` descriptor fields.

### GREEN

- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli bounded_product_domain_materializes_cartesian_world_bindings -- --nocapture`: 1 passed, 0 failed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli oversized_product_domain_defers_before_deep_axis_recursion -- --nocapture`: failed before remediation because a product descriptor with 65 axes recursed through every axis and emitted a pass row, then passed after adding an explicit product-axis cap.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli oversized_bounded_list_domain_defers_before_deep_materialization -- --nocapture`: failed before remediation because an oversized list descriptor materialized a length-65 world, then passed after adding an explicit list-length cap.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli policy_and_lifecycle_worlds_require_stable_explicit_ids -- --nocapture`: failed before remediation because policy/lifecycle descriptors with empty IDs received fallback IDs, then passed after requiring explicit stable IDs.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli synthesized_result_exceeding_timeout_is_reported_as_timeout -- --nocapture`: failed before remediation because a synthesized row with duration beyond the configured timeout remained a pass row, then passed after synthesized-result timeout classification.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`: 88 passed, 0 failed.

### Final Verification

- `cargo fmt --check`: pass.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli smallworld -- --nocapture`: 5 lib tests passed, 2 `test_command` integration tests passed, 0 failed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`: 88 passed, 0 failed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`: 29 passed, 0 failed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`: pass.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`: pass.
- `git diff --check`: pass.

## Implemented Scope

- Added explicit runner metadata for bounded product domains, bounded list domains, finite role/capability inclusion sets, stable policy contexts, and stable obligation lifecycle descriptors.
- Materialized only finite, explicitly capped richer domains through `--max-worlds` or `max_worlds_default`, plus fixed product-axis and list-length caps; uncapped/open/oversized descriptors defer before world enumeration.
- Preserved target-output-oracle execution for pass/fail and repro artifacts with materialized world snapshots plus replay controls.
- Reused existing synthesized CLI routing and added CLI integration coverage for `--fail-fast`, `--timeout`, seed, max-cases, max-worlds, human output, and JSON output on synthesized contract rows; this task did not add arbitrary source inference for these richer domains.

## Limitations

- Richer domains require explicit stable metadata; no inference from arbitrary Ash source is introduced here.
- Small-world target execution remains the narrow pure-expression/literal target-output slice. Role/capability/policy/obligation fields are materialized as world metadata snapshots and bindings for supported pure targets, not as a full policy/capability runtime.

## Completion Checklist

- [x] Richer finite domains implemented with safe caps.
- [x] Open domains defer before materialization.
- [x] CLI behavior is consistent across synthesized paths.
- [x] RED/GREEN evidence recorded.
