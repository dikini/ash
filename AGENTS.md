# Ash Project - Agent Guidelines

This document defines how AI agents should collaborate on the Ash workflow language project.

## Core Principles

### 1. Always Use Available Skills

Before starting any work, invoke the relevant skills:

```
/rust-skills    - For any Rust code (coding, reviewing, refactoring)
/superpowers    - For understanding capability system and advanced features
```

**Why**: These skills contain 179+ rules for Rust and specialized knowledge for Ash. Using them prevents common errors and ensures consistency.

### 2. Sub-Agent Task Architecture

All substantial work follows the **Main Agent / Sub-Agent** pattern:

```
┌─────────────────────────────────────────────────────────────────┐
│                        MAIN AGENT                               │
│  - Understands full context and requirements                    │
│  - Breaks work into focused sub-tasks                           │
│  - Spawns specialized sub-agents                                │
│  - Reviews and verifies all results                             │
│  - Integrates sub-agent outputs                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   TEST DEV   │    │   CODE DEV   │    │      QA      │
│   Sub-Agent  │    │   Sub-Agent  │    │  Sub-Agent   │
└──────────────┘    └──────────────┘    └──────────────┘
```

**Never** have the main agent do implementation work directly when sub-agents are available.

## Specialized Sub-Agents

### Test Development Agent

**Spawned by**: Main agent when test file needs creation

**Responsibilities**:
- Write property-based tests using `proptest`
- Ensure tests fail before implementation exists (TDD)
- Cover edge cases and invariants
- Follow test naming: `proptest!` blocks, descriptive test names

**Input from Main Agent**:
- What module/function to test
- Expected properties/invariants
- Relevant types and interfaces

**Output to Main Agent**:
- Complete test file
- Explanation of test strategy
- Property definitions

**Example Invocation**:
```rust
// Main agent spawns test dev sub-agent:
"Write property tests for Effect lattice operations.
Expected properties: associativity, commutativity, idempotence.
Effect enum has variants: Epistemic, Deliberative, Evaluative, Operational."
```

### Code Development Agent

**Spawned by**: Main agent after tests are written

**Responsibilities**:
- Implement code to make tests pass
- Follow Rust best practices from /rust-skills
- Add documentation comments
- Ensure `cargo check` and `cargo clippy` pass

**Input from Main Agent**:
- Test file content (or path)
- Specification/requirements
- Related module context

**Output to Main Agent**:
- Implementation code
- Any interface changes needed
- Notes on design decisions

**Example Invocation**:
```rust
// Main agent spawns code dev sub-agent:
"Implement PartialOrd for Effect enum to satisfy these tests:
[test content]. Effect variants form a lattice with Epistemic < Deliberative < Evaluative < Operational."
```

### QA Agent

**Spawned by**: Main agent for verification

**Responsibilities**:
- Run full test suite: `cargo test`
- Check formatting: `cargo fmt --check`
- Run clippy: `cargo clippy --all-targets --all-features`
- Verify documentation builds: `cargo doc`
- Check for warnings or errors

**Input from Main Agent**:
- What to verify (specific crate or whole workspace)
- Expected test outcomes

**Output to Main Agent**:
- Test results summary
- Any failures or warnings
- Recommendations for fixes

**Example Invocation**:
```
"Verify ash-core crate passes all tests and quality checks.
Expected: All tests pass, no clippy warnings, formatting clean."
```

### Code Review Agent

**Spawned by**: Main agent before considering work complete

**Responsibilities**:
- Review code against /rust-skills guidelines
- Check for safety issues
- Verify error handling
- Assess API design
- Identify potential bugs

**Input from Main Agent**:
- File(s) to review
- Context of changes
- Any specific concerns

**Output to Main Agent**:
- Review comments
- Severity classification (nit, suggestion, issue, blocking)
- Recommended fixes

**Example Invocation**:
```
"Review crates/ash-core/src/effect.rs for:
1. Safety - any unsafe code concerns
2. Error handling - proper Result usage
3. API design - intuitive public interface
4. Performance - unnecessary allocations"
```

## Task Workflow

### For Each PLAN-INDEX Task

