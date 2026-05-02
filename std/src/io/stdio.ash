-- Standard I/O capability and functions
--
-- Provides parser-checkable runtime-provided function declarations for stdin
-- and stdout access. The Stdio capability below records the intended authority
-- contract; concrete capability-wrapper bodies remain deferred until the
-- parser/runtime support a canonical stdlib `act` wrapper spelling.

-- Stdio capability for standard input/output operations
pub capability Stdio: observe read_line() returns String
                    | execute print(text: String)
                    | execute println(text: String);

-- Read a line from stdin
pub builtin fn read_line() -> String;

-- Print text without a newline
pub builtin fn print(text: String) -> Unit;

-- Print text with a newline
pub builtin fn println(text: String) -> Unit;
