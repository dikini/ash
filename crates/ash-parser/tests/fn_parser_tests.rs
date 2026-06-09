//! Tests for fn definition, fn type, and fn body expression parsing.

#[path = "fn_parser_tests/basics.rs"]
mod basics;
#[path = "fn_parser_tests/closures.rs"]
mod closures;
#[path = "fn_parser_tests/contracts_and_types.rs"]
mod contracts_and_types;
#[path = "fn_parser_tests/control_flow_and_blocks.rs"]
mod control_flow_and_blocks;
#[path = "fn_parser_tests/support.rs"]
mod support;
#[path = "fn_parser_tests/task590_debug_cases.rs"]
mod task590_debug_cases;
