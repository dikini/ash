---
status: drafting
created: 2026-05-12
last-revised: 2026-05-12
related-plan-tasks: []
tags:
  - future
  - compiler
  - ir
  - bytecode
  - jit
  - self-hosting
---

# FUTURE-005: Compiled Execution Substrate

## Summary

This exploration captures a future Ash engine/compiler direction: mature the IR pipeline beyond the current interpreter-oriented substrate, then add a bytecode compiler for faster program/library loading, and later add an optional JIT backend for hot execution. The primary payoff is not only performance. The work would force clearer compiler stages, stable execution artifacts, verifier boundaries, traceability metadata, and a more mature relationship between source semantics, typed IR, lowered machine IR, bytecode, and runtime execution.

The agreed implementation order is:

1. Typed Canonical IR (TCIR)
2. Ash Machine IR (AMIR)
3. Ash Bytecode
4. JIT backend
5. Post-v1 Ash-in-Ash/self-hosting exploration

The design order must account for downstream constraints earlier than implementation. JIT should influence AMIR and bytecode shape; bytecode should influence AMIR; AMIR should influence TCIR lowering contracts; self-hosting friendliness should inform artifact/schema decisions without blocking v1 or any bytecode MVP.

This note is exploratory and non-normative. It is intended to guide future design work and later spec/plan packets, not to start implementation now.

## Problem Statement

Ash currently has language semantics and runtime execution paths that are still closer to interpreted/compiler-internal forms than to a mature staged compilation architecture. Future performance and architecture goals suggest introducing explicit intermediate layers and durable artifacts:

- a typed/canonical semantic IR;
- a lowered abstract-machine IR;
- a bytecode artifact suitable for loading, verification, caching, and VM execution;
- eventually an optional JIT backend that lowers selected regions to native code.

The motivating questions are:

1. What kind of IR pipeline should Ash grow toward?
2. How should bytecode be designed so it is useful for fast loading now but does not make JIT awkward later?
3. What traceability and metadata guarantees are needed so lowering remains explainable, verifiable, and debuggable?
4. How should post-v1 self-hosting remain visible as a design pressure without becoming premature scope?

## Scope

In scope:

- future IR/bytecode/JIT architecture discussion;
- design decisions made in the originating discussion;
- constraints for later TCIR, AMIR, bytecode, verifier, artifact, and JIT specs;
- traceability, metadata, stripping, and artifact-mode distinctions;
- non-blocking self-hosting friendliness as a long-term maturity force.

Out of scope:

- immediate implementation;
- concrete bytecode instruction definitions;
- a final binary encoding choice;
- a complete runtime ABI;
- a full JIT backend selection or integration plan;
- Ash-in-Ash implementation planning;
- changing Ash language semantics.

Related but separate:

- existing small-step semantics and runtime carrier work;
- existing canonical type-expression IR / normalizer work;
- future implementation conformance specifications;
- future performance benchmarking strategy;
- future compiler pass architecture and optimization pipeline.

## Goals

### Direct goals

1. Mature Ash's IR architecture and compiler staging.
2. Speed program and library loading through bytecode artifacts.
3. Create a future-compatible path for optional JIT acceleration.

### Secondary maturity goals

The typed IR, bytecode compiler, and JIT are also maturity forcing functions. They require clearer answers to questions that can otherwise remain implicit in an interpreter:

- What is the canonical semantic representation?
- What is the abstract execution representation?
- What information survives lowering?
- What metadata is required for safety?
- What metadata is required for diagnostics and traceability?
- What is the runtime ABI boundary?
- What does it mean for compiled and interpreted Ash to be equivalent?

### Indirect post-v1 goal

A possible Ash-in-Ash/self-hosting path becomes more plausible if Ash has stable compiler targets and artifacts. TCIR, AMIR, and bytecode would provide stepping stones for future Ash-written compiler components.

Self-hosting is not a current plan. It is a non-blocking design pressure.

## Current Understanding

### Settled in this exploration

