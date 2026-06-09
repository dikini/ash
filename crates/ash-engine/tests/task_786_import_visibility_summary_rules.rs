//! TASK-786 import visibility and semantic-summary integration tests.

#[path = "task_786_import_visibility_summary_rules/callable_reexports.rs"]
mod callable_reexports;
#[path = "task_786_import_visibility_summary_rules/diagnostics.rs"]
mod diagnostics;
#[path = "task_786_import_visibility_summary_rules/glob_imports.rs"]
mod glob_imports;
#[path = "task_786_import_visibility_summary_rules/named_imports.rs"]
mod named_imports;
#[path = "task_786_import_visibility_summary_rules/public_signatures.rs"]
mod public_signatures;
#[path = "task_786_import_visibility_summary_rules/support.rs"]
mod support;
#[path = "task_786_import_visibility_summary_rules/type_reexports.rs"]
mod type_reexports;
