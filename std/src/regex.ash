-- Regex capability and functions
--
-- Provides functions for regular expression operations.
-- All functions require the Regex capability.

-- Regex capability for regular expression operations
pub capability Regex: execute find(pattern: String, text: String) returns Option<String>
                   | execute matches(pattern: String, text: String) returns Bool
                   | execute replace(pattern: String, replacement: String, text: String) returns String;

-- Find the first match of a pattern in text
pub fn find(pattern: String, text: String) -> Option<String> {
    act execute Regex.find with pattern: pattern, text: text
}

-- Check if a pattern matches anywhere in text
pub fn matches(pattern: String, text: String) -> Bool {
    act execute Regex.matches with pattern: pattern, text: text
}

-- Replace all matches of a pattern with a replacement string
pub fn replace(pattern: String, replacement: String, text: String) -> String {
    act execute Regex.replace with pattern: pattern, replacement: replacement, text: text
}