- The implementation order is IR first, then bytecode, then JIT.
- The design process must consider JIT and self-hosting constraints before implementation reaches those stages.
- A distinct Ash Machine IR (AMIR) layer should exist between Typed Canonical IR and bytecode.
- AMIR should be an abstract-machine IR, not source Ash and not native machine code.
- Bytecode should encode AMIR, not duplicate source-language semantics.
- Bytecode should be JIT-compatible but not JIT-dependent.
- A block/register-oriented AMIR and bytecode shape is preferred over a classic tiny stack VM.
- Rich source-level types may be erased in lower layers, but traceability and safety facts must not be accidentally erased.
- Traceability is the core debug/development constraint: a bytecode range should be explainable through AMIR, TCIR, and source.
- Optimized production artifacts may strip explanation metadata, but not safety-critical metadata.
- The bytecode verifier should depend only on bytecode plus required safety metadata, not on full trace/debug metadata.
- Bytecode should be a stable, sectioned external artifact format with a documented logical schema, not merely serialized Rust structs.
- AMIR text should be a semi-stable developer/debug format, preferably parseable/loadable, but not the stable production execution artifact and not a second source language.
- Self-hosting friendliness should inform choices but must not block v1, TCIR, AMIR, or bytecode MVPs.

### Still uncertain

- Exact TCIR shape and relation to current Ash AST/typechecker outputs.
- Exact AMIR instruction/block/register model.
- Exact bytecode instruction set and physical encoding.
- Exact verifier fact model.
- Exact runtime ABI boundary.
- Whether the first JIT backend should be Cranelift, LLVM, or another backend.
- How much of Act/Proc/Workflow should ever be JIT-lowered directly.
- How much source-level type information should be retained in AMIR/bytecode metadata by default.
- How AMIR textual form should be versioned and loaded.

## Layer Model

### Surface AST

Surface AST is parsed source syntax. It preserves source structure, spans, and syntactic forms.

Surface AST is not the right artifact for bytecode execution or JIT input. It is too close to syntax and too far from executable control/effect structure.

### Typed Canonical IR (TCIR)

Typed Canonical IR is the semantic authority.

Use the expanded form on introduction. In lighter prose, "Typed IR" is acceptable. Use "Typed Canonical IR (TCIR)" when emphasizing canonicality, stable compiler boundaries, or the distinction from an AST with type annotations.

TCIR concerns:

- rich Ash source-level types;
- resolved names, modules, and imports;
- typechecker output;
- tower/effect facts;
- capability requirements;
- source-level semantics;
- source anchors and diagnostics;
- semantic-preservation obligations for later lowering.

TCIR should explain what an Ash program means. It should not be constrained to look like machine code. It should preserve the information needed to prove or test that later execution artifacts preserve Ash semantics.

### Ash Machine IR (AMIR)

Ash Machine IR is the abstract-machine authority.

AMIR is C---like in spirit: it should expose lower-level machine/runtime concerns directly instead of pretending to still be source-level Ash. However, AMIR is not a user language and not hardware machine code. It is an IR-shaped representation of the Ash abstract machine.

AMIR concerns:

- functions as blocks;
- explicit basic blocks and control flow;
- registers, locals, and temporaries;
- normalized evaluation order;
- explicit call, return, fail, yield, and branch operations;
- explicit effect/tower boundaries;
- explicit capability boundary operations;
- lowered pattern matching;
- lowered `do`/bind sequencing;
- runtime or layout categories rather than full source-level types;
- metadata links back to TCIR and source.

AMIR should explain how the Ash abstract machine runs a program.

AMIR is the right layer for verifier-oriented execution structure and for later bytecode/JIT lowering. It should be block/register oriented rather than a stack-machine reconstruction target.

### AMIR textual form

AMIR should have a textual/debug representation.

Status:

- semi-stable developer/debug format;
- human-readable;
- diffable;
- suitable for golden tests;
- preferably parseable/loadable;
- useful for verifier and VM experiments;
- not the stable production execution artifact;
- not a second source language.

Possible uses:

- inspecting TCIR-to-AMIR lowering;
- writing small hand-authored AMIR verifier tests;
- debugging lowering and optimizer passes;
- minimizing bug reports;
- creating readable golden tests;
- experimenting with VM/verifier behavior before final bytecode encoding.

