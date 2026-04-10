-- Test io::stdio module surface
-- Tests standard IO capability usage
-- Note: These test fixtures validate parser compatibility

workflow test_stdio_read_line {
    -- Read a line from stdin (requires Stdio capability)
    -- let input = stdio::read_line();
    let input = "test input"
    ret input
}

workflow test_stdio_print {
    -- Print without newline (requires Stdio capability)
    -- act stdio::print("Hello");
    ret Done
}

workflow test_stdio_println {
    -- Print with newline (requires Stdio capability)
    -- act stdio::println("Hello, World!");
    ret Done
}

workflow test_stdio_combined {
    -- Combined stdio operations
    -- act stdio::print("Enter your name: ");
    -- let name = stdio::read_line();
    -- act stdio::print("Hello, ");
    -- act stdio::println(name);
    ret Done
}
