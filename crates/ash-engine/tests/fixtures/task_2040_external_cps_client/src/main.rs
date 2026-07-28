//! External-client compile probe for TASK-2040's non-Engine CPS removal.

fn main() {
    let _ = ash_engine::private_cps::eval_checked_terminal;
}