AMIR text may be accepted by dev/debug tooling, but production module loading should target Ash source or Ash Bytecode.

### Ash Bytecode

Ash Bytecode is the durable, sectioned, verifiable execution artifact encoding AMIR.

Bytecode concerns:

- deterministic loading and linking;
- faster library/program startup;
- compact execution representation;
- bytecode verifier input;
- VM/interpreter execution;
- optional JIT input;
- stable external artifact schema;
- required safety metadata;
- optional trace/debug/provenance metadata.

Ash Bytecode should store AMIR-derived executable semantics. It should not independently reinterpret source-level Ash semantics.

Good:

- TCIR defines meaning.
- AMIR lowers meaning into explicit abstract-machine execution.
- Bytecode serializes/verifies/loads AMIR.

Bad:

- Bytecode duplicates high-level language semantics and becomes a second semantic implementation.

### JIT backend

The JIT backend is an optional native lowering path for selected AMIR/bytecode regions.

It should not be the primary design center of the early compiled substrate. The bytecode and VM path must remain useful without a JIT.

Likely JIT v1 scope:

- pure or mostly-pure functions;
- hot inner kernels;
- runtime-helper calls for complex operations;
- no direct Workflow JIT;
- capability calls remain runtime-mediated;
- Act/Proc boundaries are call-outs or runtime-mediated boundaries, not initially inlined native effect machinery.

Possible later JIT scope:

- pure stdlib inlining;
- effect-local specialization where safe;
- profile-guided specialization;
- selected Proc internals, only if the runtime ABI and suspension model are mature enough.

### Ash-in-Ash / self-hosting

Ash-in-Ash is a post-v1 speculative horizon.

Status:

- not a current plan;
- not a pre-v1 milestone;
- not a blocker for TCIR, AMIR, bytecode, or JIT;
- a non-blocking design pressure.

Likely split:

Ash-in-Ash candidates:

- frontend;
- AST/IR transforms;
- diagnostics;
- static analysis;
- optimization passes;
- bytecode emission.

Rust likely remains:

- stage0 bootstrap executable;
- bytecode VM/runtime;
- capability provider host;
- OS/process integration;
- verifier/reference implementation;
- possibly JIT host.

The rule is:

> Prefer IR and bytecode designs that would not block self-hosting later. Do not design the whole compiler now around self-hosting.

## Design Decisions

| ID | Decision | Status | Rationale |
|----|----------|--------|-----------|
| D1 | Treat bytecode and JIT as part of one compiled execution substrate. | Settled | Both require maturing IR stages and share lowering/runtime/metadata concerns. |
| D2 | Implementation order is TCIR -> AMIR -> bytecode -> JIT. | Settled | JIT depends on bytecode/executable IR maturity; bytecode depends on a lowered execution representation. |
| D3 | Design order must consider downstream dependencies early. | Settled | A bytecode designed without JIT awareness may become a dead end; TCIR designed without lowering awareness may omit needed facts. |
| D4 | Add a distinct AMIR layer between TCIR and bytecode. | Settled | Compiling TCIR directly to bytecode would conflate semantic authority with abstract-machine execution. |
| D5 | Prefer block/register AMIR and bytecode over a classic stack VM shape. | Working decision | Block/register form maps more directly to verifier facts, control flow, JIT backends, source maps, and effect/capability boundaries. |
| D6 | Bytecode should encode AMIR, not source semantics. | Settled | Keeps bytecode as an execution artifact rather than a second language semantics. |
| D7 | Bytecode should be JIT-compatible but not JIT-dependent. | Settled | Bytecode must deliver load-time and VM value even if JIT never ships. |
| D8 | Traceability is the primary debug/development metadata constraint. | Settled | Type metadata is one use case; the broader need is explaining why each bytecode/AMIR region exists. |
| D9 | Lower layers may erase rich types if required safety facts and debug traceability are preserved appropriately. | Settled | Different layers have different type concerns: rich types, runtime categories, machine/layout types. |
| D10 | Optimized production artifacts may strip explanatory traceability metadata. | Settled | Optimization must not be blocked by debug metadata. |
| D11 | Stripping must not remove safety-critical metadata. | Settled | Verifier, linker, capability checks, effect boundaries, ABI checks, and runtime safety still need required sections. |
| D12 | The verifier should not depend on full debug traceability metadata. | Settled | Stripped production artifacts should remain verifiable. |
| D13 | Bytecode should be sectioned from the beginning. | Working decision | Sectioning cleanly separates required safety/execution metadata from optional trace/debug/provenance data. |
| D14 | Bytecode should be a stable external artifact schema, not just serialized Rust structs. | Settled | Enables compatibility, tooling, and future Ash-written emitters. |
| D15 | AMIR text should be a semi-stable developer/debug/loadable format. | Settled | It helps tests and debugging without becoming the production artifact or a source language. |
| D16 | Self-hosting friendliness is non-blocking design pressure. | Settled | It should inform choices without becoming premature scope. |

