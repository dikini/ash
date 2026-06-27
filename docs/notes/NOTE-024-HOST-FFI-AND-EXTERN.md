# NOTE-024: Host/FFI and Extern

**Date:** 2026-06-27
**Status:** Living document — consolidated host/FFI design space; `extern` reserved but
unspecified; `builtin(...)` is the only host-reaching mechanism in the current target
**Purpose:** Consolidate all prior extern and host/FFI design ideas from NOTE-013, NOTE-014,
NOTE-018, and NOTE-019 into a single note. Establish the current target position: `extern` is
a reserved keyword with no grammar production, `builtin(...)` is the only host-reaching
mechanism, and future FFI work is captured here as a design space, not a commitment.

Companion to NOTE-022 (declaration side: interfaces), NOTE-023 (dispatch side: handlers),
NOTE-013 (handler composition algebra), and NOTE-014 (contract systems).

## Pre-Spec Delta

This note is pre-spec. When the project moves to spec updates:

- **`extern` keyword reservation:** NOTE-024 records that `extern` is reserved in the target
  keyword set but has no grammar production. SPEC-095b should list it as reserved.
- **`builtin(...)` as sole host mechanism:** NOTE-024 confirms that `builtin(...)` is the only
  host-reaching surface form. SPEC-095b §4.1 already describes this.
- **No `extern fn` production:** If a future FFI design revives extern syntax, it will go
  through a new spec packet. The ideas collected here are the starting point.

## 0. Motivation

Prior notes (NOTE-013 §11.1, NOTE-014 §8, NOTE-018 §3.3) each documented host/FFI extern
proposals independently, with overlapping but inconsistent syntax and placement models.
NOTE-022 invalidated Placement A (externs attached to interfaces). TASK-1692 removed `builtin
fn` as a declaration form and established `builtin(...)` as the only host-reaching call. This
note consolidates the remaining design space in one place and records the current target
decision: reserve `extern`, use `builtin(...)`, park FFI.

## 1. Current Target Position

### 1.1 `extern` is reserved, not active

The `extern` keyword is reserved in the target keyword set. There is no `extern` grammar
production in SPEC-095b. No target surface form uses `extern`. If a future host/FFI design
revives the keyword, it will require a new spec packet and must start from the design ideas
collected in §3 of this note.

### 1.2 `builtin(...)` is the only host-reaching mechanism

Trusted stdlib handler/provider methods may call `builtin(...)` to reach runtime-provided
operations. This is defined in SPEC-095b §4.1:

```ebnf
builtin_expr = "builtin" "(" runtime_primitive_symbol { "," expr } ")" ;
runtime_primitive_symbol = qualified_identifier ;
```

The `runtime_primitive_symbol` is deliberately not a string literal. The compiler validates
the primitive key and aligns the surrounding handler method signature with the runtime
primitive descriptor. User libraries cannot introduce new runtime primitive bindings — only
the stdlib/runtime can define them.

### 1.3 No `builtin fn` declaration form

TASK-1692 removed `builtin fn` as a top-level declaration form. Trusted stdlib handlers use
ordinary `fn` methods whose bodies call `builtin(...)`. The `builtin fn` declaration form
exists only in the current (non-target) SPEC-BUILTIN-FN for pure runtime functions like
`string::concat`; it is not part of the target surface.

### 1.4 Compilation strategy: reduce IPC/ABI weight

A key design goal for `builtin(...)` is to guide the compiler toward eliminating or reducing
IPC/ABI overhead:

- `builtin(...)` calls with constant symbol keys are candidate compile-time dispatch sites.
  The compiler knows the exact runtime primitive being invoked and can inline the dispatch
  rather than routing through a generic call mechanism.
- For in-process runtimes (the common case), `builtin(...)` should lower to a direct function
  call into the runtime's primitive table — no serialization, no IPC channel, no ABI boundary
  crossing.
- For distributed or sandboxed runtimes, `builtin(...)` is the single point where IPC/ABI
  weight is incurred. Concentrating all host-reaching calls through one validated mechanism
  means the compiler can optimize the boundary: batch calls, cache results, fuse operations,
  or substitute local implementations when the host runtime is in-process.
