-- Buffered I/O helpers
--
-- Provides convenience functions for common buffered I/O operations.

use path::PathBuf;

-- Read entire file contents as bytes
pub fn read_to_end(path: PathBuf) -> Bytes {
    fs::read(path)
}

-- Read entire file contents as a string
pub fn read_to_string(path: PathBuf) -> String {
    fs::read_to_string(path)
}

-- Write all bytes to a file (overwrites existing)
pub fn write_all(path: PathBuf, content: Bytes) {
    fs::write(path, content);
}

-- Split text into lines
pub fn lines(text: String) -> List<String> {
    -- TODO: Implement string splitting when string module is available
    panic "lines: not yet implemented"
}