## Design Dimensions

| Dimension | Preferred direction | Rejected or lower-priority direction | Notes |
|-----------|---------------------|--------------------------------------|-------|
| Execution representation | Distinct AMIR layer | Direct TCIR-to-bytecode only | AMIR isolates abstract-machine concerns. |
| Bytecode shape | Block/register | Tiny stack VM first | Stack VM may be simpler but risks JIT impedance mismatch. |
| Bytecode role | AMIR encoding | Source semantic interpreter | Avoid duplicate semantics. |
| JIT dependency | JIT-compatible | JIT-dependent | Bytecode must stand alone. |
| Type information | Layer-specific type concerns | Rich source types everywhere | Erasure is allowed when safety and traceability constraints are met. |
| Metadata priority | Traceability + safety split | Debug info entangled with execution | Enables stripping and production optimization. |
| Verifier input | Required safety metadata only | Full debug provenance | Verifier must work for stripped artifacts. |
| Artifact format | Stable logical schema | Rust struct serialization | Rust structs can implement the schema, not define it. |
| AMIR text | Semi-stable dev/debug/loadable | Production artifact or second source language | Useful for tests and inspection; not a user language. |
| Self-hosting | Non-blocking future pressure | Near-term planning driver | Keep the door open without scope creep. |

## Type Erasure and Layer-Specific Type Concerns

Different IR layers have different type concerns.

TCIR uses rich source-level Ash types:

- `List<Int>`;
- `Result<T, E>`;
- `Act<A>`;
- `Proc<A>`;
- `Workflow<A>`;
- capability and effect facts tied to source semantics.

AMIR uses lowered/runtime categories and verifier facts:

- `Value`;
- `I64`;
- `Bool`;
- `SymbolId`;
- `FunctionRef`;
- `CapabilityRef`;
- `FrameRef`;
- register/local initialization and layout facts.

Bytecode uses compact execution encoding plus required and optional metadata:

- instruction operands;
- constant-pool references;
- function IDs;
- local/register categories;
- import/export signatures;
- effect/capability tables;
- verifier facts;
- optional rich debug/type metadata.

A JIT/backend uses machine/layout/ABI types:

- `i64`;
- `i1`;
- pointers;
- tagged pointers;
- runtime handles;
- backend-specific IR types.

The important rule is not "preserve full source types everywhere". The rule is:

> Lower layers may erase rich types. They must not erase required safety facts, and debug/development artifacts must preserve enough traceability to recover why the lower representation exists.

## Traceability Constraint

Traceability is the core cross-layer constraint.

Every meaningful AMIR or bytecode instruction/region in debug/development artifacts should be explainable:

- why it exists;
- which lowering pass or rule produced it;
- which TCIR node/fact it came from;
- which source span/construct it corresponds to;
- which semantic obligation it preserves.

Possible trace chain:

```text
source span
  -> Surface AST node id
  -> TCIR node id
  -> lowering event/rule id
  -> AMIR block/instruction ids
  -> bytecode offset/range
  -> optional JIT/native code range
```

Example trace:

```ash
x <- read_file(path)
```

Potential TCIR facts:

- do-bind in `Act`;
- capability requirement: `file.read`;
- failure behavior through the Act boundary;
- source span for the bind.

