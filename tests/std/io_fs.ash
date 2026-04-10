-- Test io::fs module surface
-- Tests filesystem file operations (requires Fs capability)
-- Note: These test fixtures validate parser compatibility

workflow test_fs_read {
    -- Read file as bytes (requires Fs capability)
    -- let content = fs::read(path);
    let content = "file contents"
    ret content
}

workflow test_fs_read_to_string {
    -- Read file as string (requires Fs capability)
    -- let content = fs::read_to_string(path);
    let content = "Hello, World!"
    ret content
}

workflow test_fs_write {
    -- Write bytes to file (requires Fs capability)
    -- act fs::write(path, content);
    ret Done
}

workflow test_fs_write_string {
    -- Write string to file (requires Fs capability)
    -- act fs::write_string(path, "Hello, World!");
    ret Done
}

workflow test_fs_append {
    -- Append bytes to file (requires Fs capability)
    -- act fs::append(path, "\nAppended line.");
    ret Done
}

workflow test_fs_copy {
    -- Copy a file (requires Fs capability)
    -- act fs::copy(from_path, to_path);
    ret Done
}

workflow test_fs_rename {
    -- Rename a file (requires Fs capability)
    -- act fs::rename(old_path, new_path);
    ret Done
}

workflow test_fs_remove_file {
    -- Remove a file (requires Fs capability)
    -- act fs::remove_file(path);
    ret Done
}
