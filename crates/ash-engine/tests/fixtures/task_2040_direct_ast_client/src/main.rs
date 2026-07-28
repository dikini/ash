//! External-client compile probe for TASK-2040.

fn main() {
    let expression = ash_core::Expr::Literal(ash_core::Value::Int(42));
    let context = __TASK_2040_RUNTIME_CRATE__::Context::new();
    let _value = __TASK_2040_RUNTIME_CRATE__::__TASK_2040_DIRECT_AST_API__(&expression, &context);
}