Potential AMIR lowering:

- `call_capability file.read`;
- branch on success/failure result;
- bind successful value to a local/register;
- jump to continuation block;
- jump or return through failure continuation.

Potential bytecode metadata:

- bytecode offsets for the capability call and branches;
- source span;
- TCIR origin node;
- AMIR origin block/instruction IDs;
- lowering rule such as `lower_act_bind_capability_call`;
- effect `Act`;
- capability `file.read`;
- failure continuation block.

This tells a debugger, verifier, auditor, or future Ash-written tool why the bytecode exists in that position.

Every destructive lowering pass should emit enough provenance to explain the destruction in debug/development artifacts.

Examples:

- if pattern matching becomes branches, record that branch group came from a match;
- if `do` bind becomes continuation blocks, record the bind origin;
- if rich type becomes `Value`, record source signature/type facts where needed;
- if contract forms become checks/obligations, record the contract origin;
- if workflow/proc structure becomes state-machine pieces, record the projection origin.

## Metadata Categories

Traceability-first metadata categories:

1. Source mapping
   - AMIR/bytecode ranges to source spans.

2. IR origin mapping
   - AMIR/bytecode ranges to TCIR nodes or regions.

3. Lowering provenance
   - pass/rule IDs explaining why code exists.

4. Semantic obligation mapping
   - effects, capabilities, failures, contracts, workflow/proc obligations.

5. Type/debug metadata
   - source-level types and names where needed for diagnostics, reflection, debugging, or tooling.

6. ABI/layout metadata
   - runtime categories, calling convention, local/register layout.

7. Verification facts
   - facts the verifier checks or consumes: initialization, control-flow validity, capability boundary validity, effect boundary validity, import/export signatures.

## Artifact Modes

### Debug bytecode

Debug bytecode should include full traceability:

- source maps;
- TCIR origin maps;
- AMIR origin maps;
- lowering provenance;
- rich type/debug metadata;
- semantic obligation metadata;
- optimization remarks if optimizations ran.

### Release bytecode

Release bytecode should include minimum required verification/linking/runtime metadata:

- module ABI;
- import/export signatures;
- runtime layout categories;
- effect/capability requirements needed for safety;
- bytecode version;
- runtime ABI version;
- compiler version or compatibility marker;
- dependency hashes or equivalent cache invalidation data.

### Stripped bytecode

Stripped bytecode may remove explanatory metadata but must retain the safety-critical envelope required for loading, linking, verification, and execution.

Optimization may remove explanation metadata. It may not remove safety metadata.

Stripping should be an explicit artifact transformation, not accidental metadata loss.

Preferred artifact flow:

```text
source + compiler version + build config
  -> debug bytecode with full traceability
  -> optimized/release bytecode
  -> optionally stripped bytecode
```

Avoid:

```text
source -> opaque optimized blob
```

## Verifier Boundary

The bytecode verifier should operate on Ash Bytecode plus required safety metadata. It should not depend on full traceability/debug metadata.

Verifier should not require:

- source spans;
- original source syntax;
- TCIR node IDs;
- lowering rule IDs;
- rich explanatory debug types beyond required signatures/layout categories.

Verifier input should include:

- bytecode instruction stream;
- constant pool or symbol table;
- function/module signatures;
- import/export table;
- runtime layout categories;
- control-flow/block structure;
- register/local declarations;
- required effect/tower boundary markers;
- capability requirement table;
- ABI/runtime/compiler versions;
- verifier-required facts.

This creates two separable contracts:

1. Safety contract
   - Can this bytecode be loaded and executed safely and consistently?

2. Explanation contract
   - Can this bytecode be traced back through AMIR, TCIR, and source?

Debug builds should satisfy both. Stripped release builds must satisfy safety and may satisfy only partial explanation.

## Sectioned Bytecode Artifact

Ash Bytecode should be sectioned from the beginning.

Required sections may include:

- header/version;
- module identity/hash;
- imports/exports;
- function table;
- code;
- constants;
- layout/signature table;
- effect/capability table;
- verification table.

Optional sections may include:

- source map;
- TCIR origin map;
- AMIR origin map;
- lowering provenance;
- rich type/debug metadata;
- optimization remarks;
- JIT hints/profile data.

Required sections support execution, linking, and safety. Optional sections support explanation, tooling, debugging, auditing, optimization analysis, and future self-hosting.

Sectioning prevents two failure modes:

1. Debug/provenance data becomes entangled with execution.
2. Production stripping becomes unsafe ad hoc deletion.

## Stable External Schema

Ash Bytecode should be a stable external artifact format, not merely serialized Rust structs.

Rust structs may be the implementation representation. They should not be the compatibility contract.

The bytecode artifact should have a documented logical schema so that:

- the loader can reject incompatible artifacts using explicit version/ABI metadata;
- inspectors, disassemblers, debuggers, and verifiers can read artifacts without linking to internal compiler crates;
- a future Ash-written compiler can emit bytecode by targeting the schema;
- compatibility can evolve section-by-section rather than by Rust struct layout churn.

Physical encoding can be chosen later. Options may include:

- custom binary format;
- CBOR/MessagePack-like encoding;
- Cap'n Proto/FlatBuffers-like schema;
- postcard/bincode-like internal cache only;
- textual debug/disassembly format.

The first design step should define logical sections and invariants, not prematurely select the final binary encoding.

A useful distinction:

- compiler internal cache: may be unstable and compiler-version fragile;
- Ash Bytecode: documented artifact with explicit versioning and compatibility rules.

Anything called "Ash Bytecode" should be schema-defined.

## JIT Design Pressure

JIT is later in implementation order but should influence bytecode and AMIR design now.

Cranelift/LLVM-style backends generally prefer:

- explicit basic blocks;
- SSA/register-like values;
- typed or layout-classified operations;
- explicit control flow;
- explicit runtime-helper calls;
- explicit calling conventions.

Therefore, AMIR and bytecode should avoid being stack-machine-specific if JIT remains a serious future goal.

This does not mean bytecode should depend on a JIT. The correct target is:

> JIT-compatible bytecode, not JIT-dependent bytecode.

Bytecode remains useful without JIT:

- faster loading;
- module/library cache;
- VM execution;
- verifier target;
- stable artifact for tooling.

JIT later becomes an optimization path over selected AMIR/bytecode regions.

## Self-Hosting Friendliness

Ash-in-Ash/self-hosting is a long-term maturity goal, only after first release.

The v1 Rust Ash implementation should be understood as:

- stage0 bootstrap engine;
- executable for Ash programs;
- reference verifier/loader/runtime;
- compatibility oracle for later self-hosting experiments.

A possible bootstrapping ladder:

1. Rust compiler/interpreter/engine executes Ash programs.
2. Rust compiler emits stable bytecode.
3. Compiler-adjacent tools are written in Ash.
4. Ash frontend or IR transform pieces are written in Ash.
5. Ash compiler emits bytecode for ordinary modules.
6. Ash compiler can compile enough of itself to be self-hosting, with Rust stage0 as verifier/fallback.

Self-hosting friendliness suggests:

- keep IR serializable and inspectable;
- keep bytecode specified, not just represented by Rust structs;
- keep compiler passes explicit and composable;
- keep verifier independent from frontend assumptions;
- keep source maps and diagnostics first-class;
- avoid Rust-only semantic magic in compiler artifacts;
- separate host/runtime services from language-level compiler logic;
- make stdlib/module loading deterministic and cacheable.

But the guardrail remains:

> Self-hosting should inform design. It should not block near-term implementation or force premature compiler architecture.

## Semantic Guardrails

This track should not change Ash language semantics.

The compiled execution substrate must preserve observable behavior:

- interpreted execution and bytecode execution should be equivalent for supported programs;
- JIT execution should be equivalent to bytecode/VM execution for compiled regions;
- capability boundaries remain runtime-mediated;
- effect/tower boundaries remain explicit and verifiable;
- Workflow orchestration semantics should not be silently redefined by bytecode or JIT lowering.

Bytecode and JIT are engine/compiler evolutions, not new surface-language semantics.

## Expected Work Scope

