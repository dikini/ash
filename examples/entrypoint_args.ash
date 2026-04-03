-- Canonical Phase 57 entry example with runtime-provided CLI args.

use result::Result
use runtime::RuntimeError
use runtime::Args

workflow main(args: cap Args) -> Result<(), RuntimeError> {
  observe Args 0 as _;
  done;
}
