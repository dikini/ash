-- Directory Operations Example
--
-- Demonstrates io::dir and io::meta modules for directory
-- operations and metadata queries. Requires Dir and Meta capabilities.

workflow main {
    -- Note: In real usage, add:
    --   use io::dir;
    --   use io::meta;
    --   use io::path;
    
    -- Create a directory (requires Dir capability)
    -- act dir::create_dir("/tmp/test_ash");
    
    -- Create nested directories (requires Dir capability)
    -- act dir::create_dir_all("/tmp/test_ash/nested/deep");
    
    -- Query metadata (requires Meta capability)
    -- let info = meta::metadata("/tmp");
    
    -- Check if directory (requires Meta capability)
    -- let is_dir = meta::is_dir("/tmp");
    
    -- List directory contents (requires Dir capability)
    -- let entries = dir::read_dir("/tmp");
    
    -- Clean up (requires Dir capability)
    -- act dir::remove_dir_all("/tmp/test_ash");
    
    ret {
        status: "demonstration",
        note: "Uncomment the IO operations when capabilities are available"
    }
}
