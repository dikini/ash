-- Test io::dir module surface
-- Tests directory operations (requires Dir capability)
-- Note: These test fixtures validate parser compatibility

workflow test_dir_create_dir {
    -- Create a directory (requires Dir capability)
    -- act dir::create_dir("/tmp/test_dir");
    ret Done
}

workflow test_dir_create_dir_all {
    -- Create directory and all parents (requires Dir capability)
    -- act dir::create_dir_all("/tmp/nested/deep/dir");
    ret Done
}

workflow test_dir_remove_dir {
    -- Remove an empty directory (requires Dir capability)
    -- act dir::remove_dir("/tmp/empty_dir");
    ret Done
}

workflow test_dir_remove_dir_all {
    -- Remove directory and all contents (requires Dir capability)
    -- act dir::remove_dir_all("/tmp/remove_me");
    ret Done
}

workflow test_dir_read_dir {
    -- Read directory contents (requires Dir capability)
    -- let entries = dir::read_dir("/tmp");
    let entries = ["file1.txt", "file2.txt"]
    ret entries
}