- The `runtime_primitive_symbol` being a compiler-validated identifier (not a string) means
  the compiler has full visibility into which host operations are invoked and can apply
  whole-program optimization across handler boundaries.

This is a deliberate constraint: by funneling all host interaction through `builtin(...)`, the
compiler has a single optimization surface. A future `extern` mechanism would either need to
lower through the same `builtin(...)` path (so the compiler still sees one mechanism) or
introduce a second optimization surface, which adds complexity.

## 2. The Static Invariant

The current target surface preserves this rule from NOTE-013/014:

```text
ordinary Ash code calls typed operations (interface methods);
trusted stdlib handler bodies call builtin(...);
nothing else reaches the host.
```

This means:

- The operation interface (`interface Fs { fn read(...) }`) is the public typed surface.
- The handler clause is the only bridge between the operation and `builtin(...)`.
- `builtin(...)` is callable only inside trusted stdlib handler/provider method bodies.
- User-authored code never sees `builtin(...)` directly — it sees the operation call.

## 3. Design Space: Future Host/FFI (Parked)

This section collects the extern/FFI ideas from NOTE-013 §11.1 and NOTE-014 §8 for future
reference. These are **not** the current target position. They are archived here so a future
FFI spec can start from documented prior art rather than rediscovering it.

### 3.1 The two-placement model (archived)

NOTE-013 and NOTE-014 proposed an `extern unsafe fn` declaration form with two placements.
**Placement A was invalidated by NOTE-022** (externs no longer attach to interfaces).
Placement B remains as a candidate shape for future handler-local FFI hooks.

#### Placement A: extern attached to the interface (INVALIDATED)

```ash
// INVALIDATED by NOTE-022 — externs do not attach to interfaces
interface Fs {
    fn read(path: String) -> String;
}

extern unsafe read_host(path: HostString) -> HostResult<HostBytes>
    abi: "ash-host-v1"
    symbol: "fs.read_file"
    for Fs
```

NOTE-022 settled that the interface declares operation signatures only. Externs are a
dispatch-side implementation concern and do not belong in or near the interface declaration.

#### Placement B: extern inside a trusted handler (candidate for future FFI)

```ash
interface Fs {
    fn read(path: Path) -> String;
}

handler PosixFs for Fs
    trusted
    requires host posix_fs
{
    extern unsafe posix_read_file(path: HostCString) -> HostResult<HostBytes>
        abi: "posix-host-v1"
        symbol: "read_file"

    read(path, resume) {
        let raw = unsafe posix_read_file(host.to_c_string(path))
        match decode_file_result(raw) {
            Ok(contents) => resume(decode_utf8(contents))
            Err(err) => raise FsError.from_host(err)
        }
    }
}
```

This placement keeps the extern inside the handler that owns it. The effect interface stays
pure semantic surface; each handler carries its own backend ABI details. This shape remains
viable for a future FFI spec but is not part of the current target language.

### 3.2 The four obligation layers (from NOTE-014)

A trusted handler that reaches the host carries four kinds of obligations. These are
documented here for future FFI design — the current `builtin(...)` mechanism handles them
implicitly through compiler validation and runtime trust:

| Layer | Obligation | Current discharge (`builtin(...)`) | Future FFI discharge (`extern`) |
|-------|------------|-------------------------------------|---------------------------------|
| ABI safety | host values, ownership, async/blocking, raw error convention | runtime-validated primitive descriptor | `unsafe` boundary, audit evidence |
| Semantic correctness | operation pre/postconditions and handler theory | Hoare contracts and laws on the handler | Hoare contracts and laws on the handler |
| Authority | operation is permitted in this execution context | row discharge / admission evidence | row discharge / admission evidence |
| Contract evidence | proof/audit trail of the operation's execution | trusted stdlib boundary | trusted handler + audit evidence |

### 3.3 The failure taxonomy at the host boundary (from NOTE-018)

When host interaction fails, the failure cause should remain distinct in diagnostics:

