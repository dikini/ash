-- Filesystem file operations
--
-- Provides functions for reading and writing files.
-- All functions require the Fs capability.

use path::PathBuf;

-- Fs capability for file operations
pub capability Fs: observe read(path: PathBuf) returns Bytes
                 | observe read_to_string(path: PathBuf) returns String
                 | execute write(path: PathBuf, content: Bytes)
                 | execute write_string(path: PathBuf, content: String)
                 | execute append(path: PathBuf, content: Bytes)
                 | execute copy(from: PathBuf, to: PathBuf)
                 | execute rename(from: PathBuf, to: PathBuf)
                 | execute remove_file(path: PathBuf);

-- Read file contents as bytes
pub fn read(path: PathBuf) -> Bytes {
    act observe Fs.read with path: path
}

-- Read file contents as a string
pub fn read_to_string(path: PathBuf) -> String {
    act observe Fs.read_to_string with path: path
}

-- Write bytes to a file (overwrites existing)
pub fn write(path: PathBuf, content: Bytes) {
    act execute Fs.write with path: path, content: content;
}

-- Write a string to a file (overwrites existing)
pub fn write_string(path: PathBuf, content: String) {
    act execute Fs.write_string with path: path, content: content;
}

-- Append bytes to a file
pub fn append(path: PathBuf, content: Bytes) {
    act execute Fs.append with path: path, content: content;
}

-- Copy a file from one location to another
pub fn copy(from: PathBuf, to: PathBuf) {
    act execute Fs.copy with from: from, to: to;
}

-- Rename/move a file from one location to another
pub fn rename(from: PathBuf, to: PathBuf) {
    act execute Fs.rename with from: from, to: to;
}

-- Remove a file
pub fn remove_file(path: PathBuf) {
    act execute Fs.remove_file with path: path;
}
