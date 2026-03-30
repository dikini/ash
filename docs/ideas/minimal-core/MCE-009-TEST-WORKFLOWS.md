---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [testing, examples, validation, workflows]
---

# MCE-009: Test and Example Workflows

## Problem Statement

To validate the minimal execution environment, we need a suite of test workflows and examples that exercise:

- All core language features
- Edge cases and error conditions
- Realistic usage patterns
- Capability interaction patterns

This exploration defines the test/example requirements and validates that the minimal core is sufficient.

## Scope

- **In scope:**
  - Test workflow categorization
  - Example program specification
  - Validation criteria (what "works" means)
  - Test harness requirements

- **Out of scope:**
  - Property-based testing strategy (general infrastructure)
  - Performance benchmarking
  - Fuzzing

- **Related but separate:**
  - All other MCE-* explorations (this validates them)

## Test Categories

### 1. Unit Tests (Language Features)

| Feature | Test Cases | Priority |
|---------|------------|----------|
| Let binding | Simple, shadowing, nested | High |
| If | True branch, false branch, nested | High |
| Match | Patterns, exhaustiveness | Medium |
| Par | Parallel execution, result aggregation | High |
| Call | Sync invocation, argument passing | High |
| Spawn | Async creation, handle return | High |
| Act | Effect recording, capability call | High |
| Observe | Pure observation | Medium |
| Return | Early return, tail return | High |

### 2. Integration Tests (Patterns)

| Pattern | Description | Priority |
|---------|-------------|----------|
| Sequential pipeline | Data processing steps | High |
| Fan-out/fan-in | Parallel map, collect results | High |
| Worker pool | Spawn N workers, distribute work | Medium |
| Request/response | Async call and await | High |
| Supervision | Parent monitors child | Medium |
| Resource cleanup | Obligation discharge on exit | High |

### 3. Capability Tests

| Capability | Test | Priority |
|------------|------|----------|
| io.Stdout | Write and verify output | High |
| io.Stdin | Read and process input | Medium |
| fs.FileSystem | Read/write files | Medium |
| env.Vars | Access environment | Medium |

### 4. Edge Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| Empty workflow | Valid, returns unit? |
| Infinite loop | Detectable? Resource limits? |
| Deep recursion | Stack behavior? |
| Large par branches | Scalability? |
| Nested spawn | Handle hierarchy? |

## Example Programs

### Minimal Set

1. **hello.ash** — Hello world (stdout)
2. **echo.ash** — Echo stdin to stdout
3. **cat.ash** — File copy (fs + io)
4. **parallel_map.ash** — Process list in parallel
5. **worker_pool.ash** — Fixed workers, task queue
6. **supervisor.ash** — Spawn, monitor, restart

### Validation Criteria

For each example:
- [ ] Parses correctly
- [ ] Lowers to IR
- [ ] Passes type/ob obligation checking
- [ ] Executes without runtime errors
- [ ] Produces expected output
- [ ] Discharges all obligations

## Test Harness Requirements

### Minimal Harness

```rust
// Pseudo-code for test runner
fn test_workflow(path: &Path) -> TestResult {
    let source = read_file(path)?;
    let ast = parse(&source)?;
    let ir = lower(ast)?;
    let _ = type_check(&ir)?;  // Including obligations
    let result = interpret(&ir)?;
    Ok(result)
}
```

### Output Verification

Options:
1. **Expected output files:** `test.ash` + `test.expected`
2. **Inline assertions:** `assert output == "..."`
3. **Golden files:** Auto-update expected outputs

### Capability Mocking

For tests, need mock capabilities:

```ash
-- Mock stdout for testing
cap MockStdout {
  captures: Vec<String>
}

impl io.Stdout for MockStdout {
  workflow write(s: String) {
    self.captures.push(s);
    return ()
  }
}
```

## Success Criteria

The minimal core is "sound" when:

1. **All unit tests pass:** Each core feature works in isolation
2. **All integration tests pass:** Patterns compose correctly
3. **All examples run:** Realistic programs execute successfully
4. **No obligation leaks:** All workflows discharge obligations
5. **Effect tracking accurate:** Effects match actual behavior

## Open Questions

1. How do we test async/parallel behavior deterministically?
2. What's the story for testing failing workflows?
3. Do we need a "test" capability for assertions?
4. How do we mock time for time-dependent tests?
5. Should examples be runnable as integration tests?

## Dependencies

This exploration depends on all others being at least `candidate` status:
- MCE-001 (entry point needed to run programs)
- MCE-002 (IR forms define what we test)
- MCE-004/MCE-005 (semantics define expected behavior)
- MCE-008 (runtime needed to execute)

## Related Explorations

- All MCE-* explorations are prerequisites

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need validation criteria |

## Next Steps

- [ ] Draft initial test cases for each category
- [ ] Write example programs
- [ ] Define test harness interface
- [ ] Create mock capability framework
- [ ] Establish CI/integration testing
