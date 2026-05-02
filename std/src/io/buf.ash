-- Buffered I/O helpers
--
-- Provides parser-checkable runtime-provided declarations for common buffered
-- I/O operations. Concrete wrappers over `io::fs` remain deferred until the
-- parser/runtime support a canonical stdlib effect-wrapper spelling.

use path::PathBuf;

-- Read entire file contents as bytes
pub builtin fn read_to_end(path: PathBuf) -> Bytes;

-- Read entire file contents as a string
pub builtin fn read_to_string(path: PathBuf) -> String;

-- Write all bytes to a file (overwrites existing)
pub builtin fn write_all(path: PathBuf, content: Bytes) -> Unit;

-- Split text into lines
pub builtin fn lines(text: String) -> List<String>;
