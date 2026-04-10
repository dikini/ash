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
    let base_str = match base {
        PathBuf { inner: b } => b
    };
    
    -- Simple path joining: if base ends with /, just concatenate
    -- otherwise add a separator
    let joined = if string::ends_with(base_str, "/") {
        string::concat(base_str, child)
    } else {
        string::concat(string::concat(base_str, "/"), child)
    };
    
    PathBuf { inner: joined }
}

-- Get the parent directory of a path
pub fn parent(path: PathBuf) -> Option<PathBuf> {
    let path_str = match path {
        PathBuf { inner: p } => p
    };
    
    -- Find the last / in the path
    match string::rfind(path_str, "/") {
        Some { value: idx } => {
            -- Extract everything before the last /
            let parent_str = string::substring(path_str, 0, idx);
            -- Handle edge cases
            if string::is_empty(parent_str) {
                if string::starts_with(path_str, "/") {
                    -- Root directory
                    Some { value: PathBuf { inner: "/" } }
                } else {
                    None
                }
            } else {
                Some { value: PathBuf { inner: parent_str } }
            }
        },
        None => None
    }
}

-- Get the file name component of a path
pub fn file_name(path: PathBuf) -> Option<String> {
    let path_str = match path {
        PathBuf { inner: p } => p
    };
    
    -- Find the last / in the path
    match string::rfind(path_str, "/") {
        Some { value: idx } => {
            -- Extract everything after the last /
            let len = string::length(path_str);
            let name = string::substring(path_str, idx + 1, len);
            if string::is_empty(name) {
                None
            } else {
                Some { value: name }
            }
        },
        None => {
            -- No separator, the whole path is the file name
            if string::is_empty(path_str) {
                None
            } else {
                Some { value: path_str }
            }
        }
    }
}

-- Get the file extension of a path
pub fn extension(path: PathBuf) -> Option<String> {
    let path_str = match path {
        PathBuf { inner: p } => p
    };
    
    -- Find the last . in the file name
    match string::rfind(path_str, ".") {
        Some { value: idx } => {
            -- Check if there's a / after the . (would be in a different component)
            match string::rfind(path_str, "/") {
                Some { value: slash_idx } => {
                    if slash_idx > idx {
                        None
                    } else {
                        let len = string::length(path_str);
                        Some { value: string::substring(path_str, idx + 1, len) }
                    }
                },
                None => {
                    let len = string::length(path_str);
                    Some { value: string::substring(path_str, idx + 1, len) }
                }
            }
        },
        None => None
    }
}

-- Check if a path is absolute (starts with /)
pub fn is_absolute(path: PathBuf) -> Bool {
    let path_str = match path {
        PathBuf { inner: p } => p
    };
    
    string::starts_with(path_str, "/")
}
