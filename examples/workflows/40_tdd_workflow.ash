-- TDD Workflow Example
-- Demonstrates: complex multi-role workflow, obligation tracking, iterative development cycles
-- Reference-oriented workflow template; canonical role and approval contracts live in `docs/spec/`.
-- 
-- This workflow orchestrates the complete Test-Driven Development lifecycle
-- involving three roles: developer, tester, and reviewer.

-- ============================================================================
-- Capability Definitions
-- ============================================================================

-- Task management capability
capability task_manager : observe(task_id: String) returns Task
    requires role(tester, developer, reviewer)

capability task_updater : act(task_id: String, status: TaskStatus) 
    requires role(developer)

-- Test management capabilities  
capability test_runner : act(test_suite: TestSuite) returns TestResults
    requires role(tester, developer)
    effect: evaluative

capability test_registry : observe(suite_id: String) returns TestSuite
    requires role(tester)

capability test_definer : act(function_name: String, test_cases: List<TestCase>)
    requires role(tester)
    effect: evaluative

-- Code repository capabilities
capability code_reader : observe(module_path: String) returns SourceCode
    requires role(developer, reviewer)

capability code_writer : act(module_path: String, code: SourceCode)
    requires role(developer)
    where not_in_review

capability type_checker : act(code: SourceCode) returns TypeCheckResult
    requires role(developer, tester)
    effect: evaluative

-- Review capabilities
capability review_assigner : act(task_id: String, reviewer: Role) 
    requires role(tester)

capability review_submitter : act(task_id: String, feedback: ReviewFeedback)
    requires role(reviewer)

capability review_reader : observe(task_id: String) returns ReviewFeedback
    requires role(developer, tester)

-- ============================================================================
-- Role Definitions
-- ============================================================================

role developer {
    authority: [
        task_manager, task_updater, test_runner, code_reader, 
        code_writer, type_checker, review_reader
    ],
    obligations: [
        must_write_tests_first,
        must_not_commit_without_tests,
        must_refactor_before_submit
    ]
}

role tester {
    authority: [
        task_manager, test_runner, test_registry, test_definer,
        type_checker, review_reader, review_assigner
    ],
    obligations: [
        must_define_property_tests,
        must_ensure_red_phase_first,
        must_verify_coverage
    ]
}

role reviewer {
    authority: [
        task_manager, code_reader, review_submitter, review_reader,
        test_runner
    ],
    obligations: [
        must_not_review_own_code,
        must_check_test_quality,
        must_verify_no_duplication
    ]
}

-- ============================================================================
-- Policy Definitions
-- ============================================================================

policy red_phase_required:
    when function_implemented and tests_not_failing
    then deny
    else permit

policy tests_must_pass_before_review:
    when submit_for_review and has_failing_tests
    then deny
    else require_approval(role: tester)

policy reviewer_separation:
    when review_assigned and reviewer_is_developer
    then deny
    else permit

policy coverage_minimum:
    when coverage < 80%
    then require_approval(role: tester)
    else permit

policy all_tests_green_required:
    when finalize and has_failing_tests
    then deny
    else permit

-- ============================================================================
-- Main TDD Workflow
-- ============================================================================

