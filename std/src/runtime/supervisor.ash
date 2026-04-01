-- Stdlib-visible system supervisor contract for entry workflows

use result::{Result, Err};
use super::error::RuntimeError;
use super::args::Args;

-- The runtime owns spawning `main(args)` and observing terminal completion.
-- TASK-363c wires that bootstrap behavior; this module only shapes the terminal exit code.
pub workflow system_supervisor(args: cap Args) -> Int {
    -- Runtime-provided `completion : Result<(), RuntimeError>` from `main(args)`.
    -- Canonical runtime payload shape: `Err { error: RuntimeError { exit_code: code, message: _ } }`.
    let exit_code=if let Err { error: RuntimeError { exit_code: code, message: _ } } = completion then code else 0;

    ret exit_code;
}
