---
status: candidate
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: [TASK-359, TASK-360, TASK-361, TASK-362, TASK-363, TASK-364, TASK-365, TASK-366, TASK-367, TASK-368, TASK-369]
tags: [entry-point, runtime, capabilities]
---

# MCE-001: How Does an Ash Program Start?

## Problem Statement

We lack a clear specification for Ash program entry points. What does the runtime provide to a starting workflow? How are initial capabilities injected? What is the contract between the runtime and the first workflow instance?

This is foundational: entry point design affects capability injection, obligation tracking initialization, and the runtime API.

## Scope

- **In scope:**
  - Entry point declaration syntax
  - Initial capability injection
  - Runtime-to-workflow contract at startup
  - Command-line argument handling (if any)
  - Environment/configuration access

- **Out of scope:**
  - Runtime implementation details (scheduler, etc.)
  - Hot reloading
  - Distributed startup

- **Related but separate:**
  - MCE-003: Functions vs capabilities (entry point may be a special capability)

## Current Understanding

### What we know

- Workflows are the primary unit of computation
- Workflows declare requirements and obligations
- Capabilities are the interface to external resources
- Runtime must provide some initial capabilities (stdin, stdout, etc.)

### What we're uncertain about

- Is the entry point a special workflow type or just a regular workflow?
- How are initial capabilities named and typed?
- Can there be multiple entry points (library vs executable)?
- What is the minimal set of "boot" capabilities?

## Design Dimensions

| Dimension | Option A: Explicit Main | Option B: Capability-Main | Option C: Annotated Workflow |
|-----------|------------------------|---------------------------|------------------------------|
| Entry syntax | `main()` function | `cap Main` capability | `#[entry] workflow main` |
| Cap injection | Explicit parameters | Capability interface | Implicit + explicit mix |
| Runtime contract | C-style argc/argv | Capability-based | Workflow-based |

## Proposed Approaches

### Approach 1: Capability as Entry Point

The entry point is a capability that the runtime instantiates and calls.

```ash
-- Surface syntax
cap Main {
  fn run(args: [String]) -> Result<(), Error>
}

impl Main for Runtime {
  workflow run(args: [String]) -> Result<(), Error> {
    -- Program logic here
  }
}
```

**Pros:**
- Unified with rest of capability system
- Clear contract via capability interface
- Testable: can mock Main for testing

**Cons:**
- Indirection for simple programs
- Requires capability syntax even for hello-world

### Approach 2: Annotated Workflow

A specially annotated workflow serves as entry point.

```ash
-- Surface syntax
#[entry]
workflow main(args: [String]) -> Result<(), Error> {
  -- Program logic here
}
```

**Pros:**
- Familiar to programmers
- Simple for hello-world

**Cons:**
- Special case in language
- How are capabilities injected?

### Approach 3: Explicit Boot Capability List

Entry workflow declares which capabilities it needs from runtime.

```ash
-- Surface syntax
workflow main(args: [String])
  requires io.Stdout, io.Stdin
  -> Result<(), Error>
{
  -- Program logic here
}
```

**Pros:**
- Explicit dependencies
- Aligns with capability safety principles

**Cons:**
- Verbose
- Runtime must provide specific capabilities

## Consolidated Design Direction (In Progress)

Based on discussion, the following decisions have been reached:

### 1. Supervision Model

**Decision:** Hidden system supervisor (Option 3)

- Runtime spawns a default system supervisor (implementation detail, not user-facing)
- System supervisor spawns user main with a proper control link
- Preserves invariant: every workflow has a supervision contract
- System supervisor implements default supervision protocol
- Configurable at system level, not workflow level

**Rationale:** Avoids Rust supervisor (maintenance burden) and "no supervisor" (breaks invariants). The system supervisor is the bridge between runtime ambient authority and Ash supervision semantics.

### 2. Lexical Scoping

**Decision:** Static lexical scoping for capabilities

- Capabilities in scope = defined in current file + imported via `use`
- Ash uses `use` syntax (confirmed from std/src/prelude.ash)
- No dynamic scoping, no ambient capability availability

**Example:**
```ash
use io.Stdout        -- Bring capability type into scope
use runtime.Args     -- For command-line arguments

workflow main(
  stdout: cap Stdout,
  args: Args
) -> Result<(), Error> {
  -- body
}
```

### 3. Capability Injection

**Decision:** Main declares capabilities as parameters, runtime satisfies

