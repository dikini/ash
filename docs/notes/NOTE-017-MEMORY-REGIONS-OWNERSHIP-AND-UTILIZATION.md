# NOTE-017: Memory Regions, Ownership, and Utilization

**Date:** 2026-06-24
**Status:** Living document — exploration in progress
**Purpose:** Define the target Ash memory story beyond "Rust handles it": per-process
memory regions as the initial semantic model, ownership transfer across communication
boundaries, and future region/reuse optimization inspired by Koka's Perceus. Companion to
NOTE-016 (runtime organization) and `process-model.md`.

## 0. Motivation

Current Ash is implemented in Rust, so concrete memory safety is inherited from Rust data
structures, ownership, reference counting, and runtime discipline. That is a good
implementation baseline, but it is not a satisfying Ash language story.

The target Ash runtime needs to explain:

1. who owns values;
2. when memory is released;
3. what can cross process/app/workflow boundaries;
4. how long-running processes avoid unbounded retention;
5. how streams and graphs avoid accidental buffering;
6. how the compiler/runtime can optimize memory without changing semantics.

The initial target story:

```text
each process owns a memory region;
values allocated by the process live in that region unless moved out;
process termination releases the region;
communication moves or copies values across region boundaries according to type/effect rules;
long-lived processes must manage state explicitly;
future analyses may refine regions and reuse storage within a process.
```

## 1. Current Baseline: Rust Safety, Ash Ambiguity

The current implementation gets memory safety from Rust. That means:

- Rust prevents use-after-free in the implementation;
- Rust APIs decide when values are cloned, reference counted, or dropped;
- async tasks, interpreter values, traces, closures, and runtime registries retain memory
  according to their Rust representation;
- Ash source code does not yet have a precise memory model of its own.

This is acceptable for implementation safety. It is not enough for target Ash semantics,
because the user needs to know whether an Ash process is isolated, whether a sent value is
copied or moved, whether long-running services retain old context, and whether graph/stream
execution is bounded.

## 2. Initial Target: Per-Process Regions

The first semantic model should be process-region memory:

```text
ProcessRegion {
  owner: ProcessId
  allocations: values created by this process
  state: live until process termination or explicit subregion reset
}
```

Lifecycle:

```text
spawn process      -> allocate fresh region
run process        -> allocate into region
send value         -> transfer/copy according to type boundary
receive value      -> value becomes owned by receiver region
terminate process  -> release region
```

This matches the process model:

- processes are isolated;
- no shared mutable memory across processes by default;
- message passing is the communication path;
- process death is a memory cleanup boundary.

## 3. Ownership and Communication

### 3.1 Values are process-owned

By default, a value belongs to the current process region.

```ash
let payload = build_payload();
```

`payload` is owned by the current process.

### 3.2 Sending transfers or copies explicitly

A channel send crosses a region boundary.

```ash
channel.send(worker, payload);
```

The semantic options must be explicit:

| Mode | Meaning |
|---|---|
| move | sender loses access; receiver owns value |
| copy | value is duplicated; requires copyable/shareable type |
| share | value is shared read-only or through a controlled resource handle |
| reject | type cannot cross the boundary |

Default preference:

```text
send by move for owned values;
copy only for explicitly copyable values;
share only through explicit shared/resource types;
reject process-local values.
```

This avoids transparent distributed/shared memory.

### 3.3 Process-local values cannot escape

Some values are tied to a process region:

```text
ProcessLocal<T>
```

Examples:

- raw handles to process-local resources;
- borrowed views into process-owned buffers;
- non-serializable host objects;
- continuations that capture process-local state;
- graph interpreter internal state.

These values must not be sent to another process or app unless wrapped in an explicit
capability/resource handle with well-defined semantics.

## 4. App and Runtime Boundaries

NOTE-016 allows one `RuntimeKernel` to host multiple app instances. Memory isolation should
follow the same layering:

```text
RuntimeKernel
  AppRegion billing
    ProcessRegion p1
    ProcessRegion p2
  AppRegion agents
    ProcessRegion p3
```

An app region is not necessarily one allocator arena. It is a semantic boundary for:

- app-local registries;
- app-local process regions;
- app-local graph instances;
- report/trace retention;
- admitted resource handles.

Default rule:

```text
one app cannot retain or inspect another app's process memory except through explicit
inter-app communication or shared provider/resource handles.
```

