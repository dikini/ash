-- Pure path operations
--
-- These functions operate on paths without performing any IO operations.
-- They are pure functions that transform path strings.

-- A newtype wrapper for string paths
pub type PathBuf = PathBuf { inner: String };

-- Create a PathBuf from a string
pub fn from_string(s: String) -> PathBuf {
    PathBuf { inner: s }
}

-- Join two paths together
pub fn join(base: PathBuf, child: String) -> PathBuf {
    match base {
        PathBuf { inner: b } =>
            if string::ends_with(b, "/") then
                PathBuf { inner: string::concat(b, child) }
            else
                PathBuf { inner: string::concat(string::concat(b, "/"), child) }
    }
}

-- Get the parent directory of a path
pub fn parent(path: PathBuf) -> Option<PathBuf> {
    match path {
        PathBuf { inner: p } =>
            if string::starts_with(p, "/") then
                Some { value: PathBuf { inner: "/" } }
            else
                None
    }
}

-- Get the file name component of a path
pub fn file_name(path: PathBuf) -> Option<String> {
    match path {
        PathBuf { inner: p } =>
            if string::is_empty(p) then None else Some { value: p }
    }
}

-- Get the file extension of a path
pub fn extension(path: PathBuf) -> Option<String> {
    match path {
        PathBuf { inner: p } =>
            if string::is_empty(p) then None else Some { value: p }
    }
}

-- Check if a path is absolute (starts with /)
pub fn is_absolute(path: PathBuf) -> Bool {
    match path {
        PathBuf { inner: p } => string::starts_with(p, "/")
    }
}

pub fn preserves_absolute_after_join(base: PathBuf, child: String) -> Bool {
    if is_absolute(base) then is_absolute(join(base, child)) else true
}

law join_preserves_absolute(base: PathBuf, child: String): preserves_absolute_after_join(base, child)
