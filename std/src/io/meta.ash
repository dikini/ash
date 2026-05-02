-- Filesystem metadata operations
--
-- Provides parser-checkable runtime-provided metadata declarations. The Meta
-- capability below records the intended authority contract; concrete
-- capability-wrapper bodies remain deferred until the parser/runtime support a
-- canonical stdlib `act` wrapper spelling.

use path::PathBuf;

-- Metadata for a file or directory
pub type Metadata = Metadata {
    is_file: Bool,
    is_dir: Bool,
    len: Int,
    readonly: Bool
};

-- Meta capability for metadata operations
pub capability Meta: observe metadata(path: PathBuf) returns Metadata;

-- Get metadata for a file or directory
pub builtin fn metadata(path: PathBuf) -> Metadata;

-- Check if path points to a file (convenience function)
pub fn is_file(path: PathBuf) -> Bool {
    let meta = metadata(path);
    match meta {
        Metadata { is_file: f, .. } => f
    }
}

-- Check if path points to a directory (convenience function)
pub fn is_dir(path: PathBuf) -> Bool {
    let meta = metadata(path);
    match meta {
        Metadata { is_dir: d, .. } => d
    }
}

-- Get the size of a file in bytes (convenience function)
pub fn len(path: PathBuf) -> Int {
    let meta = metadata(path);
    match meta {
        Metadata { len: l, .. } => l
    }
}

-- Check if a file is read-only (convenience function)
pub fn readonly(path: PathBuf) -> Bool {
    let meta = metadata(path);
    match meta {
        Metadata { readonly: r, .. } => r
    }
}
