-- Test io::buf module surface
-- Tests buffered IO helpers
-- Note: These test fixtures validate parser compatibility

workflow test_buf_read_to_end {
    -- Read entire file as bytes
    -- let content = buf::read_to_end(path);
    let content = "file contents"
    ret content
}

workflow test_buf_read_to_string {
    -- Read entire file as string
    -- let content = buf::read_to_string(path);
    let content = "Hello, World!"
    ret content
}

workflow test_buf_write_all {
    -- Write all bytes to file
    -- act buf::write_all(path, content);
    ret Done
}

workflow test_buf_lines {
    -- Split text into lines
    -- let lines = buf::lines(text);
    let lines = ["line 1", "line 2", "line 3"]
    ret lines
}