## 5. Long-Lived Processes and State Discipline

Per-process region release is simple for short-lived processes. Long-lived services need
more discipline.

A server loop can accidentally retain:

- old request values;
- captured environments;
- trace buffers;
- pending mailbox messages;
- memo thunks;
- graph node histories;
- closure dictionaries;
- unbounded accumulators.

Target Ash should distinguish:

```text
process region       -- lifetime of process
iteration subregion  -- lifetime of one loop turn / message handling step
state cell/resource   -- intentionally retained between turns
trace/report sink     -- separately bounded retention policy
```

A service loop should ideally allocate request-local data in an iteration subregion that is
released after the message is handled, while retaining only explicit state.

```text
receive message
  allocate iteration region
  decode/process/respond
  update explicit state
  release iteration region
next message
```

This gives Ash a memory story for servers and agents:

```text
agent memory is explicit state/resource;
conversation transcript retention is a policy;
temporary tool/model call data is iteration-local;
old context is not retained accidentally by the process loop.
```

## 6. Streams, Graphs, and Bounded Memory

### 6.1 Pull streams

Pull streams should naturally support bounded memory:

```text
next : Producer<A> -> {effects} Step<A, Producer<A>>
```

Each step can allocate in a step subregion. The next producer state is the only retained
codata. Accidental accumulation should be visible as explicit state growth.

### 6.2 Push streams

Push mode needs explicit buffering policy:

```text
bounded buffer
drop newest
drop oldest
block/backpressure
spill to resource
fail
```

Without this, push streams become hidden memory growth.

### 6.3 FRP graphs

Graph instances need declared retention:

- current value only;
- bounded history;
- windowed state;
- memoized node cache;
- replay log;
- external resource sink.

A graph blueprint should not imply unbounded history unless it declares it.

## 7. Rust Implementation vs Ash Semantics

Rust remains the implementation safety substrate. The target Ash memory model should sit
above it.

| Layer | Responsibility |
|---|---|
| Rust | concrete memory safety, allocation APIs, async runtime correctness |
| Ash runtime | process/app regions, message ownership, cleanup, retention policy |
| Ash type checker | Send/Share/ProcessLocal/resource boundaries, row effects |
| Ash optimizer | reuse, in-place update, region splitting, elision |
| Ash user model | values move/copy/share according to explicit rules |

Implementation may use:

- Rust ownership and moves;
- arenas/bump allocators;
- reference counting for internal sharing;
- copy-on-write;
- object pools;
- slab allocators;
- tracing buffers with bounded retention.

But these are implementation strategies. The visible story should be process/app/region
ownership, not "whatever the Rust implementation happens to do."

## 8. Future Direction: Region Discipline and Perceus-Like Reuse

Koka's Perceus demonstrates that precise reference counting and reuse analysis can support
functional programming with efficient in-place updates. Ash can take inspiration without
committing to the same mechanism immediately.

Possible future refinements:

1. **Uniqueness detection:** if a value has one owner, update/reuse in place.
2. **Drop specialization:** generate exact drops for known data shapes.
3. **Borrowed views:** allow non-owning views within a region, forbidden across process
   boundaries unless proven safe.
4. **Subregion inference:** infer temporary regions for loops, handlers, stream steps, and
   graph ticks.
5. **Region polymorphism:** functions abstract over caller-provided regions.
6. **Tail-call/loop reuse:** reuse frame/continuation storage when affine/linear facts allow.
7. **Handler-region discipline:** captured continuations carry clear region ownership and
   multiplicity constraints.
8. **Persistent data optimization:** share immutable structure with explicit copy-on-write or
   reference counting only when needed.

The key point: per-process region release is the initial coarse model. Perceus-like reuse is
the later fine-grained optimization story.

## 9. Interaction with Effects and Handlers

Handlers complicate memory because they capture continuations and may resume, discard, or
duplicate them.

Target rules should align with existing continuation multiplicity work:

| Handler behavior | Memory consequence |
|---|---|
| affine resume | captured continuation consumed once; region ownership can move linearly |
| discarded resume | captured continuation region can be released if no other references exist |
| multi-shot pure resume | captured continuation may be reused; requires pure/empty row and shareable captured state |
| delayed resume | captured continuation outlives current stack/iteration region; must be region-safe |

This is why continuation multiplicity is not only a control-flow property. It is also a
memory-retention property.

