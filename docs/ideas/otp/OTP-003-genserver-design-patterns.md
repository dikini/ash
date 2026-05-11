# OTP-003: GenServer-like Design Patterns for Ash

**Status:** Drafting
**Related:** [OTP-001](OTP-001-erlang-otp-analysis.md), [OTP-002](OTP-002-ash-otp-design.md), [SPEC-048 Proc Library](../../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049 Process Runtime Semantics](../../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050 Operational Bottom and Scoped Handling](../../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-052 Capability Interfaces and Implementations](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md), [SPEC-053 Runtime Resources and Authority Provenance](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md), [SPEC-054 Generalized Typed Do-Notation](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-056 First-Class Workflow Carrier](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md), [NOTE-011 Type-Level Protocols](../../notes/NOTE-011-TYPE-LEVEL-PROTOCOLS-CAPABILITY-AUTHORITY-AND-DISTRIBUTED-PARTICIPANTS.md)
**Last revised:** 2026-05-11
**Audience:** Ash language/runtime designers, stdlib authors, example authors, and future test-suite authors.

---

## 1. Summary

This note records several Ash-specific ways to express Erlang/OTP `gen_server`-like behavior: a long-running stateful process with a generic message loop and user-specific execution logic. The goal is not to choose one implementation yet. The goal is to preserve a family of comparable designs that can become examples, benchmarks, and differential/bisimulation test cases for Ash communicating processes and workflows.

The unifying idea is:

> GenServer-like behavior should be modeled as a reusable process/workflow pattern, not as one mandatory runtime primitive.

The likely Ash-native core is a `Proc`-level server loop with typed endpoints, optionally governed by `Workflow` contracts and optionally backed by capability/resource authority. Higher-level declarations or protocol descriptions should desugar to that core rather than creating independent runtime paths.

All examples in this note are **design sketches**, not guaranteed accepted surface syntax. They intentionally use Ash-shaped notation to make architectural differences visible. Before promotion to a spec or executable example corpus, each example must be rewritten against the live parser/typechecker surface.

---

## 2. Why this matters

The design space is useful for at least four reasons.

1. **Practical programming model.** Stateful, message-driven services are likely to appear often in real Ash programs: tool servers, model-session managers, queues, caches, routers, long-running agent loops, and supervised workers.
2. **Better process/workflow examples.** We need concrete examples that show why communicating processes/workflows are often better than one large program definition: smaller live state, clearer failure boundaries, less retained environment pressure, improved memory locality, and lower GC/retained-graph risk.
3. **Same idea, different implementation styles.** GenServer-like behavior is a compact way to show Ash's design flexibility: direct process loops, dictionaries, reducers, capabilities, workflow contracts, resources, and protocol descriptors can all express related behavior.
4. **Comparable tests.** The variants can become real test cases that should be behaviorally comparable. They can exercise different Ash features while supporting differential testing: direct loop vs reducer vs capability-backed handler vs workflow-governed server should agree on externally observable traces for the same scenario.

---

## 3. Shared scenario for examples

Most sketches below use a simple counter server.

Behavior:

- `Get` returns the current count.
- `Inc(n)` increments the count by `n` and acknowledges.
- `Reset` sets the count to zero and acknowledges.
- `Stop` terminates the server.

Illustrative message/result vocabulary:

```ash
pub type CounterCall =
  Get {}
| Inc { by: Int }
| Reset {}
| Stop {};

pub type CounterReply =
  Count { value: Int }
| Ack {};

pub type CounterState =
  CounterState { count: Int };
```

A useful test trace:

```text
start count=0
call Get       => Count(0)
call Inc(2)    => Ack
call Get       => Count(2)
cast Reset     => Ack-or-no-reply depending on variant
call Get       => Count(0)
call Stop      => Ack and server terminal
```

This trace can be reused across patterns.

---

## 4. Pattern A: Direct Erlang/OTP mirror

### Idea

Mirror Erlang's design as directly as possible: a generic `gen_server` library owns the receive loop and invokes user callbacks named `init`, `handle_call`, `handle_cast`, `handle_info`, and `terminate`.

