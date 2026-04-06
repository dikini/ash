---
status: drafting
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-405, TASK-406, TASK-407]
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

## Current Runtime Follow-On Status

The first concrete runtime-side follow-on for the frozen MCE-007 true residual drift set is now in progress/completed via [TASK-405](../../plan/tasks/TASK-405-authoritative-runtime-outcome-state-classification.md).

TASK-405 deliberately takes the narrowest contract-first slice of the runtime cleanup space:

- add one authoritative public runtime outcome/state classification in `ash-interp`;
- wire current interpreter/runtime-facing surfaces into that classification, especially `ExecError`, `ControlLinkError`, and `LinkState`;
- make blocked/suspended, invalid/terminated, execution-failure, and terminal-success classes directly observable without claiming that cumulative semantic-carrier packaging, retained completion payloads, or helper-backed `Par` aggregation are solved.

This means MCE-008 should now treat runtime-state classification as the first implemented cleanup/follow-on step after the MCE-007 closeout corpus, while keeping the broader runtime cleanup agenda open.

TASK-406 now provides the retained completion-observation carrier slice in `ash-interp`: a minimal retained terminal record keyed by control target and exposed through `RuntimeState`. [TASK-407](../../plan/tasks/TASK-407-spawned-child-execution-substrate-and-completion-sealing.md) then adds the missing spawned-child execution substrate: `RuntimeState` now owns a narrow child-workflow registry keyed by `workflow_type`, `Workflow::Spawn` can launch a real child execution path when such an entry is registered, and that child lifecycle now drives automatic retained completion sealing honestly while still preserving useful live supervisor control authority immediately after spawn. [TASK-408](../../plan/tasks/TASK-408-richer-retained-completion-payload-contents.md) now enriches that retained carrier with one honest `CompletionPayload.result`-like slice by preserving `RetainedCompletionRecord.result: Option<Box<ExecResult<Value>>>` plus the `terminal_result()` accessor for child-owned completions while keeping control tombstones distinct as `result: None`. [TASK-409](../../plan/tasks/TASK-409-retained-completion-effect-summary-contents.md) then adds the next conservative slice: `RetainedCompletionRecord.effects: Option<ConservativeRetainedEffectSummary>` with `conservative_effect_summary()`, `ConservativeRetainedEffectSummary::terminal()`, and `ConservativeRetainedEffectSummary::reached()`. That new effect-summary field is intentionally conservative rather than exact trace transport: `terminal` is a runtime-derived upper-bound summary of the effect layers the child workflow may terminate with, and `reached` is a conservative set of runtime-visible effect layers derivable from the workflow forms the current interpreter can honestly summarize today. [TASK-410](../../plan/tasks/TASK-410-retained-completion-obligations-contents.md) now adds one honest `CompletionPayload.obligations`-like slice based on terminal-visible local pending obligations plus terminal-visible active-role pending/discharged state. [TASK-411](../../plan/tasks/TASK-411-retained-completion-provenance-contents.md) now adds one honest retained provenance slice based on runtime-owned child identity and spawn lineage without claiming exact terminal `π'` transport. [TASK-412](../../plan/tasks/TASK-412-dedicated-completion-wait-carrier.md) now adds one dedicated wait surface for that same retained completion record via `RuntimeState::wait_for_retained_completion(&ControlLink) -> Result<RetainedCompletionRecord, ControlLinkError>`, so callers can await the first sealed terminal observation without polling. The implementation remains intentionally conservative: the child entry contract is only that the evaluated spawn `init` value is bound into child context as `init`, and the retained carrier still does not claim full `SPEC-004` `CompletionPayload` parity.

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
