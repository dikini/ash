//! Tests for TASK-621 (Runtime Builtin Dispatch Table) and
//! TASK-622 (Clear Error on Unknown Builtin).

#[path = "builtin_dispatch/dispatch_table.rs"]
mod dispatch_table;
#[path = "builtin_dispatch/host_hook_metadata.rs"]
mod host_hook_metadata;
#[path = "builtin_dispatch/list.rs"]
mod list;
#[path = "builtin_dispatch/markdown.rs"]
mod markdown;
#[path = "builtin_dispatch/predicates.rs"]
mod predicates;
#[path = "builtin_dispatch/process_run.rs"]
mod process_run;
#[path = "builtin_dispatch/string_regex.rs"]
mod string_regex;
#[path = "builtin_dispatch/support.rs"]
mod support;