### Sketch

```ash
pub type CounterCallbacks =
  CounterCallbacks {
    init: fn(Int) -> Act<CounterState>,
    handle_call: fn(CounterCall, From<CounterReply>, CounterState)
      -> Act<CallAction<CounterState, CounterReply>>,
    handle_cast: fn(CounterCall, CounterState)
      -> Act<CastAction<CounterState>>,
    terminate: fn(StopReason, CounterState) -> Act<Unit>,
  };

pub fn start_counter(initial: Int) -> Proc<ServerHandle<CounterCall, CounterReply>> {
  gen_server::start(counter_callbacks, initial)
}
```

### What this exercises

- OTP-style callback separation.
- Process identity and mailbox behavior.
- Call/cast distinction.
- Delayed reply and stop semantics.

### Benefits

- Familiar to Erlang users.
- Directly comparable with OTP references.
- Good teaching bridge from Erlang to Ash.

### Gaps

- Ash does not currently have Erlang's dynamic callback-module dispatch model.
- Exact function-value/record-of-functions ergonomics need live syntax verification.
- “Let it crash” needs translation into Ash operational failure and workflow failure boundaries.
- Direct mirroring may hide more Ash-native capability and contract structure.

---

## 5. Pattern B: Explicit callback dictionary

### Idea

Use manual dictionary passing. The generic library accepts a value containing the user-specific operations. This is the explicit version of what type classes or Erlang callback modules provide implicitly/dynamically.

### Sketch

```ash
pub type GenServerOps =
  GenServerOps {
    init: fn(Int) -> Act<CounterState>,
    on_call: fn(CounterCall, From<CounterReply>, CounterState)
      -> Act<CallAction<CounterState, CounterReply>>,
    on_cast: fn(CounterCall, CounterState)
      -> Act<CastAction<CounterState>>,
  };

pub fn counter_ops() -> GenServerOps {
  GenServerOps {
    init: fn(initial) {
      act { return CounterState { count: initial } }
    },
    on_call: fn(msg, from, state) {
      match msg {
        Get {} => act { return reply(Count { value: state.count }, state) },
        Inc { by } => act {
          let next = CounterState { count: state.count + by };
          return reply(Ack {}, next)
        },
        Reset {} => act { return reply(Ack {}, CounterState { count: 0 }) },
        Stop {} => act { return stop(Ack {}, state) },
      }
    },
    on_cast: fn(msg, state) {
      match msg {
        Reset {} => act { return noreply(CounterState { count: 0 }) },
        _ => act { return noreply(state) },
      }
    },
  }
}

pub fn start_counter(initial: Int) -> Proc<ServerHandle<CounterCall, CounterReply>> {
  gen_server::start(counter_ops(), initial)
}
```

### What this exercises

- First-class function values / closures.
- ADT request/reply typing.
- `Act` inside `Proc` through explicit lifting.
- Library-owned generic orchestration.

### Benefits

- Minimal new language machinery.
- Very explicit generic/concrete split.
- Easy to test with fake dictionaries.
- Can be optimized or specialized later.

### Gaps

- Current Ash user-surface support for higher-rank/generic records of functions may be incomplete or awkward.
- Dictionary values may carry large captured environments if users are careless, weakening the memory/GC teaching goal.
- Heterogeneous supervision trees require existential packaging, type erasure, or a separate handle abstraction.
- Diagnostics through dictionary fields may be less direct than named callbacks.

---

## 6. Pattern C: Reducer / state-machine GenServer

### Idea

Normalize the user-specific part to one state-transition function. The generic library owns all call/cast/reply/mailbox mechanics; the user owns `step`.

### Sketch

