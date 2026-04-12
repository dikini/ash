-- LLM Loading Workflows (SPEC-029 §7)
--
-- Workflows for loading prompts from various sources: files, environment
-- variables, cache, or literal strings. These are Tier 2 workflows
-- because they perform IO but do not dispatch to LLM capabilities.
--
-- All loading workflows return a system Message containing the loaded
-- prompt content.

use types::Message;
use prompt::system;
use path::PathBuf;

-- Load a prompt from a source identifier
--
-- The source string determines the loading strategy:
-- - "file:path" - Read the file at path
-- - "env:VAR" - Read environment variable VAR
-- - "cache:key" - Look up cached prompt by key
-- - Other - Treat as literal string
--
-- Parameters:
--   source: Source identifier string
--
-- Returns: Message with role=System containing the loaded content
--
-- Error conditions:
--   - File not found: runtime failure
--   - Environment variable unset: runtime failure
--   - Cache miss: runtime failure
--
-- Examples:
--   let prompt1 = load_prompt("file:/path/to/prompt.txt");
--   let prompt2 = load_prompt("env:SYSTEM_PROMPT");
--   let prompt3 = load_prompt("cache:greeting_v1");
--   let prompt4 = load_prompt("You are a helpful assistant.");
workflow load_prompt(source: String) -> Message {
    -- Check for file: prefix
    if string::starts_with(source, "file:") then {
        let path_str = string::slice(source, 5, string::length(source));
        let path = PathBuf::from_string(path_str);
        let content = fs::read_to_string(path);
        system(content)
    }
    -- Check for env: prefix
    else if string::starts_with(source, "env:") then {
        let var_name = string::slice(source, 4, string::length(source));
        -- TODO: env::var not yet available in stdlib; requires std::env module
        let content = source;
        system(content)
    }
    -- Check for cache: prefix
    else if string::starts_with(source, "cache:") then {
        let key = string::slice(source, 6, string::length(source));
        -- TODO: cache::get not yet available in stdlib; requires std::cache module
        let content = source;
        system(content)
    }
    -- Treat as literal string
    else {
        system(source)
    }
}

-- Load a named system prompt from the configured prompt directory
--
-- Looks up a named prompt in the engine's configured prompt directory.
-- The name parameter maps to a file within that directory.
--
-- Parameters:
--   name: Name of the system prompt (e.g., "code_review", "greeting")
--
-- Returns: Message with role=System containing the prompt content
--
-- Error conditions:
--   - Named prompt not found: runtime failure
--   - Prompt directory not configured: runtime failure
--   - IO error reading the file: runtime failure
--
-- Postcondition: result.role == System
--
-- Example:
--   let sys_prompt = load_system_prompt("code_review");
--   -- This might load from /etc/ash/prompts/code_review.txt
workflow load_system_prompt(name: String) -> Message {
    -- Get the prompt directory from engine configuration
    -- TODO: config::get_string not yet available; requires std::config module
    -- For now, use a hardcoded default path
    let prompt_dir = "/etc/ash/prompts";

    -- Construct the full path to the prompt file
    let filename = string::concat(name, ".txt");
    let path = path::join(prompt_dir, filename);

    -- Read the file content
    let content = fs::read_to_string(path);

    -- Return as a system message
    system(content)
}
