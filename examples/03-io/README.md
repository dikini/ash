# IO Examples

This directory contains examples demonstrating Ash's IO standard library.

## Files

### path_operations.ash
Demonstrates pure path operations using `io::path`:
- Creating paths from strings
- Joining paths together
- Extracting parent directories
- Getting file names and extensions
- Checking if paths are absolute

```bash
ash check examples/03-io/path_operations.ash
ash run examples/03-io/path_operations.ash
```

### file_read_write.ash
Demonstrates file operations using `io::fs` and `io::stdio`:
- Reading file contents as strings
- Writing content to files
- Using stdio for console output

```bash
ash check examples/03-io/file_read_write.ash
ash run examples/03-io/file_read_write.ash
```

### directory_listing.ash
Demonstrates directory operations using `io::dir` and `io::meta`:
- Creating directories (including nested paths)
- Querying file metadata
- Checking if paths are files or directories

```bash
ash check examples/03-io/directory_listing.ash
ash run examples/03-io/directory_listing.ash
```

## IO Module Overview

The Ash standard library provides a comprehensive IO module with these submodules:

| Module | Purpose | Capability |
|--------|---------|------------|
| `io::path` | Pure path operations (no IO) | None |
| `io::stdio` | Standard input/output | `Stdio` |
| `io::fs` | File read/write operations | `Fs` |
| `io::dir` | Directory operations | `Dir` |
| `io::meta` | File/directory metadata | `Meta` |
| `io::buf` | Buffered I/O helpers | Uses `Fs` |

## Key Concepts

1. **PathBuf** - A type-safe wrapper for file paths
2. **Capabilities** - All IO operations require explicit capabilities
3. **Pure Functions** - `path` module functions are pure (no side effects)
4. **Observe vs Execute** - File reads are observations, writes are executions

## Example: Combined IO Operations

```ash
use io::path;
use io::fs;
use io::dir;
use io::meta;
use io::stdio;

workflow main() {
    -- Build a path
    let root = path::from_string("/tmp");
    let subdir = path::join(root, "myapp");
    let file = path::join(subdir, "data.txt");
    
    -- Create directory structure
    act dir::create_dir_all(subdir);
    
    -- Write a file
    act fs::write_string(file, "Hello from Ash!");
    
    -- Check file metadata
    let info = meta::metadata(file);
    
    -- Print file size
    act stdio::println(string::from_int(meta::len(file)));
    
    done;
}
```
