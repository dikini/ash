-- Standard library prelude
-- Automatically imported in all modules

use option::{Option, Some, None};
use result::{Result, Ok, Err};

-- Re-export commonly used functions
pub use option::{is_some, is_none, unwrap, unwrap_or, map, and, or, ok_or};
pub use result::{is_ok, is_err, unwrap as unwrap_res, unwrap_err, unwrap_or as unwrap_or_res, map as map_res, map_err, and_then, ok, err as err_opt};
