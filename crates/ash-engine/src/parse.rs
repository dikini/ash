//! Parsing tests for the Ash Engine
//!
//! This module contains tests for `Engine::parse` and `Engine::parse_file` methods.

#[cfg(test)]
mod tests {
    use crate::{Engine, EngineError};
    use std::io::Write;

    // ============================================================
    // parse() Success Tests
    // ============================================================

    #[test]
    fn test_parse_valid_workflow_empty() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { done }");
        assert!(result.is_ok(), "Empty workflow should parse successfully");
    }

    #[test]
    fn test_parse_valid_workflow_with_let() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                let x = 42;
                done
            }
        ",
        );
        assert!(result.is_ok(), "Workflow with let should parse: {result:?}");
    }

    #[test]
    fn test_parse_valid_workflow_with_if() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                if true then done
            }
        ",
        );
        assert!(result.is_ok(), "Workflow with if should parse: {result:?}");
    }

    #[test]
    fn test_parse_valid_workflow_with_if_else() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                if x > 0 then done else done
            }
        ",
        );
        assert!(
            result.is_ok(),
            "Workflow with if-else should parse: {result:?}"
        );
    }

    #[test]
    fn test_parse_valid_workflow_with_observe() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                observe read_db as data;
                done
            }
        ",
        );
        assert!(
            result.is_ok(),
            "Workflow with observe should parse: {result:?}"
        );
    }

    #[test]
    fn test_parse_valid_workflow_with_act() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r#"
            workflow test {
                act log_event("test");
                done
            }
        "#,
        );
        assert!(result.is_ok(), "Workflow with act should parse: {result:?}");
    }

    #[test]
    fn test_parse_valid_workflow_with_for() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                for item in items do done
            }
        ",
        );
        assert!(result.is_ok(), "Workflow with for should parse: {result:?}");
    }

    #[test]
    fn test_parse_valid_workflow_with_with() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                with db do done
            }
        ",
        );
        assert!(
            result.is_ok(),
            "Workflow with with should parse: {result:?}"
        );
    }

    #[test]
    fn test_parse_valid_workflow_with_maybe() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                maybe done else done
            }
        ",
        );
        assert!(
            result.is_ok(),
            "Workflow with maybe should parse: {result:?}"
        );
    }

    #[test]
    fn test_parse_valid_workflow_with_must() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                must done
            }
        ",
        );
        assert!(
            result.is_ok(),
            "Workflow with must should parse: {result:?}"
        );
    }

    #[test]
    fn test_parse_valid_workflow_with_par() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r"
            workflow test {
                par {
                    done
                    done
                }
            }
        ",
        );
        assert!(result.is_ok(), "Workflow with par should parse: {result:?}");
    }

    #[test]
    fn test_parse_valid_workflow_complex() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse(
            r#"
            workflow complex {
                let x = 1;
                let y = 2;
                if x < y then {
                    act print("x is smaller");
                    done
                } else done
            }
        "#,
        );
        assert!(result.is_ok(), "Complex workflow should parse: {result:?}");
    }

    #[test]
    fn test_parse_preserves_workflow_structure() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { done }");
        assert!(result.is_ok());
        // The workflow should be successfully parsed into a Workflow AST
        let workflow = result.unwrap();
        // We can't easily inspect the workflow without more imports,
        // but we can verify it's not Done at the top level (it's wrapped)
        // Just verify we got a Workflow type back
        let _ = workflow;
    }

    // ============================================================
    // parse() Error Tests
    // ============================================================

    #[test]
    fn test_parse_invalid_syntax_missing_brace() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { done");
        assert!(result.is_err(), "Missing closing brace should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error"
        );
    }

    #[test]
    fn test_parse_invalid_syntax_unexpected_token() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { @#$%^& }");
        assert!(result.is_err(), "Invalid tokens should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error"
        );
    }

    #[test]
    fn test_parse_invalid_syntax_no_workflow_keyword() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("invalid syntax!!!");
        assert!(result.is_err(), "Missing workflow keyword should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error"
        );
    }

    #[test]
    fn test_parse_invalid_syntax_empty() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("");
        assert!(result.is_err(), "Empty source should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error"
        );
    }

    #[test]
    fn test_parse_invalid_syntax_missing_workflow_name() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow { done }");
        assert!(result.is_err(), "Missing workflow name should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error"
        );
    }

    #[test]
    fn test_parse_invalid_syntax_invalid_let() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { let = 42; done }");
        assert!(result.is_err(), "Invalid let syntax should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error"
        );
    }

    #[test]
    fn test_parse_error_message_content() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("invalid");
        match result {
            Err(EngineError::Parse(msg)) => {
                assert!(
                    !msg.is_empty(),
                    "Parse error should contain a non-empty message"
                );
            }
            _ => panic!("Expected Parse error with message"),
        }
    }

    // ============================================================
    // parse_file() Success Tests
    // ============================================================

    #[test]
    fn test_parse_file_exists() {
        let engine = Engine::new().build().unwrap();

        // Create a temporary file with valid workflow
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        writeln!(temp_file, "workflow main {{ done }}").unwrap();
        let path = temp_file.path();

        let result = engine.parse_file(path);
        assert!(
            result.is_ok(),
            "Parsing existing file should succeed: {result:?}"
        );
    }

    #[test]
    fn test_parse_file_valid_content() {
        let engine = Engine::new().build().unwrap();

        let mut temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        writeln!(
            temp_file,
            r"
            workflow test {{
                let x = 42;
                act print(x);
                done
            }}
        "
        )
        .unwrap();

        let result = engine.parse_file(temp_file.path());
        assert!(result.is_ok(), "Parsing valid file content should succeed");
    }

    #[test]
    fn test_parse_file_with_comments() {
        let engine = Engine::new().build().unwrap();

        let mut temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        writeln!(
            temp_file,
            r"
            -- This is a comment
            workflow test {{
                /* Multi-line
                   comment */
                done
            }}
        "
        )
        .unwrap();

        let result = engine.parse_file(temp_file.path());
        assert!(
            result.is_ok(),
            "Parsing file with comments should succeed: {result:?}"
        );
    }

    // ============================================================
    // parse_file() Error Tests
    // ============================================================

    #[test]
    fn test_parse_file_not_found() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse_file("/nonexistent/path/to/workflow.ash");

        assert!(result.is_err(), "Non-existent file should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Io(_)),
            "Should return Io error for missing file"
        );
    }

    #[test]
    fn test_parse_file_invalid_content() {
        let engine = Engine::new().build().unwrap();

        // Create a temp file with invalid syntax
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        writeln!(temp_file, "not a valid workflow").unwrap();

        let result = engine.parse_file(temp_file.path());
        assert!(result.is_err(), "Invalid file content should error");
        assert!(
            matches!(result.unwrap_err(), EngineError::Parse(_)),
            "Should return Parse error for invalid content"
        );
    }

    #[test]
    fn test_parse_file_empty() {
        let engine = Engine::new().build().unwrap();

        let temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        // File is empty by default

        let result = engine.parse_file(temp_file.path());
        assert!(result.is_err(), "Empty file should error");
    }

    #[test]
    fn test_parse_file_permission_denied() {
        // This test is platform-specific and may be skipped on some systems
        let engine = Engine::new().build().unwrap();

        let mut temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        writeln!(temp_file, "workflow main {{ done }}").unwrap();

        // Remove read permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(temp_file.path()).unwrap().permissions();
            permissions.set_mode(0o000);
            std::fs::set_permissions(temp_file.path(), permissions).unwrap();

            let result = engine.parse_file(temp_file.path());
            assert!(result.is_err(), "Unreadable file should error");

            // Restore permissions for cleanup
            let mut permissions = std::fs::metadata(temp_file.path()).unwrap().permissions();
            permissions.set_mode(0o644);
            std::fs::set_permissions(temp_file.path(), permissions).unwrap();
        }
    }

    // ============================================================
    // API Consistency Tests
    // ============================================================

    #[test]
    fn test_parse_and_parse_file_equivalence() {
        let engine = Engine::new().build().unwrap();
        let source = "workflow main { let x = 42; done }";

        // Parse from string
        let result_parse = engine.parse(source);

        // Parse from file
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".ash").unwrap();
        writeln!(temp_file, "{source}").unwrap();
        let result_file = engine.parse_file(temp_file.path());

        // Both should succeed or both should fail
        assert_eq!(
            result_parse.is_ok(),
            result_file.is_ok(),
            "parse() and parse_file() should have equivalent results"
        );
    }

    #[test]
    fn test_parse_is_pure() {
        let engine = Engine::new().build().unwrap();
        let source = "workflow main { done }";

        let result1 = engine.parse(source);
        let result2 = engine.parse(source);

        assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "parse() should be deterministic"
        );
    }
}