```ash
pub type ServerInput =
  Call { request: CounterCall, from: From<CounterReply> }
| Cast { request: CounterCall }
| Info { message: RuntimeInfo };

pub type StepAction =
  Reply { reply: CounterReply, state: CounterState }
| NoReply { state: CounterState }
| Stop { reply: CounterReply, state: CounterState };

pub fn counter_step(input: ServerInput, state: CounterState) -> Act<StepAction> {
  match input {
    Call { request: Get {}, from } =>
      act { return Reply { reply: Count { value: state.count }, state } },

    Call { request: Inc { by }, from } =>
      act { return Reply { reply: Ack {}, state: CounterState { count: state.count + by } } },

    Cast { request: Reset {} } =>
      act { return NoReply { state: CounterState { count: 0 } } },

    Call { request: Stop {}, from } =>
      act { return Stop { reply: Ack {}, state } },

    _ =>
      act { return NoReply { state } },
  }
}

pub fn start_counter(initial: Int) -> Proc<ServerHandle<CounterCall, CounterReply>> {
  let initial_state = CounterState { count: initial };
  gen_server::serve(initial_state, counter_step)
}
```

### What this exercises

- ADT-driven state machines.
- Explicit operational action vocabulary.
- Generic receive loop vs user transition function.
- Differential testing against other encodings.

### Benefits

- Very Ash-native: a process is an interpreter for a state-transition algebra.
- Easier to test than callback sets.
- Easier for humans and AI tools to inspect.
- Natural stepping stone to protocol descriptors and model checking.

### Gaps

- Large reducers can become monolithic without good examples for decomposition.
- Pattern matching and state update syntax must be aligned with live Ash surface.
- Delayed reply semantics need an explicit `From`/reply-handle design.
- Needs a clear story for non-call messages (`Info`) and runtime/system events.

---

## 7. Pattern D: `Proc` combinator server

### Idea

Make GenServer a normal `Proc` library combinator. The server loop is a process computation; starting it returns a typed server endpoint or process handle.

### Sketch

```ash
pub fn counter_proc(initial: Int) -> Proc<ServerHandle<CounterCall, CounterReply>> {
  do:Proc {
    let state = CounterState { count: initial };
    handle <- gen_server::spawn(state, counter_step);
    return handle
  }
}

pub fn example_client(server: ServerHandle<CounterCall, CounterReply>) -> Proc<Int> {
  do:Proc {
    r0 <- gen_server::call(server, Get {});
    _ <- gen_server::call(server, Inc { by: 2 });
    r1 <- gen_server::call(server, Get {});
    return counter_value(r1)
  }
}
```

### What this exercises

- `do:Proc` sequencing.
- `Proc` child process identity and handles.
- Explicit `Act` to `Proc` lifting where capability calls are used.
- Process-level failure and cancellation semantics.

### Benefits

- Keeps the tower clean: GenServer lives inside `Proc`, not as a new tower level.
- Works well with async `par`, `await`, `join`, and `gather` design work.
- Makes memory/GC pedagogy concrete: each server process retains only its state and loop environment, rather than a large top-level workflow retaining everything.

### Gaps

- Requires enough process runtime substrate: mailboxes/endpoints, process start, handle lifecycle, cancellation.
- `P<A>` vs `ServerHandle<Req, Resp>` must be separated: one is a running process observation handle; the other is communication authority.
- Affine/linear handle rules may constrain API ergonomics.
- Dropping/detaching server handles needs a policy.

---

## 8. Pattern E: Capability-backed handler

### Idea

The generic loop invokes an admitted capability implementation for user-specific behavior. This turns the callback dictionary into explicit authority.

### Sketch

```ash
pub capability CounterHandler {
  execute init(initial: Int) -> CounterState;
  execute handle(input: ServerInput, state: CounterState) -> StepAction;
}

pub capability implementation CounterHandlerImpl for CounterHandler {
  execute init(initial: Int) -> CounterState {
    CounterState { count: initial }
  }

  execute handle(input: ServerInput, state: CounterState) -> StepAction {
    match input {
      Call { request: Get {}, from } =>
        Reply { reply: Count { value: state.count }, state },
      Call { request: Inc { by }, from } =>
        Reply { reply: Ack {}, state: CounterState { count: state.count + by } },
      _ => NoReply { state },
    }
  }
}

pub workflow counter_server(initial: Int) -> ServerHandle<CounterCall, CounterReply> {
  requires: capability CounterHandler;

  do:Workflow {
    handle <- workflow::from_proc(
      gen_server::serve_with_capability("CounterHandler", initial)
    );
    return handle
  }
}
```

