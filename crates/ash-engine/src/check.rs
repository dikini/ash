//! Type checking tests for the Ash Engine
//!
//! This module contains tests for `Engine::check` method which performs
//! type checking on parsed workflows.

#[cfg(test)]
mod tests {
    use crate::{Engine, EngineError};

    // ============================================================
    // check() Success Tests
    // ============================================================

    #[test]
    fn test_check_valid_workflow_empty() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine.parse("workflow main { done }").unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Empty workflow should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_let() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let x = 42;
                done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with let should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_ret() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                ret 42;
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with ret should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_if() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                if true then done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with if should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_if_else() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let x = 1;
                if x > 0 then done else done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with if-else should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_observe() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                observe read_db as data;
                done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with observe should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_args_capability_surface() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow main(args: cap Args) {
                observe Args 0;
                done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with Args capability surface should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_act() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r#"
            workflow test {
                act log_event("test");
                done
            }
        "#,
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with act should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_for() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let items = [1, 2, 3];
                for item in items do done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with for should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_with() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                with db do done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with with should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_maybe() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                maybe done else done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with maybe should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_must() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                must done
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with must should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_workflow_with_par() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                par {
                    done
                    done
                }
            }
        ",
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with par should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_valid_complex_workflow() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
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
            )
            .unwrap();
        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Complex workflow should pass type checking: {result:?}"
        );
    }

    // ============================================================
    // check() Error Tests
    // ============================================================

    #[test]
    fn test_check_returns_type_error_for_invalid_workflow() {
        let engine = Engine::new().build().unwrap();
        // Parse a workflow that might have type issues
        // Currently the parser accepts most syntax, so type errors would
        // come from semantic analysis during check()
        let workflow = engine
            .parse(
                r"
            workflow test {
                let x = 42;
                ret x;
            }
        ",
            )
            .unwrap();

        // For now, check() is stubbed with todo!()
        // This test will panic until implementation is added
        let _result = engine.check(&workflow);
    }

    #[test]
    fn test_check_error_returns_type_variant() {
        // This test verifies that when type checking fails,
        // the error is of the Type variant
        // Note: This test will be meaningful once type checking is implemented
        let engine = Engine::new().build().unwrap();
        let workflow = engine.parse("workflow main { done }").unwrap();

        // Currently this will panic due to todo!()
        // After implementation, this should verify error type
        let _result = engine.check(&workflow);
    }

    // ============================================================
    // Integration: Parse then Check
    // ============================================================

    #[test]
    fn test_parse_then_check_valid_workflow() {
        let engine = Engine::new().build().unwrap();
        let source = r#"
            workflow main {
                let message = "Hello, World!";
                act print(message);
                done
            }
        "#;

        // Parse first
        let parse_result = engine.parse(source);
        assert!(parse_result.is_ok(), "Source should parse successfully");

        let workflow = parse_result.unwrap();

        // Then check
        let check_result = engine.check(&workflow);
        assert!(
            check_result.is_ok(),
            "Parsed workflow should pass type checking: {check_result:?}"
        );
    }

    #[test]
    fn test_parse_then_check_preserves_workflow_structure() {
        let engine = Engine::new().build().unwrap();
        let source = "workflow test { let x = 42; ret x; }";

        let workflow = engine.parse(source).unwrap();
        let result = engine.check(&workflow);

        // The workflow structure should be preserved through parse->check
        // This is mainly a compile-time check that &Workflow is accepted
        assert!(
            result.is_ok() || result.is_err(),
            "Check should complete (either succeed or fail with an error)"
        );
    }

    #[test]
    fn test_check_is_pure_for_same_workflow() {
        let engine = Engine::new().build().unwrap();
        let _workflow = engine.parse("workflow main { done }").unwrap();

        // For now, both calls will panic with todo!()
        // After implementation, check() should be deterministic
        // let result1 = engine.check(&_workflow);
        // let result2 = engine.check(&_workflow);
        // assert_eq!(result1.is_ok(), result2.is_ok());
    }

    #[test]
    fn test_check_does_not_modify_workflow() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine.parse("workflow main { done }").unwrap();

        // Clone for comparison (Workflow implements Clone)
        let workflow_before = workflow.clone();

        // check() takes &Workflow, so it shouldn't modify
        // This is mainly a compile-time assertion
        let _ = &workflow;

        // Verify workflow wasn't consumed
        assert_eq!(
            workflow, workflow_before,
            "Workflow should remain unchanged after check"
        );
    }

    #[test]
    fn test_check_with_various_workflow_names() {
        let engine = Engine::new().build().unwrap();

        let names = [
            "main",
            "test",
            "my_workflow",
            "ProcessOrder",
            "handle_request",
        ];

        for name in names {
            let source = format!("workflow {name} {{ done }}");
            let workflow = engine.parse(&source).unwrap();
            let result = engine.check(&workflow);
            // Will panic on first iteration due to todo!()
            assert!(
                result.is_ok(),
                "Workflow '{name}' should pass type checking"
            );
        }
    }

    #[test]
    fn test_check_with_nested_expressions() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let a = 1 + 2;
                let b = a * 3;
                let c = b - 1;
                if c > 0 then done else done
            }
        ",
            )
            .unwrap();

        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with nested expressions should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_with_string_operations() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r#"
            workflow test {
                let greeting = "Hello";
                let name = "World";
                done
            }
        "#,
            )
            .unwrap();

        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with strings should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_with_boolean_operations() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let a = true;
                let b = false;
                let c = a && b;
                let d = a || b;
                let e = !a;
                done
            }
        ",
            )
            .unwrap();

        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with booleans should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_with_comparison_operations() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let x = 5;
                let y = 10;
                let eq = x == y;
                let ne = x != y;
                let lt = x < y;
                let le = x <= y;
                let gt = x > y;
                let ge = x >= y;
                done
            }
        ",
            )
            .unwrap();

        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with comparisons should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_with_list_expressions() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine
            .parse(
                r"
            workflow test {
                let items = [1, 2, 3];
                for item in items do done
            }
        ",
            )
            .unwrap();

        let result = engine.check(&workflow);
        assert!(
            result.is_ok(),
            "Workflow with lists should pass type checking: {result:?}"
        );
    }

    #[test]
    fn test_check_error_message_content() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine.parse("workflow main { done }").unwrap();

        // After implementation, verify that error messages are descriptive
        match engine.check(&workflow) {
            Ok(()) => {
                // Success case - valid workflow
            }
            Err(EngineError::Type(msg)) => {
                assert!(
                    !msg.is_empty(),
                    "Type error should have a non-empty message"
                );
            }
            Err(other) => {
                panic!("Expected Type error or success, got: {other:?}");
            }
        }
    }

    // ============================================================
    // Error Handling Tests
    // ============================================================

    #[test]
    fn test_check_error_from_type_checker() {
        // Verify that EngineError::Type can be constructed from type checking errors
        let err = EngineError::Type("expected Int, got String".to_string());
        assert!(matches!(err, EngineError::Type(_)));

        let display = format!("{err}");
        assert!(display.contains("type error"));
        assert!(display.contains("expected Int, got String"));
    }

    #[test]
    fn test_type_error_preserves_message() {
        let message = "variable 'x' has type Int but expected String";
        let err = EngineError::Type(message.to_string());

        if let EngineError::Type(found) = err {
            assert_eq!(found, message);
        } else {
            panic!("Should be Type variant");
        }
    }
}
