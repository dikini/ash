-- Standard I/O capability and functions
--
-- Provides functions for reading from stdin and writing to stdout.
-- All functions require the Stdio capability.

-- Stdio capability for standard input/output operations
pub capability Stdio: observe read_line() returns String
                    | execute print(text: String)
                    | execute println(text: String);

-- Read a line from stdin
pub fn read_line() -> String {
    act observe Stdio.read_line
}

-- Print text without a newline
pub fn print(text: String) {
    act execute Stdio.print with text: text;
}

-- Print text with a newline
pub fn println(text: String) {
    act execute Stdio.println with text: text;
}
