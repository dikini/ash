-- Concrete TDD Example: Implementing a Stack
-- 
-- This workflow demonstrates TDD with a concrete example - implementing
-- a stack data structure with push, pop, and peek operations.
--
-- Usage: ash run 40a_tdd_concrete_example.ash -- '{"task_id": "STACK-001"}'

-- ============================================================================
-- Required Capabilities
-- ============================================================================

capability task_repo : observe(task_id: String) returns Task
capability test_db : observe(suite_id: String) returns TestSuite
capability test_writer : act(tests: List<TestCase>) 
    requires role(tester)
capability code_repo : observe(path: String) returns SourceCode
capability code_commit : act(path: String, code: SourceCode)
    requires role(developer)
capability test_executor : act(tests: TestSuite) returns TestResults
capability coverage_analyzer : act(code: SourceCode, tests: TestSuite) returns CoverageReport

-- ============================================================================
-- Roles
-- ============================================================================

role developer {
    authority: [code_repo, code_commit, test_executor, coverage_analyzer],
    obligations: [write_tests_first, green_before_refactor]
}

role tester {
    authority: [task_repo, test_db, test_writer, test_executor],
    obligations: [define_property_tests, ensure_red_phase]
}

-- ============================================================================
-- Policies
-- ============================================================================

policy test_first:
    when commit_code and tests_not_defined
    then deny
    else permit

policy coverage_80:
    when coverage < 80
    then require_approval(role: tester)
    else permit

-- ============================================================================
-- Types (for documentation - Ash type inference handles these)
-- ============================================================================

-- Stack<T> = { elements: List<T>, size: Int }
-- Operation results: PushResult, PopResult<T>, PeekResult<T>

-- ============================================================================
-- Concrete TDD Workflow for Stack Implementation
-- ============================================================================

workflow tdd_stack_implementation {
    let task_id = "STACK-001";
    
    observe task_repo with task_id: task_id as task;
    
    ret "TDD Stack Implementation workflow loaded. Task: " + task.name
    
    -- In a real implementation, the workflow would continue through
    -- all TDD phases. This abbreviated version shows the structure.
}