```
1. MAIN AGENT reads task requirements
   └─> Invokes /rust-skills if Rust code involved

2. MAIN AGENT spawns TEST DEV sub-agent
   └─> "Write property tests for [task]"
   └─> Reviews test output, provides feedback if needed

3. MAIN AGENT spawns CODE DEV sub-agent  
   └─> "Implement to satisfy these tests: [tests]"
   └─> Reviews implementation, guides if off-track

4. MAIN AGENT spawns QA sub-agent
   └─> "Verify tests pass and code quality"
   └─> Reviews QA report

5. MAIN AGENT spawns CODE REVIEW sub-agent
   └─> "Review implementation for issues"
   └─> Addresses review feedback (may loop to step 3)

6. MAIN AGENT integrates and finalizes
   └─> Updates task status in PLAN-INDEX.md
   └─> Commits with descriptive message
```

### Task Completion Criteria

Before marking a task complete, the main agent MUST verify:

- [ ] Property tests written and passing (TDD)
- [ ] Implementation satisfies all tests
- [ ] `cargo test` passes for affected crate
- [ ] `cargo clippy --all-targets` clean
- [ ] `cargo fmt --check` clean
- [ ] Documentation comments added
- [ ] Code review completed, issues addressed
- [ ] Task status updated in PLAN-INDEX.md

## Communication Protocol

### Main Agent → Sub-Agent

Always provide:
1. **Context**: What we're building and why
2. **Requirements**: Specific functionality needed
3. **Constraints**: Performance, safety, API boundaries
4. **References**: Relevant files, specs, examples

```
Good: "Implement Effect::join for the effect lattice (crates/ash-core/src/effect.rs).
The join operation computes the least upper bound. Effect ordering:
Epistemic < Deliberative < Evaluative < Operational.
Must satisfy tests in [path] that verify associativity, commutativity, idempotence.
See SPEC-001-IR.md Section 3.2 for lattice axioms."

Bad: "Implement the join method."
```

### Sub-Agent → Main Agent

Always return:
1. **What was done**: Summary of changes
2. **Key decisions**: Design choices made
3. **Issues encountered**: Problems and resolutions
4. **Next steps**: What main agent should do

## Project-Specific Guidelines

### Rust Conventions

From /rust-skills, prioritize:
- **Safety First**: Minimize unsafe, document invariants
- **Error Handling**: Use `thiserror` for library errors
- **Async**: Use `tokio`, propagate `async` properly
- **Testing**: Property tests with `proptest`
- **Documentation**: All public items have doc comments

### Ash-Specific Patterns

- **Effect Tracking**: Every operation tracks its effect level
- **Provenance**: Audit trails required for operational effects
- **Capability Safety**: Capabilities checked at runtime
- **Policy Conflicts**: Z3 SMT for compile-time detection (feature-gated)

### File Organization

```
crates/ash-core/src/
├── lib.rs          # Public exports
├── effect.rs       # Effect lattice
├── workflow.rs     # AST types
├── value.rs        # Runtime values
├── provenance.rs   # Audit trail types
└── capability.rs   # Capability definitions

docs/
├── design/         # Architecture, decisions
├── spec/           # Formal specifications
└── plan/           # Task tracking
```

## Example: TASK-001 Session

```
User: "Implement TASK-001: Effect Lattice Properties"

Main Agent:
1. Reads TASK-001-effect-lattice.md
2. Invokes /rust-skills
3. Spawns TEST DEV: "Write property tests for Effect lattice"
4. Reviews tests, approves
5. Spawns CODE DEV: "Implement Effect to satisfy these tests"
6. Reviews implementation
7. Spawns QA: "Verify ash-core tests and quality"
8. Reviews QA report (all pass)
9. Spawns CODE REVIEW: "Review effect.rs implementation"
10. Addresses review feedback
11. Updates PLAN-INDEX.md (TASK-001: done)
12. Commits changes
13. Reports completion to user
```

## Tool Usage

### Preferred Tools

- **File Operations**: `ReadFile`, `WriteFile`, `StrReplaceFile`
- **Search**: `Grep` (always use over shell grep)
- **Execution**: `Shell` for cargo commands, limited to 60s
- **Sub-Agents**: `Task` for delegating work

### Avoid

- Direct git mutations (commit, push, rebase) - ask user first
- Long-running shell commands without timeout
- Modifying files outside project directory

## Questions or Issues?

If guidelines are unclear:
1. Check /rust-skills for Rust-specific guidance
2. Check /superpowers for capability system details
3. Refer to SPEC documents in docs/spec/
4. When in doubt, ask the main agent (or user)
