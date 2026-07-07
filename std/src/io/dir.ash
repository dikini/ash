-- Filesystem directory operations
--
-- Provides runtime-backed function declarations for directory access. The Dir
-- capability records the intended authority contract; Phase 198 profiles admit
-- explicit directory rows and enforce sandbox/provenance policy at the projected
-- provider boundary.

use path::PathBuf;

-- Dir capability for directory operations
pub capability Dir: execute create_dir(path: PathBuf)
                  | execute create_dir_all(path: PathBuf)
                  | execute remove_dir(path: PathBuf)
                  | execute remove_dir_all(path: PathBuf)
                  | observe read_dir(path: PathBuf) returns List<String>;

-- Create a directory
pub builtin fn create_dir(path: PathBuf) -> Unit;

-- Create a directory and all parent directories
pub builtin fn create_dir_all(path: PathBuf) -> Unit;

-- Remove an empty directory
pub builtin fn remove_dir(path: PathBuf) -> Unit;

-- Remove a directory and all its contents
pub builtin fn remove_dir_all(path: PathBuf) -> Unit;

-- Read directory entries, returning a list of file/directory names
pub builtin fn read_dir(path: PathBuf) -> List<String>;