### What this exercises

- Capability interface declarations.
- Capability implementation dispatch.
- Admission/availability verification.
- Capability dependency/resource provenance.
- Host-provided vs Ash-defined implementations.

### Benefits

- Strong fit for Ash authority model.
- Useful when handler logic needs external effects: model calls, file IO, network, database access.
- Makes test doubles and simulation natural: swap capability implementation.
- Gives the runtime a clear admission point before the server loop starts.

### Gaps

- Need to prevent over-modeling ordinary local logic as capabilities.
- Exact source syntax for capability implementations and workflow admission must follow SPEC-052/SPEC-056 live status.
- Need clear effect/authority boundary between server endpoint authority and handler authority.
- Capability dispatch may be heavier than local function calls.

---

## 9. Pattern F: Resource-owned server

### Idea

Treat the server state as a runtime resource with controlled operations. The GenServer-like API becomes a capability/resource adapter rather than a mailbox-first actor.

### Sketch

```ash
pub resource CounterStore;

pub capability CounterApi {
  execute get(store: CounterStore) -> Int;
  execute inc(store: CounterStore, by: Int) -> Unit;
  execute reset(store: CounterStore) -> Unit;
}

pub workflow counter_resource_example() -> Int {
  requires: resource CounterStore;
  requires: capability CounterApi;

  do:Workflow {
    _ <- workflow::from_act(counter::inc(CounterStore, 2));
    value <- workflow::from_act(counter::get(CounterStore));
    return value
  }
}
```

### What this exercises

- Runtime resource identity and authority provenance.
- Capability/resource binding.
- Snapshot/restore or restart policy surfaces.
- State ownership without exposing shared mutable state directly.

### Benefits

- Good for services that are really governed stateful resources: caches, registries, queues, model-session stores.
- Avoids unnecessary actor/mailbox machinery when request serialization is enough.
- Natural place for checkpointing, resource lifecycle, and host authority.

### Gaps

- Less faithful to actor-style message ordering unless the resource layer defines serialization.
- Could drift toward object/service semantics if authority boundaries are unclear.
- Needs explicit policy for concurrent access, splitting, sharing, and merge behavior.
- Does not by itself demonstrate process communication unless combined with endpoint/process wrappers.

---

## 10. Pattern G: Workflow-governed server

### Idea

Represent a server as a governed workflow whose body is a `Proc` server and whose contract describes admission, obligations, failure/reporting, and allowed effects.

### Sketch

```ash
pub workflow governed_counter(initial: Int) -> ServerHandle<CounterCall, CounterReply> {
  requires: initial >= 0;
  requires: capability CounterHandler;
  ensures: server_started(result);

  do:Workflow {
    server <- workflow::from_proc(
      gen_server::serve_with_capability("CounterHandler", initial)
    );
    ensures: reachable(server);
    return server
  }
}
```

### What this exercises

- First-class `Workflow<A>` carrier and `WorkflowForm` projection.
- `requires`/`ensures` around process behavior.
- Lower process failure reinterpretation at workflow boundary.
- Report/provenance obligations for long-running processes.

### Benefits

- Excellent for auditable agent/tool servers.
- Demonstrates why `Workflow` is a governance envelope over `Proc`, not just a bigger function.
- Can expose the same server body with or without governance for comparison.

### Gaps

- Heavier than needed for a simple local server.
- Long-lived server handles create contract questions: what does the workflow guarantee after returning a handle?
- Ensures over future process behavior require latent/handle contracts or monitoring obligations.
- Need clear separation between server startup success and later child process failure.

---

## 11. Pattern H: Typed protocol / session-style server

### Idea

Describe the communication protocol explicitly and generate or check the server/client endpoints from that protocol. This improves on Erlang's untyped messages.

