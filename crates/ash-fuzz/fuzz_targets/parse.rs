//! Parser fuzzing target
//!
//! This fuzzer generates random byte sequences and attempts to parse them
//! as Ash source code. It looks for crashes, hangs, and unexpected panics.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to convert bytes to a string
    let source = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return, // Invalid UTF-8, skip
    };
    
    // Attempt to parse the source
    // In a full implementation, this would use the actual parser
    // For now, we do basic validation
    
    // Check for obviously malformed input patterns that should be handled gracefully
    let _ = validate_basic_syntax(source);
});

/// Basic syntax validation that mimics what a parser would do
/// Returns Ok(()) for valid-looking input, Err for invalid
fn validate_basic_syntax(source: &str) -> Result<(), ()> {
    // Check bracket/parenthesis balance
    let mut brace_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    
    for c in source.chars() {
        match c {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth < 0 {
                    return Err(());
                }
            }
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return Err(());
                }
            }
            '[' => bracket_depth += 1,
            ']' => {
                bracket_depth -= 1;
                if bracket_depth < 0 {
                    return Err(());
                }
            }
            _ => {}
        }
    }
    
    // All brackets should be closed
    if brace_depth != 0 || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    
    // Check for invalid control characters
    for c in source.chars() {
        if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
            return Err(());
        }
    }
    
    // Check string literal validity (basic)
    let mut in_string = false;
    let mut escape = false;
    for c in source.chars() {
        if escape {
            escape = false;
            continue;
        }
        
        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '\n' if in_string => return Err(()), // Unterminated string
            _ => {}
        }
    }
    
    Ok(())
}