| Cause | Boundary classification |
|-------|------------------------|
| operation not admitted | authority/admission failure |
| provider missing | admission/runtime configuration failure |
| operation precondition fails | contract violation |
| policy denies operation | policy denial |
| host call fails | host ABI/provider failure |
| host result cannot decode | host ABI/provider failure, possibly contract violation |
| user wants recoverable operation error | operation declares result/failure protocol |

### 3.4 Why extern is parked, not killed

The keyword is reserved (not removed) because:

1. A future host/FFI design is likely needed for C library interop, system calls, or foreign
   code linking.
2. The `builtin(...)` mechanism is intentionally limited to runtime-compiled-in primitives.
   It cannot express dynamic symbol resolution, ABI versioning, or calling conventions.
3. Reserving the keyword now prevents grammar conflicts if the mechanism is later specified.
4. The design space (§3.1–3.3) is documented so future work has a starting point.

The constraint on any future revival: a future `extern` mechanism must either lower through
`builtin(...)` (preserving the single optimization surface) or justify a second host-reaching
path with a clear compilation strategy that does not regress the IPC/ABI weight reduction goal
(§1.4).

## 4. What Changed From Prior Notes

| Prior position | Source | Current position |
|---|---|---|
| `extern unsafe fn` with Placement A (interface-attached) | NOTE-013 §11.1, NOTE-014 §8 | **Invalidated** by NOTE-022. Archived in §3.1. |
| `extern unsafe fn` with Placement B (handler-local) | NOTE-013 §11.1, NOTE-014 §8 | **Parked.** Candidate for future FFI. Archived in §3.1. |
| `builtin fn` as declaration form | SPEC-BUILTIN-FN (current) | **Removed** from target by TASK-1692. Target uses ordinary `fn` + `builtin(...)`. |
| `extern fn` reserved for future FFI | SPEC-BUILTIN-FN, NOTE-018 §3 | **Still reserved**, but now consolidated in this note. |
| Two-placement extern as active design | NOTE-018 §3.3 | **Replaced.** Current target uses `builtin(...)` only. |

## 5. Open Questions

1. **Future FFI mechanism.** If/when extern is revived, does it lower through `builtin(...)`
   or introduce a separate dispatch path? The compilation strategy constraint (§1.4) applies.
2. **Host ABI metadata.** `builtin(...)` has no syntax for ABI version, calling convention, or
   symbol name. These are implicit in the runtime primitive descriptor. A future extern would
   need explicit metadata — where does it live?
3. **User library FFI.** Currently, user libraries cannot introduce new runtime primitive
   bindings. Is this a permanent constraint, or will trusted packages eventually declare
   their own `builtin(...)` keys or extern hooks?
4. **IPC/ABI weight reduction scope.** §1.4 sketches the compilation strategy. Which
   specific optimizations (inlining, batching, caching, fusion) should be specified as
   compiler obligations vs left as implementation freedom?

## 6. References

Internal references:

- [NOTE-022: Effects as Interfaces — Declaration Side](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-023: Handler Surface — Dispatch Side](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md) — §11.1 (archived extern proposals)
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md) — §8 (archived extern proposals)
- [NOTE-018: Boundary Discipline](NOTE-018-BOUNDARY-DISCIPLINE.md) — §3.3 (archived extern placement)
- [NOTE-019: Target Ash Convergence Plan](NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md) — §4.4
- [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md) — §4.1 (`builtin_expr`)
- [SPEC-BUILTIN-FN](../spec/SPEC-BUILTIN-FN.md) — current (non-target) `builtin fn` spec

## 7. Changelog

- 2026-06-27: Initial version. Consolidated all extern/FFI ideas from NOTE-013/014/018/019
  into one note. Established current target position: `extern` reserved but unspecified,
  `builtin(...)` is the only host-reaching mechanism, `builtin fn` declaration form removed.
  Added the compilation strategy goal: reduce or eliminate IPC/ABI weight through a single
  validated host-reaching path. Archived Placement A (invalidated by NOTE-022) and Placement
  B (candidate for future FFI) with the four obligation layers and failure taxonomy.
