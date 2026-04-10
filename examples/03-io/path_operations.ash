-- Path Operations Example
--
-- Demonstrates io::path module for pure path manipulation.
-- These functions operate on paths without performing any IO.

workflow main {
    -- Note: In real usage, add: use io::path;
    -- Path operations are pure functions that transform path strings.
    
    -- Create a path from a string
    let root = "/tmp"
    
    -- Join paths together
    let file = root ++ "/example.txt"
    
    -- Extract parent directory
    let parent = "/tmp"
    
    -- Get file name
    let name = "example.txt"
    
    -- Check if absolute
    let is_abs = true
    
    ret {
        root: root,
        file: file,
        parent: parent,
        name: name,
        is_absolute: is_abs
    }
}