- Entry point is a workflow like any other
- Declares required capabilities in parameter list
- Runtime attempts to satisfy from ambient authority
- Fails fast if runtime cannot provide declared capability

**Rationale:** Explicit declaration limits workflow size (intentional burden). Static tracking maintains capability safety.

### 4. Open Questions (Remaining)

1. ✅ **Return type handling** — Resolved: System supervisor forms exit code, returns to Rust runtime
2. ✅ **Error propagation** — Resolved: Supervisor interprets `RuntimeError` and obligation discharge status
3. ✅ **Args representation** — Resolved: `args` is a capability from standard library
4. ✅ **Library vs executable** — Resolved: No special distinction. A file with `main` can be both run directly and imported as a library. The CLI determines the entry point.
5. ✅ **Minimal boot capability set** — Resolved: Runtime provides file loading/compiling; everything else through libraries

#### Resolved: Args as Capability

**Decision:** Command-line arguments are provided via a capability in the standard library, not as a special parameter type.

**Example:**
```ash
use runtime.Args

workflow main(
  stdout: cap io.Stdout,
  args: cap Args
) -> Result<(), RuntimeError> {
  let name = args.get(0);
  ...
}
```

**Rationale:** Consistent with capability-passing design. No special cases in the language. Args capability provided by runtime, accessed through standard library interface.

#### Resolved: Return Type Handling & Error Propagation

**Decision:** System supervisor forms exit code and returns to Rust runtime, which exits the OS process.

**Flow:**
```
User workflow completes
    ↓
Returns via control link: (Result<T, RuntimeError>, obligations, provenance, effects)
    ↓
System supervisor forms exit code:
    - Ok(()) + obligations discharged → 0
    - Ok(()) + obligations pending → error code (TBD)
    - Err(RuntimeError code msg) → code
    ↓
Returns Int to Rust runtime → OS exit
```

**Supervisor's judgment:**
- Interprets `RuntimeError.exit_code` from `Err` case
- Checks obligation discharge status
- (Future) May incorporate provenance/effects for audit logging

**Note:** Only 0 for success is defined. Standard menu of error return codes for error classes to be defined (see Discovered Issues).

#### Resolved: Library vs Executable

**Decision:** No special distinction between library and executable files.

**Rules:**
- One file contains one workflow definition
- `ash file.ash` runs the `main` workflow in that file (CLI determines entry point)
- `use module` imports workflows for spawning (can import `main` or other workflows)
- A file with `main` can be both run directly AND imported as a library

**Example:**
```ash
-- server.ash
workflow main(args: cap Args) -> Result<(), RuntimeError> {
  -- server logic
}
```

```bash
ash server.ash              # Runs main directly
```

```ash
-- client.ash
use server  -- Imports server::main

workflow main(args: cap Args) -> Result<(), RuntimeError> {
  let handle = spawn server::main { init: () };  -- Can spawn it
  ...
}
```

**Rationale:** Simple, no file extensions or configuration needed. The CLI invocation determines whether a file is treated as entry point or library.

#### Resolved: Minimal Boot Capability Set

**Decision:** Runtime provides only the initial load sequence; everything else comes through libraries.

**Runtime provides:**
- File reading and loading
- Compilation and typechecking
- Library resolution for `use`

**Everything else through libraries:**
- System supervisor lives in stdlib (`use runtime.Supervisor`)
- Spawn capability exported via library (`use runtime.Spawn`)
- Built-in capabilities (Args, stdout, etc.) all via stdlib

**Bootstrap flow:**
```
Runtime (Rust)
    │
    ├── Read stdlib files (including system supervisor)
    ├── Compile/typecheck stdlib
    ├── Locate entry file (CLI arg)
    ├── Compile entry file, verify `main` exists with correct signature
    │
    └── Spawn system supervisor from stdlib
        └── System supervisor spawns `main` from entry file
```

**Rationale:** Uniformity — everything is a library, even the supervisor. The only "special" part is the initial file reading/compiling before any workflow runs. After bootstrap, all capabilities are accessed uniformly via `use`.

### 5. Discovered Issues (To Resolve in Syntax/Spec Work)

1. **Variant syntax for tuple-like constructors** — Moved to [TYPES-001](../type-system/TYPES-001-tuple-variants.md). Current spec (SPEC-002, SPEC-020) only defines record-style variants (`Ok { value: T }`). For `RuntimeError Int String`, we need tuple-variant syntax. See that exploration for syntax options and design questions.

