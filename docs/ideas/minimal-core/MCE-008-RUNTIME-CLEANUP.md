---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [runtime, capabilities, libraries, ffi, minimal]
---

# MCE-008: Runtime Cleanup — Libraries and Capabilities

## Problem Statement

The runtime support for libraries and capabilities may contain unnecessary complexity from initial exploration. This exploration identifies the minimal runtime surface needed for a sound execution environment.

Goal: A lean runtime that provides exactly what's needed, no more.

## Scope

- **In scope:**
  - Runtime capability injection mechanisms
  - Library loading and linking
  - FFI boundaries
  - Capability registry
  - Initial "boot" capabilities

- **Out of scope:**
  - Runtime scheduler implementation
  - Memory allocator design
  - Platform abstractions (OS-specific code)

- **Related but separate:**
  - MCE-001: Entry point (what runtime provides at start)
  - MCE-003: Functions vs capabilities (affects runtime API)

## Current Understanding

### What we know

- Runtime provides capabilities to workflows
- Capabilities have native (Rust) implementations
- Libraries can define capabilities
- There's some form of capability registry/lookup
- Entry point receives initial capabilities from runtime

### What we're uncertain about

- What is the exact runtime API for capability registration?
- How are capabilities discovered and loaded?
- Is there dynamic linking or only static?
- What capabilities are "built-in" vs user-defined?
- How does the runtime enforce capability safety?

## Runtime Components Audit

| Component | Current State | Needed? | Notes |
|-----------|---------------|---------|-------|
| Capability registry | Exists? | Yes | Core mechanism |
| Library loader | Exists? | Yes | For non-built-in caps |
| FFI boundary | Partial? | Yes | For Rust interop |
| Boot capability set | Undefined | Yes | Minimal set needed |
| Capability introspection | Unknown? | Maybe | For debugging |
| Dynamic capability creation | Unknown? | Maybe | For advanced patterns |

## Boot Capability Set

Minimal set of capabilities every Ash runtime must provide:

| Capability | Purpose | Essential? |
|------------|---------|------------|
| `io.Stdout` | Console output | Yes |
| `io.Stdin` | Console input | For CLI programs |
| `io.Stderr` | Error output | Yes |
| `env.Vars` | Environment access | Probably |
| `fs.FileSystem` | File operations | For most programs |
| `time.Clock` | Time access | Probably |
| `rand.Random` | Randomness | Maybe |

**Question:** What is truly essential vs what can be user-provided?

## Library Loading Models

### Option 1: Static Linking

All capabilities compiled into executable. Libraries are Rust crates compiled with Ash program.

**Pros:** Simple, no runtime loading, type-safe
**Cons:** No dynamic extension, larger binaries

### Option 2: Dynamic Loading

Libraries loaded at runtime from `.so`/`.dll` files.

**Pros:** Extension without recompile, smaller base
**Cons:** Complex, safety concerns, versioning

### Option 3: Capability Scripting

Libraries written in Ash itself, loaded as Ash code.

**Pros:** Unified language, safe by construction
**Cons:** Performance, limited low-level access

**Recommendation:** Start with Option 1 (static), design for future Option 3.

## FFI Design

How do Ash capabilities call Rust code?

```rust
// Rust side
capability! {
    cap Foo {
        fn bar(x: i32) -> i32;
    }
}

impl Foo for Runtime {
    fn bar(&self, x: i32) -> i32 {
        x * 2
    }
}
```

**Questions:**
- How are Ash types mapped to Rust types?
- How are errors handled across boundary?
- How are effects tracked across FFI?

## Capability Registry API

Minimal registry operations:

```rust
// Register a capability implementation
fn register<C: Capability>(impl: C);

// Acquire capability by type
fn acquire<C: Capability>() -> C;

// Check if capability available
fn has<C: Capability>() -> bool;
```

**Questions:**
- How are capability dependencies resolved?
- Is registration static or dynamic?
- How do we prevent capability spoofing?

## Open Questions

1. Can capabilities be parameterized (e.g., `FileSystem` vs `FileSystemAtPath`)?
2. How do capabilities declare their effects?
3. Is there a capability for creating capabilities?
4. How does the runtime handle capability revocation?
5. What's the minimal FFI surface for built-in capabilities?

## Related Explorations

- MCE-001: Entry point (boot capabilities)
- MCE-003: Functions vs capabilities (affects runtime API)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Runtime needs audit |

## Next Steps

- [ ] Inventory current runtime components
- [ ] Define minimal boot capability set
- [ ] Design minimal FFI boundary
- [ ] Document capability registration mechanism
