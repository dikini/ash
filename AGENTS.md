# Ash Project - Agent Guidelines

This document defines how AI agents should collaborate on the Ash workflow language project.

## Project Language Constraint

- Primary implementation language: `rust`
- Current project baseline: `edition 2024`, `rust-version  1.94.0`
- New runtime or core logic should use Rust unless explicitly waived in a spec or plan.

## Changelog and Commits

- Use Common Changelog format: <https://common-changelog.org/>.
- Task-completion work MUST update `CHANGELOG.md`.
- Any staged change in code, tooling, hooks, docs policy, or project workflow MUST stage a matching `CHANGELOG.md` update.
- Use Conventional Commits when possible for commit messages.

## Policy Enforcement

- Install hooks once per clone:
  - `scripts/install-hooks.sh`
- Local `pre-commit` enforces:
  - staged docs evidence
  - staged `CHANGELOG.md` updates for staged implementation/tooling/docs-policy changes
  - `CHANGELOG.md` structure policy
  - Rust formatting
  - `cargo check`
  - `cargo clippy`
  - Rust tests
  - property-test detection
  - fuzz checks when a runnable harness exists
- Local `pre-push` runs the full local gate.

## Dependency Management

**Keep dependencies current to avoid technical debt.**

### Version Policy

- Use **latest stable versions** of all dependencies
- Avoid pinning to old versions unless absolutely necessary
- Update dependencies when:
  - Starting new work
  - Fixing compatibility issues
  - Addressing security advisories

### Cargo.toml Guidelines

```toml
# GOOD: Specific recent version
serde = "1.0"

# GOOD: Using workspace dependencies for consistency
[workspace.dependencies]
tokio = { version = "1.42", features = ["full"] }

# AVOID: Very old pinned versions
old-crate = "=0.1.2"  # Only if critical
```

### Checking for Updates

```bash
# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated

# Update all dependencies
cargo update

# Check for security advisories
cargo install cargo-audit
cargo audit
```

### Breaking Changes

When updating dependencies with breaking changes:
1. Update version in `Cargo.toml`
2. Fix compilation errors
3. Run tests: `cargo test --all`
4. Run clippy: `cargo clippy --all`
5. Update `CHANGELOG.md` with dependency changes

### Workspace Dependencies

Use workspace-level dependencies for consistency across crates:

```toml
# In workspace Cargo.toml
[workspace.dependencies]
serde = "1.0"
uuid = "1.12"

# In crate Cargo.toml
[dependencies]
serde = { workspace = true }
uuid = { workspace = true }
```

## Documentation Workflow

- `docs/specs/` stores canonical behavior and invariant specs.
- `docs/plans/` stores implementation and alignment plans.
- `docs/reference/` stores frozen contract and behavior references needed to work locally.
- `docs/notes/` stores per-subsystem future opportunities, cleanup notes, and non-blocking follow-up observations.
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

### Task File Requirement (CRITICAL)

**Agents MUST NOT implement any task without a corresponding task file in `docs/plan/tasks/`.**

Before starting any implementation:
1. Check if `docs/plan/tasks/TASK-XXX-*.md` exists for the task
2. If the task file is **MISSING**:
   - **STOP** - Do not proceed with implementation
   - Create the task file based on the PLAN-INDEX description and relevant SPEC documents
   - Include: Description, Requirements, TDD Steps, Completion Checklist
   - Ask the user to review if requirements are unclear
3. Only proceed with implementation after the task file exists

This ensures:
- Clear, documented requirements before coding
- Consistent task structure across the project
- Traceability from PLAN-INDEX → Task File → Implementation → Tests

### For Each PLAN-INDEX Task

```
1. MAIN AGENT reads task requirements
   └─> Verifies task file exists in docs/plan/tasks/
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
- [ ] CHANGELOG.md updated with entry for this task
- [ ] Code review completed, issues addressed
- [ ] Task status updated in PLAN-INDEX.md

### Changelog Guidelines

Every task completion MUST include a CHANGELOG.md entry following [Common Changelog](https://common-changelog.org/) format.

**Where to add:** Add entries to the `[Unreleased]` section at the top.

**Format:**
```markdown
## [Unreleased]

### Added
- Effect lattice implementation with join/meet operations (TASK-001). The effect system tracks computational power from epistemic (read-only) through operational (side effects).

### Changed
- Refactored parser error handling for better diagnostics (TASK-015).

### Fixed
- Corrected partial ordering in Effect::partial_cmp (TASK-001).
```

**Categories:**
- `Added` - New features, capabilities, modules
- `Changed` - Changes to existing functionality
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Fixed` - Bug fixes
- `Security` - Security-related changes

**Entry content:**
- One line per significant change
- Include task reference in parentheses: `(TASK-001)`
- Brief description of what changed
- Optional: brief rationale (after period)

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