### Sketch

```ash
pub protocol CounterProtocol {
  client -> server: Get;
  server -> client: Count { value: Int };

  client -> server: Inc { by: Int };
  server -> client: Ack;

  client -> server: Stop;
  server -> client: Ack;
  end;
}

pub fn counter_from_protocol(initial: Int) -> Proc<Endpoint<CounterProtocol, ClientRole>> {
  protocol::serve(CounterProtocol, CounterState { count: initial }, counter_step)
}
```

### What this exercises

- Type-level protocol descriptors.
- Endpoint role/projection concepts.
- Client/server duality checks.
- Future distributed-node communication contracts.

### Benefits

- Strong teaching contrast with Erlang: same actor idea, but protocol violations can become type/checker errors.
- Good substrate for distributed Ash nodes and sandboxed/LLM participants.
- Protocol descriptors can generate documentation, diagrams, traces, and tests.

### Gaps

- Full session typing likely needs linear/affine endpoint state and richer type-level machinery.
- We should probably start with protocol descriptors and trace checking, not full MPST.
- Need to decide how protocol state interacts with `ServerHandle` reuse for ordinary services.
- Requires surface syntax and typechecker design that are not current MVP substrate.

---

## 12. Pattern I: Declarative server declaration / codegen

### Idea

Provide a user-facing declaration that expands to one of the lower-level patterns, most likely reducer + `Proc` combinator + typed endpoint.

### Sketch

```ash
pub server CounterServer {
  state CounterState { count: Int };

  init(initial: Int) {
    CounterState { count: initial }
  }

  call Get {} -> Count {
    Count { value: state.count }
  }

  call Inc { by: Int } -> Ack {
    state.count = state.count + by;
    Ack {}
  }

  cast Reset {} {
    state.count = 0;
  }
}
```

Desugaring target:

```text
server declaration
  -> message ADTs
  -> reducer function
  -> endpoint API helpers
  -> gen_server::serve(initial_state, reducer)
```

### What this exercises

- Future compile-time expansion / quotation / template machinery.
- Generated examples and docs.
- Consistent endpoint helper generation.
- Cross-checking generated reducer against direct reducer.

### Benefits

- Best eventual UX for common use cases.
- Great source for examples because boilerplate is hidden.
- Can enforce good practice by construction: small state, typed messages, explicit effects.

### Gaps

- Requires macro/codegen/derive or compiler-known declaration work.
- Error reporting and generated-code debugging must be designed carefully.
- Should not be the first semantic substrate; it should lower to an already tested core.
- Surface syntax would need a dedicated spec.

---

## 13. Pattern J: Supervisor-first server child specs

### Idea

Start from the supervision tree. A GenServer is one kind of child spec. The generic orchestration problem is not only “how do I loop?” but “how does this child start, fail, restart, and report?”

### Sketch

```ash
pub fn counter_child(initial: Int) -> supervisor::ChildSpec {
  supervisor::server_child(
    id: "counter",
    restart: supervisor::Permanent,
    intensity: supervisor::RestartWindow { max: 3, seconds: 60 },
    proc: counter_proc(initial),
  )
}

pub fn app_supervisor() -> Proc<SupervisorHandle> {
  supervisor::one_for_one([
    counter_child(0),
    router_child(),
    metrics_child(),
  ])
}
```

### What this exercises

- Process failure observation.
- Restart strategies and intensity windows.
- Child process identity and reporting.
- Bisimulation against unsupervised server variants under failure injection.

### Benefits

- Mirrors why OTP patterns matter in practice: reliable systems are supervised systems.
- Gives concrete examples for failure boundaries and memory cleanup after restart.
- Good differential-test driver: same server under no supervision, one-for-one, one-for-all, rest-for-one.

### Gaps

- Requires clear process failure semantics, child terminal observation, cancellation, and restart admission.
- Links/monitors equivalent remains open.
- State restart policy needs explicit choice: clean start, preserved snapshot, or explicit checkpoint resource.
- Need to avoid folding too much workflow governance into basic supervision too early.