workflow tdd_development {
    -- Input: task_id provided when workflow is invoked
    let task_id = $input.task_id;
    
    -- ========================================================================
    -- PHASE 1: ANALYSIS & DESIGN (Orient)
    -- ========================================================================
    
    observe task_manager with task_id: task_id as task;
    
    orient {
        let requirements = parse_requirements(task.description);
        let functions = identify_functions(requirements);
        let types = infer_types(functions);
        
        -- Design document for the solution
        {
            task_id: task_id,
            requirements: requirements,
            functions: functions,
            types: types,
            status: "designed"
        }
    } as design;
    
    act task_updater with 
        task_id: task_id, 
        status: { phase: "testing", design: design };
    
    -- ========================================================================
    -- PHASE 2: TEST DEFINITION (Testing - Test Design)
    -- ========================================================================
    
    oblige tester must_define_property_tests {
        task: task_id,
        for_each: design.functions
    };
    
    -- Define tests for each function
    for func in design.functions do {
        
        -- Define property tests (proptest style)
        orient define_property_tests(func, design.types) as property_tests;
        
        -- Define unit/regression tests
        orient define_unit_tests(func, design.types) as unit_tests;
        
        -- Define integration tests
        orient define_integration_tests(func, design.types) as integration_tests;
        
        -- Register all tests
        act test_definer with 
            function_name: func.name,
            test_cases: concat(property_tests, unit_tests, integration_tests);
        
        -- Store test suite for this function
        let test_suites = test_suites + [{
            function: func.name,
            properties: property_tests,
            units: unit_tests,
            integration: integration_tests,
            all_tests: concat(property_tests, unit_tests, integration_tests)
        }]
    };
    
    -- ========================================================================
    -- PHASE 3: RED PHASE - Run tests to confirm they fail
    -- ========================================================================
    
    act task_updater with 
        task_id: task_id, 
        status: { phase: "red", message: "Running tests to confirm failures" };
    
    -- Run all tests - they should fail since no implementation exists
    observe test_registry with suite_id: task_id as test_suite;
    
    act test_runner with test_suite: test_suite as red_results;
    
    decide { red_results.all_failed } under red_phase_required then {
        -- Red phase successful - all tests fail as expected
        act task_updater with 
            task_id: task_id, 
            status: { 
                phase: "red_complete", 
                failing_count: red_results.fail_count 
            };
    } else {
        -- Some tests passed unexpectedly - investigate
        act alert_tester with 
            message: "Some tests passed without implementation!",
            details: red_results.passed_tests;
        ret { error: "invalid_test_setup", results: red_results }
    };
    
    -- ========================================================================
    -- PHASE 4: GREEN PHASE - Implement to pass tests
    -- ========================================================================
    
    act task_updater with 
        task_id: task_id, 
        status: { phase: "green", message: "Implementing functions" };
    
    -- Implement each function
    for func in design.functions do {
        
        let func_tests = find_tests(test_suites, func.name);
        
        -- Iterative implementation for this function
        loop {
            
            -- Implement minimal code to pass current failing tests
            orient {
                let current_code = if exists_code(func.module_path) then
                    observe code_reader with module_path: func.module_path
                else
                    generate_stub(func, design.types);
                
                implement_minimal(current_code, func, func_tests.failing)
            } as implementation;
            
            -- Write implementation
            act code_writer with 
                module_path: func.module_path, 
                code: implementation;
            
            -- Run tests for this function
            act test_runner with 
                test_suite: func_tests 
                as test_results;
            
            decide { test_results.all_pass } then {
                -- Green phase complete for this function
                break
            } else {
                -- Still failing - continue implementation
                let func_tests = update_failing_tests(func_tests, test_results);
                continue
            }
        };
        
        -- ========================================================================
        -- PHASE 5: REFACTOR - Clean up while keeping tests green
        -- ========================================================================
        
        orient {
            let current_code = observe code_reader with module_path: func.module_path;
            let refactored = refactor(current_code);
            
            -- Ensure no duplication, better naming, cleaner structure
            verify_no_code_duplication(refactored);
            verify_clean_architecture(refactored);
            
            refactored
        } as clean_implementation;
        
        act code_writer with 
            module_path: func.module_path, 
            code: clean_implementation;
        
        -- Verify tests still pass after refactoring
        act test_runner with test_suite: func_tests as refactor_results;
        
        check must_not_break_tests(refactor_results) then {
            act task_updater with 
                task_id: task_id,
                function: func.name, 
                status: { 
                    phase: "refactored", 
                    tests_passing: refactor_results.pass_count 
                };
        };
        
        oblige developer must_refactor_before_submit {
            function: func.name,
            code_quality_score: calculate_quality(clean_implementation)
        }
    };
    
    -- ========================================================================
    -- PHASE 6: FULL TEST SUITE VERIFICATION
    -- ========================================================================
    
    act task_updater with 
        task_id: task_id, 
        status: { phase: "verification", message: "Running full test suite" };
    
    -- Run complete test suite
    act test_runner with 
        test_suite: { all: true, suite_id: task_id } 
        as final_test_results;
    
    decide { 
        final_test_results.all_pass and 
        final_test_results.coverage >= 80 
    } under all_tests_green_required then {
        
        act task_updater with 
            task_id: task_id, 
            status: { 
                phase: "ready_for_review",
                test_count: final_test_results.total,
                coverage: final_test_results.coverage
            };
    } else {
        ret { 
            error: "tests_not_passing", 
            results: final_test_results 
        }
    };
    
    -- ========================================================================
    -- PHASE 7: CODE REVIEW
    -- ========================================================================
    
    -- Assign reviewer (must not be developer)
    orient select_reviewer(available_reviewers, task.developer) as reviewer;
    
    decide { reviewer != task.developer } under reviewer_separation then {
        act review_assigner with 
            task_id: task_id, 
            reviewer: reviewer;
    } else {
        ret { error: "reviewer_conflict", message: "Reviewer cannot be developer" }
    };
    
    -- Wait for review
    loop {
        observe review_reader with task_id: task_id as review_status;
        
        decide { review_status.completed } then {
            break
        } else {
            sleep(60s);
            continue
        }
    };
    
    -- Process review feedback
    observe review_reader with task_id: task_id as feedback;
    
    decide { feedback.approved } then {
        -- Review passed
        act task_updater with 
            task_id: task_id, 
            status: { phase: "reviewed", approved: true };
    } else {
        -- Review found issues - address them
        act task_updater with 
            task_id: task_id, 
            status: { 
                phase: "review_fixes", 
                issues: feedback.issues 
            };
        
        -- Address each issue
        for issue in feedback.issues do {
            
            orient analyze_issue(issue) as fix_strategy;
            
            -- Implement fix
            observe code_reader with module_path: issue.file as current_code;
            orient apply_fix(current_code, fix_strategy) as fixed_code;
            
            act code_writer with 
                module_path: issue.file, 
                code: fixed_code;
            
            -- Verify fix with tests
            act test_runner with 
                test_suite: { related_to: issue.file } 
                as fix_results;
            
            check must_not_break_tests(fix_results) then {
                act log_fix with 
                    task_id: task_id, 
                    issue: issue.id, 
                    status: "fixed";
            }
        };
        
        -- Re-run full test suite after fixes
        act test_runner with 
            test_suite: { all: true } 
            as post_fix_results;
        
        decide { post_fix_results.all_pass } then {
            -- Re-review required
            act review_assigner with 
                task_id: task_id, 
                reviewer: reviewer;
            
            -- Loop back to wait for new review
            continue
        }
    };
    
    -- ========================================================================
    -- PHASE 8: FINALIZATION
    -- ========================================================================
    
    act task_updater with 
        task_id: task_id, 
        status: { 
            phase: "finalizing",
            message: "Preparing for deployment" 
        };
    
    -- Final quality checks
    orient {
        let final_code = observe code_reader with module_path: task.module_path;
        {
            code_quality: calculate_quality(final_code),
            test_coverage: final_test_results.coverage,
            documentation: check_documentation(final_code),
            no_todos: check_no_todo_comments(final_code)
        }
    } as quality_report;
    
    decide { 
        quality_report.code_quality >= 90 and
        quality_report.test_coverage >= 80 and
        quality_report.documentation and
        quality_report.no_todos
    } then {
        
        act task_updater with 
            task_id: task_id, 
            status: { 
                phase: "completed",
                quality: quality_report,
                ready_for_deployment: true
            };
        
        ret {
            task_id: task_id,
            status: "completed",
            tests: {
                total: final_test_results.total,
                passing: final_test_results.pass_count,
                coverage: final_test_results.coverage
            },
            quality: quality_report,
            review: feedback
        }
    } else {
        ret {
            error: "quality_check_failed",
            quality_report: quality_report,
            message: "Task does not meet quality standards for deployment"
        }
    }
}

