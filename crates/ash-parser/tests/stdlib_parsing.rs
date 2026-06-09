//! Integration tests for stdlib parsing
//!
//! These tests verify that the standard library .ash files can be parsed correctly.

#[path = "stdlib_parsing/core_files.rs"]
mod core_files;
#[path = "stdlib_parsing/io_buf.rs"]
mod io_buf;
#[path = "stdlib_parsing/io_fs_dir_meta.rs"]
mod io_fs_dir_meta;
#[path = "stdlib_parsing/io_mod.rs"]
mod io_mod;
#[path = "stdlib_parsing/io_path.rs"]
mod io_path;
#[path = "stdlib_parsing/io_stdio.rs"]
mod io_stdio;
#[path = "stdlib_parsing/option_result_prelude.rs"]
mod option_result_prelude;
#[path = "stdlib_parsing/runtime.rs"]
mod runtime;
#[path = "stdlib_parsing/support.rs"]
mod support;