-- ============================================================================
-- Phase 2: Test Design (Tester's Responsibility)
-- ============================================================================

workflow design_stack_tests {
    
    -- Property tests for stack operations
    let stack_properties = [
        -- push then pop returns original stack (inverse property)
        {
            name: "push_pop_inverse",
            description: "push(x); pop() should return x and restore original stack",
            forall: { stack: Stack<Int>, value: Int },
            property: "let original = stack.clone(); stack.push(value); let (popped, _) = stack.pop(); popped == value and stack == original"
        },
        
        -- peek doesn't modify stack
        {
            name: "peek_is_pure",
            description: "peek() should not modify stack size",
            forall: { stack: Stack<Int> where not_empty(stack) },
            property: "let before = stack.size(); let _ = stack.peek(); stack.size() == before"
        },
        
        -- size increases by 1 on push
        {
            name: "push_increments_size",
            description: "push increases size by exactly 1",
            forall: { stack: Stack<Int>, value: Int },
            property: "let before = stack.size(); stack.push(value); stack.size() == before + 1"
        },
        
        -- size decreases by 1 on pop
        {
            name: "pop_decrements_size",
            description: "pop decreases size by exactly 1",
            forall: { stack: Stack<Int> where not_empty(stack) },
            property: "let before = stack.size(); stack.pop(); stack.size() == before - 1"
        },
        
        -- LIFO property
        {
            name: "lifo_order",
            description: "Last pushed element is first popped",
            forall: { stack: Stack<Int>, a: Int, b: Int },
            property: "stack.push(a); stack.push(b); let (first, _) = stack.pop(); first == b"
        }
    ];
    
    -- Unit tests for edge cases
    let stack_unit_tests = [
        -- Empty stack operations
        {
            name: "pop_empty_fails",
            input: { stack: Stack::new() },
            expected: Error("EmptyStack"),
            test: "stack.pop()"
        },
        {
            name: "peek_empty_fails", 
            input: { stack: Stack::new() },
            expected: Error("EmptyStack"),
            test: "stack.peek()"
        },
        {
            name: "new_stack_is_empty",
            input: {},
            expected: 0,
            test: "Stack::new().size()"
        },
        
        -- Single element operations
        {
            name: "push_one_peek_correct",
            input: { value: 42 },
            setup: "let s = Stack::new(); s.push(value);",
            expected: 42,
            test: "s.peek()"
        },
        {
            name: "push_one_size_one",
            input: { value: 42 },
            setup: "let s = Stack::new(); s.push(value);",
            expected: 1,
            test: "s.size()"
        },
        
        -- Multiple element operations
        {
            name: "push_multiple_order",
            input: { values: [1, 2, 3] },
            setup: "let s = Stack::new(); for v in values { s.push(v); }",
            expected: [3, 2, 1],
            test: "[s.pop().0, s.pop().0, s.pop().0]"
        }
    ];
    
    -- Integration tests
    let stack_integration_tests = [
        {
            name: "stack_as_collection_adapter",
            description: "Stack works with generic collection interface",
            setup: "let stack = Stack::new(); let adapter = CollectionAdapter::new(stack);",
            operations: ["adapter.add(1)", "adapter.add(2)", "adapter.remove()"],
            expected: [Ok(()), Ok(()), Ok(2)]
        },
        {
            name: "stack_with_iterator",
            description: "Stack can be converted to iterator",
            setup: "let stack = Stack::new(); stack.push(1); stack.push(2); stack.push(3);",
            test: "stack.iter().collect::<List<Int>>()",
            expected: [3, 2, 1]
        }
    ];
    
    -- Register all tests
    act test_writer with tests: concat(
        map(stack_properties, |p| TestCase::Property(p)),
        map(stack_unit_tests, |u| TestCase::Unit(u)),
        map(stack_integration_tests, |i| TestCase::Integration(i))
    );
    
    ret {
        phase: "test_design_complete",
        property_tests: len(stack_properties),
        unit_tests: len(stack_unit_tests),
        integration_tests: len(stack_integration_tests),
        total_tests: len(stack_properties) + len(stack_unit_tests) + len(stack_integration_tests)
    }
}

-- ============================================================================
-- Phase 4: Implementation (Red → Green → Refactor cycles)
-- ============================================================================

workflow implement_stack {
    let module_path = "src/collections/stack.rs";
    
    -- RED PHASE: Verify tests fail without implementation
    observe test_db with suite_id: "stack_tests" as tests;
    
    act test_executor with tests: tests as red_results;
    
    decide { red_results.fail_count == red_results.total } then {
        -- All tests fail as expected - red phase complete
        act log with message: "Red phase confirmed: all " + red_results.total + " tests fail";
    } else {
        ret { error: "tests_pass_without_impl", passed: red_results.passed_tests }
    };
    
    -- GREEN PHASE: Implement minimal code
    
    -- Iteration 1: Implement new() and size()
    orient {
        "
        pub struct Stack<T> {
            elements: Vec<T>,
        }
        
        impl<T> Stack<T> {
            pub fn new() -> Self {
                Stack { elements: Vec::new() }
            }
            
            pub fn size(&self) -> usize {
                self.elements.len()
            }
        }
        "
    } as impl_iteration_1;
    
    act code_commit with path: module_path, code: impl_iteration_1;
    
    act test_executor with tests: { filter: "size|new" } as iteration_1_results;
    
    -- Iteration 2: Implement push()
    orient extend_code(impl_iteration_1) with {
        "
        pub fn push(&mut self, value: T) {
            self.elements.push(value);
        }
        "
    } as impl_iteration_2;
    
    act code_commit with path: module_path, code: impl_iteration_2;
    
    act test_executor with tests: { filter: "push" } as iteration_2_results;
    
    -- Iteration 3: Implement peek() and pop()
    orient extend_code(impl_iteration_2) with {
        "
        pub fn peek(&self) -> Result<&T, StackError> {
            self.elements.last()
                .ok_or(StackError::EmptyStack)
        }
        
        pub fn pop(&mut self) -> Result<T, StackError> {
            self.elements.pop()
                .ok_or(StackError::EmptyStack)
        }
        "
    } as impl_iteration_3;
    
    act code_commit with path: module_path, code: impl_iteration_3;
    
    act test_executor with tests: tests as iteration_3_results;
    
    decide { iteration_3_results.all_pass } then {
        act log with message: "Green phase complete: all tests pass";
    } else {
        -- Debug failing tests
        for failure in iteration_3_results.failures do {
            act log with 
                level: "error",
                test: failure.name,
                error: failure.message;
        };
        ret { error: "tests_still_failing", failures: iteration_3_results.failures }
    };
    
    -- REFACTOR PHASE: Clean up implementation
    
    orient {
        -- Add derive macros for common traits
        -- Extract error type
        -- Add documentation
        -- Ensure consistent naming
        "
        /// A LIFO stack implementation
        #[derive(Debug, Clone, Default)]
        pub struct Stack<T> {
            elements: Vec<T>,
        }
        
        /// Errors that can occur during stack operations
        #[derive(Debug, Clone, PartialEq)]
        pub enum StackError {
            EmptyStack,
        }
        
        impl std::fmt::Display for StackError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    StackError::EmptyStack => write!(f, \"Stack is empty\"),
                }
            }
        }
        
        impl<T> Stack<T> {
            /// Creates a new empty stack
            pub fn new() -> Self {
                Self::default()
            }
            
            /// Returns the number of elements in the stack
            pub fn size(&self) -> usize {
                self.elements.len()
            }
            
            /// Returns true if the stack is empty
            pub fn is_empty(&self) -> bool {
                self.elements.is_empty()
            }
            
            /// Pushes a value onto the stack
            pub fn push(&mut self, value: T) {
                self.elements.push(value);
            }
            
            /// Returns a reference to the top element without removing it
            /// 
            /// # Errors
            /// Returns `StackError::EmptyStack` if the stack is empty
            pub fn peek(&self) -> Result<&T, StackError> {
                self.elements.last()
                    .ok_or(StackError::EmptyStack)
            }
            
            /// Removes and returns the top element
            ///
            /// # Errors
            /// Returns `StackError::EmptyStack` if the stack is empty
            pub fn pop(&mut self) -> Result<T, StackError> {
                self.elements.pop()
                    .ok_or(StackError::EmptyStack)
            }
        }
        
        impl<T> From<Vec<T>> for Stack<T> {
            fn from(elements: Vec<T>) -> Self {
                Stack { elements }
            }
        }
        "
    } as refactored_impl;
    
    act code_commit with path: module_path, code: refactored_impl;
    
    -- Verify refactoring didn't break tests
    act test_executor with tests: tests as refactor_results;
    
    act coverage_analyzer with 
        code: refactored_impl, 
        tests: tests 
        as coverage_report;
    
    decide { 
        refactor_results.all_pass and 
        coverage_report.line_coverage >= 80 
    } under coverage_80 then {
        
        ret {
            phase: "tdd_complete",
            status: "success",
            iterations: 3,
            tests_passing: refactor_results.pass_count,
            coverage: coverage_report.line_coverage,
            code_quality: calculate_quality(refactored_impl)
        }
    } else {
        ret {
            error: "refactor_failed",
            tests_pass: refactor_results.all_pass,
            coverage: coverage_report.line_coverage
        }
    }
}

-- ============================================================================
-- Helper Functions
-- ============================================================================

function concat(lists: List<List<T>>) -> List<T> {
    -- Flatten list of lists into single list
}

function map(list: List<T>, f: Function<T, U>) -> List<U> {
    -- Apply function to each element
}

function len(list: List<T>) -> Int {
    -- Return list length
}

function not_empty<T>(stack: Stack<T>) -> Bool {
    stack.size() > 0
}

function extend_code(existing: SourceCode) -> Function<SourceCode, SourceCode> {
    -- Returns function that appends new code to existing
}

function calculate_quality(code: SourceCode) -> Int {
    -- Calculate quality score (0-100) based on:
    -- - Documentation coverage
    -- - Naming conventions
    -- - Complexity metrics
    -- - Code duplication
}