This section is only an order-of-magnitude guide for future planning.

### TCIR maturity

Kind:

- compiler-core architecture;
- type/effect/capability representation;
- semantic preservation substrate.

Likely outputs:

- stable typed/canonical representation;
- resolved module/name/import model;
- explicit type/effect/capability facts;
- source-span/source-anchor preservation;
- lowering contract into AMIR;
- equivalence and diagnostic tests.

Risk: medium.

Value: high even without bytecode or JIT.

### AMIR

Kind:

- semantics + abstract-machine architecture;
- lowered execution representation.

Likely outputs:

- block/register representation;
- instruction/control/effect boundary model;
- verifier-friendly facts;
- AMIR textual/debug format;
- parser/loader for AMIR text, if feasible;
- TCIR-to-AMIR lowering;
- tests comparing TCIR/interpreter behavior with AMIR execution.

Risk: medium-high.

Value: very high; this is the main architecture maturity forcing function.

### Ash Bytecode

Kind:

- compiler backend + loader/cache/runtime artifact work.

Likely outputs:

- sectioned logical schema;
- physical encoding choice;
- bytecode instruction set;
- bytecode verifier;
- VM/dispatch engine;
- compiled-library cache;
- cache invalidation rules;
- module linking;
- bytecode disassembler/inspector;
- tests comparing interpreter/AMIR/bytecode behavior.

Risk: medium after AMIR exists.

Value: faster loading, stable executable artifacts, and stronger compiler architecture.

### JIT

Kind:

- backend/runtime ABI/performance work.

Likely outputs:

- backend integration;
- runtime helper ABI;
- native calling convention;
- region selection/hotness policy;
- fallback path to bytecode/VM;
- source/bytecode/JIT mapping;
- benchmarks.

Risk: high.

Value: speed for selected hot paths, especially pure or mostly-pure kernels.

### Ash-in-Ash

Kind:

- post-v1 compiler self-hosting exploration.

Likely outputs if pursued later:

- Ash-written compiler passes;
- Ash-written diagnostics/static analysis;
- Ash bytecode emitter;
- self-hosting bootstrap tests;
- stage0/stage1 equivalence checks.

Risk: high and intentionally deferred.

Value: major maturity signal and dogfooding path.

## Suggested Future Spec/Plan Starting Points

When this exploration is promoted, split the work into separate specs/plans rather than one huge phase.

Possible sequence:

1. TCIR audit and boundary spec
   - Map current parser/typechecker/core/runtime IR carriers.
   - Define what TCIR is and is not.
   - Define TCIR semantic authority and required facts.

2. AMIR design spec
   - Define abstract-machine model.
   - Define blocks/registers/control/effect boundary operations.
   - Define AMIR textual form and loadability level.
   - Define TCIR-to-AMIR traceability requirements.

3. Bytecode artifact schema spec
   - Define sectioned logical schema.
   - Define required vs optional sections.
   - Define versioning, ABI, imports/exports, signatures, capabilities, effects, and verifier facts.

4. Bytecode verifier spec
   - Define safety contract independent of debug metadata.
   - Define load/link/reject behavior.
   - Define verification facts.

5. Bytecode VM/cache implementation plan
   - Start with a narrow pure subset.
   - Add module/library artifact caching.
   - Add deterministic invalidation.
   - Expand coverage incrementally.

6. JIT feasibility spike
   - Choose candidate backend.
   - Compile pure-only AMIR/bytecode regions.
   - Measure and decide whether to continue.

7. Post-v1 self-hosting exploration
   - Only after TCIR/AMIR/bytecode are stable enough.
   - Treat Rust as stage0 verifier/runtime.

## Non-Goals for First Bytecode Work

A future bytecode MVP should probably not attempt to:

