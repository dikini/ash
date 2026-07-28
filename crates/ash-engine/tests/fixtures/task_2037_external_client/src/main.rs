//! External-client negative fixture for TASK-2037.

fn main() {
    let _ = ash_runtime::cps::validate::validate_cps_program;
    let _ = std::any::TypeId::of::<ash_engine::__TASK_2037_MODULE__::__TASK_2037_TYPE__>();
    let _ = __TASK_2037_RUNTIME_ACCESS__;
}
