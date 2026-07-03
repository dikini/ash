# TASK-1827: Audit row metadata against admission/runtime authority paths

## Description

Audit the current runtime, admission, provider registry, resource initializer, role, policy, and workflow execution paths against the explicit row metadata produced by Phase 178. The goal is to identify exactly which existing admission checks can consume explicit row requirements without leaking authority, and which paths must remain fail-closed.

## Owner decision gate

D1: Which current runtime/admission paths can consume explicit row requirements without authority leakage?

## Requirements

- Read `crates/ash-engine/src/lib.rs`, `crates/ash-engine/src/module_loader.rs`, `crates/ash-core/src/core_ash.rs`, `crates/ash-core/src/runtime.rs`, `crates/ash-core/src/runtime_kernel.rs`, and relevant interpreter admission paths.
- Map `CoreRowItem` variants to existing authority concepts.
- Identify exact public APIs for provider lookup, capability admission, resource initializer selection, role admission, and policy admission.
- Record concrete hazards where row metadata could be mistaken for authority.
- Deliver a structured report as the task output.

## Completion criteria

- [x] Report covers admission paths, authority registration APIs, row metadata carriers, gaps/hazards, and implementation recommendations for TASK-1828 through TASK-1831.
- [x] Report references exact file paths and symbol names.
- [x] Report is reviewed by the main agent before TASK-1828 begins.
- [x] Implemented in `crates/ash-engine/src/row_admission.rs` and verified by `task_1829_1830_1831_1832_1833_row_admission.rs`.
