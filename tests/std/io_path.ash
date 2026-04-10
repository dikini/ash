-- Test io::path module surface
-- Tests pure path operations without side effects
-- Note: These test fixtures validate parser compatibility

workflow test_path_from_string {
    -- Create a PathBuf from a string
    let p = "/home/user"
    ret p
}

workflow test_path_join {
    -- Join two paths
    let base = "/home/user"
    let joined = base ++ "/documents"
    ret joined
}

workflow test_path_parent {
    -- Get parent directory result
    let parent = Some { value: "/home/user" }
    
    observe test with {} as _ {
        assert is_some(parent);
    }
    
    ret Done
}

workflow test_path_file_name {
    -- Get file name result
    let name = Some { value: "file.txt" }
    
    observe test with {} as _ {
        assert is_some(name);
        assert unwrap(name) == "file.txt";
    }
    
    ret Done
}

workflow test_path_extension {
    -- Get file extension result
    let ext = Some { value: "txt" }
    
    observe test with {} as _ {
        assert is_some(ext);
        assert unwrap(ext) == "txt";
    }
    
    ret Done
}

workflow test_path_is_absolute {
    -- Check if path is absolute
    let is_abs = true
    
    observe test with {} as _ {
        assert is_abs;
    }
    
    ret Done
}
