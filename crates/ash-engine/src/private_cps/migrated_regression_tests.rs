//! Regression coverage migrated with the private checked-CPS kernel.
//!
//! These were direct public `ash_interp::cps` integration tests. Keeping them
//! below the Engine-private module preserves their kernel coverage without a
//! public or test-only evaluator escape hatch.

#[path = "tests/task_1590_cps_ir.rs"]
mod task_1590_cps_ir;
#[path = "tests/task_1591_cps_ir.rs"]
mod task_1591_cps_ir;
#[path = "tests/task_1592_cps_ir.rs"]
mod task_1592_cps_ir;
#[path = "tests/task_1593_cps_ir.rs"]
mod task_1593_cps_ir;
#[path = "tests/task_1594_cps_ir.rs"]
mod task_1594_cps_ir;
#[path = "tests/task_1595_cps_ir.rs"]
mod task_1595_cps_ir;
#[path = "tests/task_1596_cps_ir.rs"]
mod task_1596_cps_ir;
#[path = "tests/task_1599_cps_ir.rs"]
mod task_1599_cps_ir;
#[path = "tests/task_1616_cps_ir_speculative_fixtures.rs"]
mod task_1616_cps_ir_speculative_fixtures;
#[path = "tests/task_1616b_cps_ir_correctness_fixes.rs"]
mod task_1616b_cps_ir_correctness_fixes;
#[path = "tests/task_1663_cps_runtime_scaffold.rs"]
mod task_1663_cps_runtime_scaffold;
#[path = "tests/task_1664_cps_force_runtime.rs"]
mod task_1664_cps_force_runtime;
#[path = "tests/task_1672_cps_thunk_trace_observability.rs"]
mod task_1672_cps_thunk_trace_observability;
#[path = "tests/task_1682_cps_multishot_runtime.rs"]
mod task_1682_cps_multishot_runtime;
#[path = "tests/task_1683_cps_multishot_validation.rs"]
mod task_1683_cps_multishot_validation;
#[path = "tests/task_1858_1859_handler_provider_semantics.rs"]
mod task_1858_1859_handler_provider_semantics;
#[path = "tests/task_1993_frame_ordered_dispatch.rs"]
mod task_1993_frame_ordered_dispatch;
#[path = "tests/task_2003_cps_terminal_projection.rs"]
mod task_2003_cps_terminal_projection;
#[path = "tests/task_2003_source_return_cps_lowering.rs"]
mod task_2003_source_return_cps_lowering;
#[path = "tests/task_2014_handler_inspection_cps_gap.rs"]
mod task_2014_handler_inspection_cps_gap;
