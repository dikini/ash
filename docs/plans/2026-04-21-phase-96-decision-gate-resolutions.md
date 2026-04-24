# Phase 96 Decision Gate Resolutions

All four gates resolved April 21, 2026. All are Tier 0/T1 (no Red Team needed).

---

## D1: Module Resolver Location — RESOLVED

**Question:** New `ash-resolver` crate or inside `ash-engine`?

**Evidence:**
- ash-engine has 21 source files, 7376 total lines
- module_loader.rs is the largest at ~1350 lines
- The engine already owns: parsing, type-checking entry, execution, module loading, provider registration, LLM/MCP providers
- RuntimeState (in ash-interp) is the provider/callable registry, not ash-engine

**Resolution: CONFIRMED — keep in ash-engine.**

Rationale: 21 files / 7K lines is moderate. The resolver is tightly coupled to the engine's parse→check→execute pipeline and shares types with module_loader.rs. A separate crate would need to re-export engine types or create circular deps. The resolver adds ~3-4 new files, bringing engine to ~25 files — still manageable.

Caveat: If the resolver grows past ~2000 lines, reconsider extraction. Add a TODO comment.

---

## D2: Builtin Dispatch Strategy — REVISED

**Question:** Extend eval.rs match arms, HashMap registry, or per-module BuiltinModule trait?

**Original recommendation:** Option C (per-module BuiltinModule trait).

**Evidence discovered:**
- eval.rs is 3322 lines
- A `builtin_dispatch_table()` function ALREADY EXISTS (lines 32-286) — it's a `HashMap<&'static str, BuiltinEntry>` with arity/variadic/implemented metadata
- A `dispatch_builtin()` function (lines 321-351) already does table lookup → arity check → delegate to `eval_function_call()`
- `eval_function_call()` then does a giant match on `(module, func_name)` tuples for the actual implementation
- The dispatch table has entries for ~35 builtins, but eval_function_call has 61 match patterns
- BuiltinEntry has `implemented: bool` — unimplemented builtins already return `EvalError::UnimplementedBuiltin`

**Resolution: REVISED — extend the existing dispatch table pattern, not a new trait.**

The infrastructure already exists. The right move is:
1. Add new builtins to `builtin_dispatch_table()` with `implemented: true`
2. Add new match arms to `eval_function_call()` for each module
3. Organize the match arms by module with clear section headers
4. When eval.rs exceeds ~4000 lines, extract per-module handler functions into `crates/ash-interp/src/builtins/` as plain functions (not a trait), called from the match arms

This is less disruptive than introducing a new trait/registry system when one already exists. The per-module extraction can happen later as a pure refactor.

**Adjustment to Track B tasks:**
- TASK-661 becomes: "Extend builtin_dispatch_table with IO/runtime/LLM entries + add match arms + organize into sections"
- Instead of creating 9 new files with trait implementations, add ~100 lines of match arms + table entries
- Keep the `builtins/` directory extraction as an optional cleanup task (lower priority)

---

## D3: Capability Constraint Model — RESOLVED

**Question:** Per-provider constraint structs or universal schema?

**Evidence:**
- `Constraint` in ash-core/ast.rs is `struct Constraint { predicate: Predicate }` — a single predicate wrapper
- FsProvider uses `FsConfig { allowed_paths, read_only, base_dir }` — a separate config struct, NOT derived from `Constraint`
- The provider trait takes `&[Constraint]` in `observe()` but providers interpret them internally
- ConstraintEnforcer in ash-interp has its own violation enum
- The pattern: each provider has its own XxxConfig struct + interprets Constraint values according to its domain

**Resolution: CONFIRMED — per-provider config structs.**

Each new provider gets:
- `HttpConfig { allowed_methods, allowed_hosts, max_body_size, timeout }`
- `TimeConfig { timezone }`
- `ProcessConfig { allowed_commands, timeout, max_output_bytes }`

These are construction-time configuration, separate from the runtime `Constraint` predicate system. Providers may also respect `Constraint` values in `observe()` calls.

---

## D4: Stdlib Loading Scope — RESOLVED

**Question:** Auto-load all stdlib, entry-only, or on-demand?

**Evidence:**
- Stdlib: 35 .ash files, 2128 total lines, ~65KB total
- Current entry path loads only 4 modules (result, runtime, runtime::error, runtime::args)
- Parsing 35 files at startup is trivially fast (<50ms estimate)
- But: auto-loading means all 35 modules' types enter the namespace, causing potential collisions
- The graph resolver (Track A) already needs to resolve `use` statements to file paths

**Resolution: CONFIRMED — on-demand (resolve-and-load).**

The resolver loads only modules referenced by `use` statements. Given the tiny stdlib size (65KB), there's no performance concern. But namespace hygiene matters — users shouldn't see types from modules they didn't import.

Implementation detail: the resolver maintains a "stdlib root" path. When a `use std::json` is encountered, it resolves to `std/src/json.ash`, parses it, registers its type definitions and builtin declarations. Transitively resolved imports within the stdlib module are also loaded.

**Caveat:** The prelude module (if any) should be auto-loaded. Currently `prelude.ash` exists but is minimal. This can be a follow-up task.
