-- IO module root
--
-- Provides types and functions for input/output operations.

-- Error kind classification for IO operations
pub type ErrorKind = NotFound | PermissionDenied | InvalidInput | Other;

-- IO error type with kind and message
pub type Error = Error { kind: ErrorKind, message: String };

-- Result type alias for IO operations
pub type Result<T> = Ok { value: T } | Err { error: Error };

-- Path module for pure path operations
pub mod path;

-- Stdio module for standard I/O operations
pub mod stdio;

-- Filesystem file operations
pub mod fs;

-- Filesystem directory operations
pub mod dir;

-- Filesystem metadata operations
pub mod meta;

-- Buffered I/O helpers
pub mod buf;

-- Re-exports from path module
pub use path::{PathBuf, from_string, join, parent, file_name, extension, is_absolute};

-- Re-exports from stdio module
pub use stdio::{Stdio, read_line, print, println};

-- Re-exports from fs module
pub use fs::{Fs, read, read_to_string, write, write_string, append, copy, rename, remove_file};

-- Re-exports from dir module
pub use dir::{Dir, create_dir, create_dir_all, remove_dir, remove_dir_all, read_dir};

-- Re-exports from meta module
pub use meta::{Metadata, Meta, metadata, is_file, is_dir, len, readonly};

-- Re-exports from buf module
pub use buf::{read_to_end, read_to_string, write_all, lines};