---

## 14. Comparison table

| Pattern | Generic/concrete split | Primary Ash features exercised | Best use | Main gap |
|---|---|---|---|---|
| Direct OTP mirror | Runtime callback module / callback table | mailbox, call/cast, process loop | Erlang comparison | dynamic dispatch and failure translation |
| Callback dictionary | Explicit value-level functions | function values, ADTs, `Act`/`Proc` | MVP library pattern | ergonomics and heterogeneity |
| Reducer/state machine | One `step` function | ADTs, pattern matching, process loop | canonical comparable core | delayed replies and system messages |
| `Proc` combinator | Library process constructor | `do:Proc`, process handles, endpoints | Ash-native process examples | mailbox/runtime substrate |
| Capability-backed handler | Admitted capability implementation | capabilities, resources, admission | effectful/authority-sensitive servers | overusing capabilities for local logic |
| Resource-owned server | Runtime resource + capability ops | resource identity, provenance, access modes | caches/stores/registries | actor ordering/concurrency semantics |
| Workflow-governed server | Workflow contract over process body | `WorkflowForm`, contracts, reporting | audited agent/tool servers | latent handle contracts |
| Typed protocol server | Protocol descriptor/endpoint roles | type-level protocols, future MPST | distributed/sandboxed communication | linear/session substrate |
| Declarative server | compiler/library expansion | codegen, examples, generated APIs | final user ergonomics | macro/template substrate |
| Supervisor-first | child specs and restart policies | process failure, restart, reporting | reliability examples | links/monitors/restart semantics |

---

## 15. Differential and bisimulation test plan

A future test suite should keep one scenario and implement it in several patterns.

### 15.1 Observable events

For each implementation, collect an abstract trace:

```text
Started(server_id)
CallSent(client_id, server_id, Get, ref0)
ReplyReceived(client_id, ref0, Count(0))
CallSent(client_id, server_id, Inc(2), ref1)
ReplyReceived(client_id, ref1, Ack)
CallSent(client_id, server_id, Get, ref2)
ReplyReceived(client_id, ref2, Count(2))
CastSent(client_id, server_id, Reset)
CallSent(client_id, server_id, Get, ref3)
ReplyReceived(client_id, ref3, Count(0))
Stopped(server_id, Normal)
```

Comparable implementations should produce equivalent traces modulo internal event labels.

### 15.2 Candidate variants

1. Direct hand-written process loop.
2. Reducer/state-machine loop.
3. Callback dictionary loop.
4. Capability-backed handler loop.
5. Workflow-governed loop.
6. Resource-backed loop, where applicable.
7. Generated/declarative server, once available.

### 15.3 Failure-injection cases

- Handler returns operational failure on `Inc(-1)`.
- Handler traps/panics, if a panic boundary exists.
- Client call times out.
- Server stops normally while clients are waiting.
- Server crashes while a supervisor is watching.
- Restart exceeds intensity window.
- Capability required by handler is missing or wrong-mode.
- Resource handle is unavailable/revoked.

### 15.4 Properties

- Sequential call ordering is preserved per server endpoint.
- Calls receive exactly one reply or one timeout/failure.
- Casts do not require replies.
- State transitions match the reference model.
- Restart policy determines whether state is clean, restored, or unavailable.
- Workflow-governed variants do not widen authority relative to declared requirements.
- Resource-backed variants do not expose hidden mutable state outside declared capabilities.

---

## 16. Memory/GC and program-structure examples

The examples should explicitly contrast two styles.

### 16.1 Monolithic workflow anti-pattern

```ash
pub workflow monolithic_agent_app(input: Request) -> Response {
  do:Workflow {
    let conversation = load_large_history(input);
    let tool_registry = build_all_tools();
    let cache = build_large_cache();
    let router_state = build_router_state();

    result <- run_everything_in_one_body(
      conversation,
      tool_registry,
      cache,
      router_state,
      input,
    );

    return result
  }
}
```

Risk: one large definition may retain broad environment state longer than needed, obscure failure boundaries, and make restart/testing coarse.