## 10. Interaction with Contracts and Evidence

Contracts can help memory discipline:

- `ProcessLocal<T>` cannot escape current process;
- channel payload type must be sendable;
- graph node history is bounded by contract;
- stream producer is productive;
- service state size is bounded or explicitly unbounded;
- resource handle lifetime is tied to a scope/process/app.

Evidence can justify optimizations:

- a function does not retain its input;
- a handler resumes at most once;
- a producer emits bounded-size chunks;
- a graph node keeps only a fixed-size window.

These are future optimization hooks, not first-slice requirements.

## 11. Failure, Restart, and Cleanup

Supervision and memory should compose:

```text
child process fails
  -> supervisor observes failure
  -> child process region released
  -> restart creates fresh region
  -> supervisor may preserve selected state separately if policy says so
```

Restart should not accidentally keep the failed child's region alive through diagnostics,
closures, or traces. Reports should copy or summarize needed evidence under a bounded
retention policy.

## 12. Resolved Direction

1. Rust memory safety is necessary but not a sufficient Ash language story.
2. The initial target memory model is per-process regions.
3. Process termination releases the process region.
4. App instances are memory/authority/reporting boundaries above processes.
5. Communication moves/copies/shares values explicitly.
6. Process-local values cannot escape without explicit resource/capability wrapping.
7. Long-lived processes need iteration subregions and explicit retained state.
8. Push streams and FRP graphs need explicit buffering/retention policies.
9. Future optimization can use Perceus-like uniqueness/reuse and region inference.
10. Continuation multiplicity affects memory retention and region safety.

## 13. To Be Resolved

### 13.1 Surface type classes or traits

Need final names and semantics for:

```text
Send
Copy
Share
ProcessLocal
RegionLocal
Serialize
ResourceHandle
```

These may be interfaces, marker traits, effects, contracts, or compiler-known predicates.

### 13.2 Channel payload rules

Open questions:

- Which types can be sent by move?
- Which types are copied?
- Which types are rejected?
- How do affine process handles interact with channel sends?
- Can closures cross process boundaries?
- Can continuations cross process boundaries?

### 13.3 Region syntax

The first slice should avoid user-visible region syntax if possible. Later work may need:

- explicit region annotations;
- process-local declarations;
- scoped temporary regions;
- resource-lifetime parameters.

### 13.4 Long-lived state analysis

Need diagnostics for:

- unbounded mailbox growth;
- unbounded list accumulation;
- retained closure environments;
- unbounded graph history;
- memo caches without eviction;
- trace/report retention without policy.

### 13.5 Runtime allocator strategy

Implementation choices remain open:

- one arena per process;
- arena pools per scheduler thread;
- app-level pools;
- slab allocation by value shape;
- Rust allocator plus logical region tracking first;
- hybrid reference counting for immutable shared data.

## 14. Migration Implications

Current docs and implementation should stop implying that Rust memory handling alone is the
Ash memory model.

Migration path:

1. Document process-region semantics as the target user model.
2. Mark current Rust allocation behavior as implementation detail.
3. Add sendability/process-local vocabulary to type/effect docs.
4. Add runtime region cleanup events to traces.
5. Add diagnostics for unbounded retention patterns.
6. Later: implement region/subregion allocation or logical region tracking.

## 15. Working Principle

The memory rule:

```text
Processes own regions. Communication crosses regions explicitly. Termination releases
regions. Long-lived state is explicit. Sharing is a capability/resource decision, not an
accident of implementation.
```

This gives Ash a memory story that fits its process, app, supervision, stream, and handler
model while leaving room for Rust-backed implementation and future Perceus-like optimization.

## 16. References

Internal references:

- [NOTE-016: Runtime Organization, Behaviours, and Reactive Modes](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [Ash Process Model](../design/process-model.md)
- [SPEC-049: Process Runtime Semantics](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-070: Runtime Kernel and OS-Facing Execution Surface](../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [SPEC-101: Lazy and Memo Computation Modes](../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
- [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)
- [effectful-stream-sinks](../design/effectful-stream-sinks.md)

External reference:

- Koka Perceus: precise reference counting and reuse analysis for functional programs.

## 17. Changelog

- 2026-06-24: Initial synthesis note. Defines per-process memory regions as the target
  memory model, separates Ash memory semantics from Rust implementation safety, and records
  future region/reuse optimization directions inspired by Perceus.
