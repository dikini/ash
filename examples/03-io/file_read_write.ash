-- File Read/Write Example
--
-- Demonstrates io::fs and io::stdio modules for file operations
-- and console output. Requires Fs and Stdio capabilities.

workflow main {
    -- Note: In real usage, add:
    --   use io::fs;
    --   use io::stdio;
    --   use io::path;
    
    -- Read file content (requires Fs capability)
    -- let content = fs::read_to_string("input.txt");
    
    -- Print to stdout (requires Stdio capability)
    -- act stdio::println(content);
    
    -- Write to file (requires Fs capability)
    -- let out_path = path::from_string("output.txt");
    -- act fs::write_string(out_path, "Hello, World!");
    
    -- Copy file (requires Fs capability)
    -- act fs::copy(from_path, to_path);
    
    ret {
        status: "demonstration",
        note: "Uncomment the IO operations when capabilities are available"
    }
}
