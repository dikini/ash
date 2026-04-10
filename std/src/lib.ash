-- Ash Standard Library
--
-- This module provides the core types and functions for the Ash workflow language.

-- Core types
pub use option::{Option, Some, None};
pub use result::{Result, Ok, Err};
pub use runtime::{RuntimeError, Args};
pub use runtime::supervisor::{system_supervisor};

-- IO types
pub use io::{Error, ErrorKind, Result};
pub use io::path::{PathBuf, from_string, join, parent, file_name, extension, is_absolute};
pub use io::stdio::{Stdio, read_line, print, println};
pub use io::fs::{Fs, read, read_to_string, write, write_string, append, copy, rename, remove_file};
pub use io::dir::{Dir, create_dir, create_dir_all, remove_dir, remove_dir_all, read_dir};
pub use io::meta::{Metadata, Meta, metadata, is_file, is_dir, len, readonly};
pub use io::buf::{read_to_end, read_to_string, write_all, lines};

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