### 16.2 Communicating processes/workflows pattern

```ash
pub workflow agent_app(input: Request) -> Response {
  do:Workflow {
    cache <- workflow::from_proc(cache_server());
    router <- workflow::from_proc(router_server(cache));
    tools <- workflow::from_proc(tool_server());

    response <- workflow::from_proc(
      router::call(router, Route { input, tools })
    );

    return response
  }
}
```

Benefit: each process owns a smaller state slice; restart/failure/reporting boundaries are clearer; tests can exercise each server independently and together.

Open requirement: the runtime must make the retained-state/process-boundary benefit observable enough to teach and benchmark honestly.

---

## 17. Current gaps and questions

### 17.1 Process and endpoint substrate

- What is the exact distinction between `P<A>` process observation handles and reusable server communication endpoints?
- Are endpoints affine, duplicable, capability-backed, or resource-backed?
- How are mailboxes represented: built-in process mailbox, channel resource, capability, or library abstraction?
- What are the ordering, backpressure, and timeout semantics?

### 17.2 Callback and function-value substrate

- Can user-facing Ash ergonomically represent dictionaries of functions/closures?
- How much genericity is available for reusable `GenServerOps<State, Req, Resp>`-like APIs?
- Are closure captures visible/auditable enough to avoid hidden memory retention?

### 17.3 Failure and supervision

- How exactly does operational `fail` inside a server handler propagate to process failure, supervisor observation, and workflow failure?
- What is the Ash equivalent of Erlang links and monitors?
- Does restart start from clean `init`, from explicit snapshot resource, or from a user-provided recovery action?
- How are pending calls completed when a server stops or restarts?

### 17.4 Workflow contracts for long-lived handles

- If a workflow returns a server handle, what does `ensures` mean for future behavior after the workflow body returns?
- Do server handles need latent contracts that are discharged by `call`, `await`, `join`, or monitor operations?
- How does workflow reporting represent later child process failures?

### 17.5 Capability/resource authority

- When should a handler be a local function vs a capability implementation?
- How is handler authority separated from endpoint authority?
- Can capability admission happen once at server start, or must every message re-check authority?
- How do revocation and resource lifecycle affect a running server?

### 17.6 Type-level protocols

- Should the first protocol work be a lightweight descriptor/trace checker rather than full session typing?
- What linearity or affine-state machinery is required before endpoint state can be represented in types?
- How do protocols compose with distributed Ash nodes, external actors, and LLM/tool participants?

### 17.7 Differential testing mechanics

- What is the canonical trace format for comparing implementations?
- Which events are externally observable vs internal instrumentation?
- How do we normalize traces across direct loop, reducer, capability, workflow, and resource variants?
- Can examples double as conformance tests without becoming too brittle?

---

## 18. Suggested next artifacts

1. **Reference model:** a small abstract counter-server trace model independent of any implementation style.
2. **Example packet:** three non-normative examples implementing the same counter behavior as direct loop, reducer, and capability-backed handler.
3. **Gap note:** endpoint/mailbox/handle taxonomy: `P<A>` vs `ServerHandle<Req, Resp>` vs capability endpoint vs resource handle.
4. **Future spec candidate:** a narrow `gen_server::serve` stdlib design centered on reducer + `Proc` combinator, with adapters for callback dictionaries and capability handlers.
5. **Testing note:** differential trace schema for communicating-process examples.

---

## 19. Working recommendation

Use the reducer/`Proc` combinator as the candidate semantic core:

```text
state + input -> Act<step action>
serve(state, step) -> Proc<server endpoint>
```

Then treat other styles as adapters or surfaces:

```text
callback dictionary -> reducer
capability implementation -> reducer
protocol descriptor -> reducer + endpoint API
declarative server -> generated reducer + Proc combinator
workflow-governed server -> Workflow contract over the Proc server
supervisor child spec -> supervised process around the same Proc server
```

This keeps one comparable behavior model while allowing Ash to showcase many implementation patterns and feature strata.