-- ============================================================================
-- Helper Functions (would be defined as capabilities in real implementation)
-- ============================================================================

-- Parse requirements from task description
function parse_requirements(description: String) -> Requirements {
    -- Extract functional and non-functional requirements
    -- Identify inputs, outputs, constraints
}

-- Identify functions needed from requirements
function identify_functions(requirements: Requirements) -> List<FunctionSpec> {
    -- Break down requirements into function specifications
}

-- Infer types for function signatures
function infer_types(functions: List<FunctionSpec>) -> TypeMap {
    -- Determine input and output types for each function
}

-- Define property-based tests for a function
function define_property_tests(func: FunctionSpec, types: TypeMap) -> List<TestCase> {
    -- Generate proptest-style property tests
    -- Examples: associativity, commutativity, inverse, idempotence
}

-- Define unit and regression tests
function define_unit_tests(func: FunctionSpec, types: TypeMap) -> List<TestCase> {
    -- Edge cases, boundary conditions, known bug regressions
}

-- Define integration tests
function define_integration_tests(func: FunctionSpec, types: TypeMap) -> List<TestCase> {
    -- Tests involving multiple functions/systems
}

-- Generate minimal implementation stub
function generate_stub(func: FunctionSpec, types: TypeMap) -> SourceCode {
    -- Create function signature with todo!() body
}

-- Implement minimal code to pass tests
function implement_minimal(
    current: SourceCode, 
    func: FunctionSpec, 
    failing_tests: List<TestCase>
) -> SourceCode {
    -- Add minimal logic to make failing tests pass
}

-- Refactor code for cleanliness
function refactor(code: SourceCode) -> SourceCode {
    -- Extract functions, rename variables, simplify expressions
}

-- Calculate code quality score
function calculate_quality(code: SourceCode) -> Int {
    -- Based on complexity, duplication, documentation, naming
}

-- Check for code duplication
function verify_no_code_duplication(code: SourceCode) -> Bool {
    -- AST-based duplication detection
}

-- Check clean architecture compliance
function verify_clean_architecture(code: SourceCode) -> Bool {
    -- Single responsibility, dependency direction, etc.
}