- JIT anything;
- directly compile Workflow orchestration to native code;
- solve every optimization question;
- define Ash-in-Ash;
- replace the current runtime all at once;
- make AMIR text a user-facing language;
- preserve full rich source types inline in every bytecode instruction;
- treat Rust struct serialization as the bytecode compatibility contract.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-12 | Frame bytecode and JIT as one compiled execution substrate. | Both depend on a more mature IR pipeline and shared runtime/metadata boundaries. |
| 2026-05-12 | Implementation order is TCIR -> AMIR -> bytecode -> JIT. | JIT is later and riskier; bytecode needs a lowered executable representation first. |
| 2026-05-12 | Design must account for downstream dependencies early. | Bytecode and IR choices can accidentally block JIT or self-hosting if designed in isolation. |
| 2026-05-12 | Add Ash Machine IR (AMIR) between Typed IR and bytecode. | TCIR should remain semantic; AMIR should expose abstract-machine execution. |
| 2026-05-12 | Prefer expanded names in prose, abbreviations in diagrams/tables. | Acronym-only prose harms readability. |
| 2026-05-12 | Use Typed Canonical IR (TCIR) when canonicality matters. | Distinguishes the semantic compiler boundary from an AST with type annotations. |
| 2026-05-12 | Prefer block/register bytecode shape. | Better fit for verifier facts, explicit control/effect boundaries, and later JIT backends. |
| 2026-05-12 | Lower layers may erase rich types but must preserve traceability and safety facts. | Different layers have different type concerns; full source typing everywhere is not required. |
| 2026-05-12 | Traceability is a core debug/development constraint. | Future tools must explain why a bytecode range exists and trace it back through IR layers to source. |
| 2026-05-12 | Production builds may strip explanatory traceability metadata. | Optimization should not be blocked by debug metadata. |
| 2026-05-12 | Verifier depends on required safety metadata, not full debug traceability. | Stripped production artifacts must remain verifiable. |
| 2026-05-12 | Bytecode should be sectioned from the beginning. | Required safety sections and optional trace/debug sections need clean separation. |
| 2026-05-12 | Bytecode should be a stable external schema, not serialized Rust structs. | Enables compatibility, tooling, and future Ash-written emitters. |
| 2026-05-12 | AMIR text should be semi-stable, debug-adjacent, and preferably loadable. | Supports tests, debugging, and verifier experiments without becoming production bytecode. |
| 2026-05-12 | Self-hosting friendliness is non-blocking design pressure. | It should inform choices without becoming premature scope. |

## Open Questions

1. What live Ash carrier should become or feed TCIR?
2. How much of current core IR can be reused for TCIR versus replaced or wrapped?
3. What is the minimal AMIR instruction set for a pure subset?
4. How should AMIR represent `Act`, `Proc`, and `Workflow` boundaries?
5. What verifier facts are intrinsic to AMIR versus bytecode sections?
6. How should traceability IDs be allocated and preserved across lowering passes?
7. What metadata belongs in required bytecode sections versus optional debug sections?
8. What physical encoding is appropriate for Ash Bytecode once the logical schema is known?
9. Should AMIR text be accepted by `ash run` in dev mode, or only by dedicated verifier/debug commands?
10. What is the first honest bytecode MVP subset?
11. What benchmark should justify bytecode work: stdlib load time, CLI startup, repeated library load, or all of these?
12. What first JIT backend should be evaluated, and what constraints does it impose on AMIR?
13. Which self-hosting-friendly properties should be checked by design review without becoming implementation blockers?

## Related Explorations

- [MCE-002: IR Core Forms Audit](../minimal-core/MCE-002-IR-AUDIT.md)
- [MCE-005: Small-Step Semantics](../minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Small-Step ↔ IR Execution](../minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [FUTURE-001: First-Class Workflows](FIRST-CLASS-WORKFLOWS.md)
- [FUTURE-002: AI-Native Workflows and Generated Ash Programs](AI-NATIVE-WORKFLOWS.md)

## Next Steps

- [ ] Audit live parser/typechecker/core/runtime representations before drafting a TCIR spec.
- [ ] Write a narrow TCIR boundary design note.
- [ ] Write an AMIR design sketch with block/register examples.
- [ ] Draft a bytecode logical-section schema independent of physical encoding.
- [ ] Define a verifier contract that does not depend on debug traceability.
- [ ] Define artifact modes: debug, release, stripped.
- [ ] Later, derive a staged implementation plan only after TCIR and AMIR design questions are clearer.
