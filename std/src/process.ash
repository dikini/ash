-- Process execution capability and functions
--
-- Provides parser-checkable runtime-provided function declarations for external
-- process access. The Process capability below records the intended authority
-- contract; concrete capability-wrapper bodies remain deferred until the
-- parser/runtime support a canonical stdlib `act` wrapper spelling.
-- Process execution is effectful (Operational) per the three-pillar principle.

-- Process.run returns a runtime record with stdout, stderr, and exit_code.
-- Process.which returns Some(path) when found and None when absent.
pub capability Process: execute run(cmd: String, args: List<String>) returns Record
                     | execute which(cmd: String) returns Option<String>;

-- Execute a command with arguments, returning the provider output record.
pub builtin fn run(cmd: String, args: List<String>) -> Record;

-- Check if a command exists, returning Some(path) or None.
pub builtin fn which(cmd: String) -> Option<String>;