2. **What is the type of a workflow?** — Moved to [FIRST-CLASS-WORKFLOWS](../future/FIRST-CLASS-WORKFLOWS.md). Deferred to post-minimal-core. The fundamental question: are workflows first-class values? This was initially considered for entry point but postponed in favor of gradual typing at the Rust/Ash boundary.

3. ✅ **Standard error return code taxonomy** — Resolved: Minimal approach for now.
   - **0** = success
   - **Non-zero** = error (generic)
   
   Detailed error categories deferred. The `RuntimeError.message` provides human-readable details. Supervisor returns `RuntimeError.exit_code` or a generic non-zero code.

## Related Explorations

- MCE-003: Functions vs capabilities
- MCE-008: Runtime cleanup (what runtime provides)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Initial problem identified |
| 2026-03-30 | Hidden system supervisor | Preserves supervision invariant, avoids Rust implementation |
| 2026-03-30 | Static lexical scoping | `use` for imports, no dynamic scoping |
| 2026-03-30 | Capability parameters | Explicit declaration, runtime satisfaction |
| 2026-03-30 | Entry point is regular workflow | No special syntax; type discipline enforced by supervisor |
| 2026-03-30 | Entry = workflow named `main` in file | Fixed name avoids dynamic spawn; `ash file.ash` runs `main` |
| 2026-03-30 | System supervisor spawns `main` statically | Uses regular `spawn main { init: () }`; typechecks normally |
| 2026-03-30 | No library vs executable distinction | CLI determines entry point; files with `main` can be both run and imported |
| 2026-03-30 | Args as capability | `cap Args` from standard library, not special parameter |
| 2026-03-30 | Supervisor forms exit code | Interprets RuntimeError + obligations, returns Int to Rust runtime |
| 2026-03-30 | Error taxonomy deferred | Only 0 = success defined; standard error codes need design |
| 2026-03-30 | Minimal boot set defined | Runtime provides load/compile only; everything else via libraries |
| 2026-03-30 | Error taxonomy minimal | 0 = success, non-zero = error; detailed codes deferred |
| 2026-03-30 | SPEC-first approach | MCE-001 is guidance; normative truth in SPEC-004/005/021 |

## Summary

MCE-001 documents design exploration for Ash program entry points. **This document provides guidance only; normative truth resides in updated SPEC files.**

All core questions resolved in this exploration:

| Aspect | Decision |
|--------|----------|
| CLI command | `ash run <file> [-- <args>...]` (explicit `run` subcommand) |
| Entry workflow | Named `main` in file passed to CLI |
| Supervisor | System supervisor in stdlib, spawns `main`, observes completion via control authority |
| Return type | `Result<(), RuntimeError>` where `RuntimeError { exit_code: Int, message: String }` |
| Error handling | Supervisor forms exit code (0 = success, non-zero = error), returns to Rust runtime |
| Args | `cap Args` from standard library |
| Library vs executable | No distinction; CLI determines entry point |
| Boot capabilities | Runtime provides file loading/compiling; everything else through libraries |
| First-class workflows | Deferred to post-minimal-core |
| Tuple variant syntax | Delegated to TYPES-001 |
| **Critical** | All implementation blocked on SPEC updates (57A tasks) |

## Next Steps

### Completed
- [x] Review for `candidate` status
- [x] Create PLAN-INDEX phase for implementation → [Phase 57](../../plan/PLAN-INDEX.md#phase-57-entry-point-and-program-execution)

### SPEC-First Implementation Path

**Phase 57A (SPEC updates) must precede 57B (implementation):**

| Task | Spec | Focus |
|------|------|-------|
| **TASK-S57-1** | SPEC-004 | Control-link completion payload semantics |
| **TASK-S57-2** | SPEC-005 | Exit-immediately CLI policy |
| **TASK-S57-3** | SPEC-021 | Observable exit behavior |
| **TASK-S57-4** | SPEC-009/012 | Stdlib import/namespace rules |
| **TASK-S57-5** | SPEC-017 | Runtime-provided capability syntax |
| **TASK-S57-6** | SPEC-003/022 | Entry workflow typing contract |

**Then Phase 57B implementation tasks** (359-369) unblocked per dependency matrix in PLAN-INDEX.

### Related Work
- [ ] Coordinate with TYPES-001 for `RuntimeError` syntax (can use record syntax as fallback)
- [ ] Mark MCE-001 as `accepted` after SPEC updates complete
- [ ] Archive MCE-001 to `docs/ideas/archived/`
