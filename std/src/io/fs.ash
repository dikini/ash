-- Filesystem file operations
--
-- Provides runtime-backed function declarations for reading and writing files.
-- Phase 198 profiles admit explicit filesystem rows and enforce
-- sandbox/provenance policy at the projected provider boundary.

use path::PathBuf;

-- Read file contents as bytes
pub builtin fn read(path: PathBuf) -> Bytes;

-- Read file contents as a string
pub builtin fn read_to_string(path: PathBuf) -> String;

-- Write bytes to a file (overwrites existing)
pub builtin fn write(path: PathBuf, content: Bytes) -> Unit;

-- Write a string to a file (overwrites existing)
pub builtin fn write_string(path: PathBuf, content: String) -> Unit;

-- Append bytes to a file
pub builtin fn append(path: PathBuf, content: Bytes) -> Unit;

-- Copy a file from one location to another
pub builtin fn copy(from: PathBuf, to: PathBuf) -> Unit;

-- Rename/move a file from one location to another
pub builtin fn rename(from: PathBuf, to: PathBuf) -> Unit;

-- Remove a file
pub builtin fn remove_file(path: PathBuf) -> Unit;
