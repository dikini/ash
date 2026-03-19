-- Ash Standard Library
--
-- This module provides the core types and functions for the Ash workflow language.

-- Core types
pub use option::{Option, Some, None};
pub use result::{Result, Ok, Err};

-- Helper functions
pub use option::{
    is_some,
    is_none,
    unwrap as unwrap_opt,
    unwrap_or as unwrap_or_opt,
    map as map_opt,
    and as and_opt,
    or as or_opt,
    ok_or
};

pub use result::{
    is_ok,
    is_err,
    unwrap as unwrap_res,
    unwrap_err,
    unwrap_or as unwrap_or_res,
    map as map_res,
    map_err,
    and_then,
    ok,
    err as err_opt
};
