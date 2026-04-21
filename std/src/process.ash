-- Process execution capability and functions
--
-- Provides functions for executing external processes.
-- All functions require the Process capability.
-- Process execution is effectful (Operational) per the three-pillar principle.

-- Process capability for command execution
pub capability Process: execute run(cmd: String, args: List<String>) returns String
                     | execute which(cmd: String) returns String;

-- Execute a command with arguments, returns a record with stdout, stderr, exit_code
pub fn run(cmd: String, args: List<String>) -> String {
    act execute Process.run with cmd: cmd, args: args;
}

-- Check if a command exists, returns its path or null
pub fn which(cmd: String) -> String {
    act execute Process.which with cmd: cmd;
}
