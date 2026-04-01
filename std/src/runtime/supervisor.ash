-- Minimal stdlib-visible system supervisor scaffold for entry workflows

use super::error::RuntimeError;
use super::args::Args;

-- Keep the body as a placeholder until TASK-362 wires spawn/observation semantics.
pub workflow system_supervisor(args: cap Args) -> Int {
    ret 0;
}
