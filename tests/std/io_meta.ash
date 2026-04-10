-- Test io::meta module surface
-- Tests metadata operations (requires Meta capability)
-- Note: These test fixtures validate parser compatibility

workflow test_meta_metadata {
    -- Get metadata for a path (requires Meta capability)
    -- let info = meta::metadata("/tmp");
    let info = {
        is_file: false,
        is_dir: true,
        len: 0,
        readonly: false
    }
    ret info
}

workflow test_meta_is_file {
    -- Check if path is a file (requires Meta capability)
    let is_file = false
    
    observe test with {} as _ {
        assert !is_file;
    }
    
    ret Done
}

workflow test_meta_is_dir {
    -- Check if path is a directory (requires Meta capability)
    let is_dir = true
    
    observe test with {} as _ {
        assert is_dir;
    }
    
    ret Done
}

workflow test_meta_len {
    -- Get file size (requires Meta capability)
    let size = 1024
    
    observe test with {} as _ {
        assert size >= 0;
    }
    
    ret Done
}

workflow test_meta_readonly {
    -- Check if file is read-only (requires Meta capability)
    let ro = false
    ret ro
}
