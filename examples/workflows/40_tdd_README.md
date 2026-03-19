# TDD Workflow Examples

These examples demonstrate how Ash can orchestrate complex development workflows, specifically Test-Driven Development (TDD) processes involving multiple roles and iterative cycles.

## Files

- `40_tdd_workflow.ash` - Generic TDD workflow template showing the complete lifecycle
- `40a_tdd_concrete_example.ash` - Concrete implementation of TDD for a Stack data structure

## Overview

The TDD workflow examples demonstrate:

1. **Multi-role orchestration** - Developer, Tester, and Reviewer roles with distinct authorities and obligations
2. **Iterative development cycles** - Red/Green/Refactor phases with loop constructs
3. **Policy enforcement** - Automated checks for TDD principles (test-first, coverage requirements)
4. **Obligation tracking** - Ensures roles fulfill their responsibilities
5. **Decision gates** - Policy-based approval workflows

## The TDD Lifecycle in Ash

### Phase 1: Analysis & Design (Orient)

```ash
observe task_manager with task_id: task_id as task;

orient {
    let requirements = parse_requirements(task.description);
    let functions = identify_functions(requirements);
    let types = infer_types(functions);
    -- ... create design document
} as design;
```

The workflow starts by fetching the task and creating a design document that outlines the solution.

### Phase 2: Test Definition (Testing)

```ash
oblige tester must_define_property_tests {
    task: task_id,
    for_each: design.functions
};

for func in design.functions do {
    orient define_property_tests(func, design.types) as property_tests;
    orient define_unit_tests(func, design.types) as unit_tests;
    orient define_integration_tests(func, design.types) as integration_tests;
    
    act test_definer with 
        function_name: func.name,
        test_cases: concat(property_tests, unit_tests, integration_tests);
}
```

The tester defines three types of tests:
- **Property tests** - Invariants that should always hold (associativity, commutativity, etc.)
- **Unit tests** - Edge cases, boundary conditions, specific examples
- **Integration tests** - Tests involving multiple components

### Phase 3: Red Phase - Confirm Tests Fail

```ash
act test_runner with test_suite: test_suite as red_results;

decide { red_results.all_failed } under red_phase_required then {
    -- Red phase successful
} else {
    ret { error: "invalid_test_setup" }
}
```

Before any implementation, all tests are run to confirm they fail. This ensures tests are actually testing something meaningful.

### Phase 4: Green Phase - Minimal Implementation

```ash
loop {
    orient {
        implement_minimal(current_code, func, func_tests.failing)
    } as implementation;
    
    act code_writer with module_path: func.module_path, code: implementation;
    
    act test_runner with test_suite: func_tests as test_results;
    
    decide { test_results.all_pass } then { break }
    else { continue }
}
```

The developer iteratively implements just enough code to make tests pass.

### Phase 5: Refactor Phase - Clean Code

```ash
orient {
    let refactored = refactor(current_code);
    verify_no_code_duplication(refactored);
    verify_clean_architecture(refactored);
    refactored
} as clean_implementation;

act code_writer with module_path: func.module_path, code: clean_implementation;

-- Verify tests still pass after refactoring
act test_runner with test_suite: func_tests as refactor_results;
```

Code is cleaned up while ensuring tests continue to pass.

### Phase 6-8: Review and Finalization

The workflow includes code review with separation of duties (reviewer cannot be developer), addressing feedback, and final quality checks.

## Roles and Responsibilities

### Developer
- **Authority**: Write code, run tests, commit changes
- **Obligations**:
  - `must_write_tests_first` - Cannot commit without tests
  - `must_not_commit_without_tests` - Enforced by policy
  - `must_refactor_before_submit` - Code must be cleaned before review

### Tester
- **Authority**: Define tests, review coverage, approve deployments
- **Obligations**:
  - `must_define_property_tests` - Property-based testing required
  - `must_ensure_red_phase_first` - Verify tests fail initially
  - `must_verify_coverage` - Minimum 80% coverage

### Reviewer
- **Authority**: Review code, approve/reject, request changes
- **Obligations**:
  - `must_not_review_own_code` - Enforced by `reviewer_separation` policy
  - `must_check_test_quality` - Review test completeness
  - `must_verify_no_duplication` - Check for DRY violations

## Policies

### red_phase_required
```ash
policy red_phase_required:
    when function_implemented and tests_not_failing
    then deny
    else permit
```

Ensures tests are written before implementation and actually fail initially.

### tests_must_pass_before_review
```ash
policy tests_must_pass_before_review:
    when submit_for_review and has_failing_tests
    then deny
    else require_approval(role: tester)
```

Blocks submission to review if tests are failing.

### reviewer_separation
```ash
policy reviewer_separation:
    when review_assigned and reviewer_is_developer
    then deny
    else permit
```

Enforces separation of duties - developer cannot review their own code.

### coverage_minimum
```ash
policy coverage_minimum:
    when coverage < 80%
    then require_approval(role: tester)
    else permit
```

Requires tester approval if coverage is below threshold.

## Concrete Example: Stack Implementation

The `40a_tdd_concrete_example.ash` file shows a concrete TDD implementation for a Stack data structure with:

### Property Tests
- `push_pop_inverse` - Push then pop restores original stack
- `peek_is_pure` - Peek doesn't modify state
- `push_increments_size` - Size increases by exactly 1
- `pop_decrements_size` - Size decreases by exactly 1
- `lifo_order` - Last in, first out ordering

### Unit Tests
- Empty stack operations (pop/peek fail appropriately)
- Single element operations
- Multiple element operations

### Integration Tests
- Stack as collection adapter
- Stack with iterator

### Implementation Iterations

1. **Iteration 1**: `new()` and `size()`
2. **Iteration 2**: `push()`
3. **Iteration 3**: `peek()` and `pop()`
4. **Refactor**: Add documentation, error types, derive macros

## Required Capabilities

To run these workflows, the following capabilities would need to be implemented:

### Task Management
- `task_manager:observe(task_id)` - Fetch task details
- `task_updater:act(task_id, status)` - Update task status

### Test Management
- `test_registry:observe(suite_id)` - Fetch test suite
- `test_definer:act(function, tests)` - Register tests
- `test_runner:act(suite)` - Execute tests

### Code Repository
- `code_reader:observe(path)` - Read source code
- `code_writer:act(path, code)` - Write source code
- `type_checker:act(code)` - Type check code

### Review System
- `review_assigner:act(task, reviewer)` - Assign reviewer
- `review_submitter:act(task, feedback)` - Submit review
- `review_reader:observe(task)` - Read review status

### Quality Analysis
- `coverage_analyzer:act(code, tests)` - Calculate coverage
- `quality_checker:act(code)` - Check code quality

## Running the Examples

Currently, these are **outline workflows** that demonstrate the Ash language's capabilities for orchestrating complex processes. To actually run them:

1. Implement the required capability providers (see `../providers/`)
2. Set up the runtime context with appropriate providers
3. Run with: `ash run 40_tdd_workflow.ash -- '{"task_id": "TASK-001"}'`

## Key Ash Features Demonstrated

- **Multi-role workflows** with distinct authorities
- **Obligation tracking** ensuring responsibilities are met
- **Policy enforcement** at decision points
- **Loop constructs** for iterative development
- **Pattern matching** on test results
- **Parallel composition** (can be used for parallel test execution)
- **Effect tracking** (evaluative for tests, operational for code changes)

## Related Examples

- Level 4 examples show simpler policy and role usage
- Level 5 examples demonstrate real-world integration patterns
- Level 6 examples show advanced patterns like circuit breakers
