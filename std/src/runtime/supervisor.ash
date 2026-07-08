-- Stdlib-visible system supervisor contract for entry definitions

use result::{Result, Err};
use error::RuntimeError;
use args::Args;

-- The runtime owns spawning `main(args)` and observing terminal completion.
-- TASK-363c wires that bootstrap behavior; this module only shapes the terminal exit code.
pub fn system_supervisor(args: capability Args) -> Int {
    -- Runtime-provided `completion : Result<(), RuntimeError>` from `main(args)`.
    -- Canonical runtime payload shape: `Err { error: RuntimeError(code, _) }`.
    let exit_code = if let Err { error: RuntimeError(code, _) } = completion then code else 0;

    exit_code
}
