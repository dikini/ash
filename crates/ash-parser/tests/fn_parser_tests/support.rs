#![allow(unused_imports)]

pub use ash_parser::input::new_input;
pub use ash_parser::lower::lower_expr;
pub use ash_parser::parse_module::parse_fn_definition;
pub use ash_parser::surface::{BlockStmt, Definition, Expr, Type};

// ---------------------------------------------------------------------------
// Helper: parse a fn definition from source text
// ---------------------------------------------------------------------------
pub fn parse_fn(input_str: &str) -> Definition {
    let mut input = new_input(input_str);
    parse_fn_definition(&mut input).expect("fn definition should parse")
}

// ---------------------------------------------------------------------------
// 1. Simple fn definition
// ---------------------------------------------------------------------------
