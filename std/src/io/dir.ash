-- Filesystem directory operations
--
-- Provides functions for creating, removing, and listing directories.
-- All functions require the Dir capability.

use path::PathBuf;
use list::List;

-- Dir capability for directory operations
pub capability Dir: execute create_dir(path: PathBuf)
                  | execute create_dir_all(path: PathBuf)
                  | execute remove_dir(path: PathBuf)
                  | execute remove_dir_all(path: PathBuf)
                  | observe read_dir(path: PathBuf) returns List<String>;

-- Create a directory
pub fn create_dir(path: PathBuf) {
    act execute Dir.create_dir with path: path;
}

-- Create a directory and all parent directories
pub fn create_dir_all(path: PathBuf) {
    act execute Dir.create_dir_all with path: path;
}

-- Remove an empty directory
pub fn remove_dir(path: PathBuf) {
    act execute Dir.remove_dir with path: path;
}

-- Remove a directory and all its contents
pub fn remove_dir_all(path: PathBuf) {
    act execute Dir.remove_dir_all with path: path;
}

-- Read directory entries, returning a list of file/directory names
pub fn read_dir(path: PathBuf) -> List<String> {
    act observe Dir.read_dir with path: path
}
