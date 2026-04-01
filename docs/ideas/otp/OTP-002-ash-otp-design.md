# OTP-002: OTP-like Functionality in Ash — Design Considerations

**Status:** Drafting  
**Depends on:** OTP-001  
**Related:** Future work on Ash runtime architecture

---

## 1. Problem Statement

The goal is to investigate implementing OTP-like supervision, fault tolerance, and process management patterns in Ash. This is complicated by fundamental differences between Erlang's actor model and Ash's capability-effect system.

**Key Questions:**
1. Can we achieve Erlang-like fault isolation without VM-level process boundaries?
2. How do we model supervision trees in a type-safe way?
3. What supervision semantics make sense for Ash's error model?
4. Can we leverage Rust's structured concurrency primitives?

---

## 2. Design Options Analysis

### 2.1 Option A: Direct Erlang Port (Not Recommended)

**Approach:** Implement gen_server and supervisor behaviors as close as possible to Erlang.

**Implementation Sketch:**
```ash
// gen_server-like behavior
trait GenServer {
    type State;
    type Request;
    type Response;
    
    fn init(args: InitArgs) -> Result<State, Error>;
    fn handle_call(req: Request, from: Caller, state: State) 
        -> Result<(Response, State), Error>;
    fn handle_cast(req: Request, state: State) 
        -> Result<State, Error>;
}
```

**Problems:**
- ❌ Panic handling doesn't match Erlang's "let it crash"
- ❌ **Incorrect:** Ash *does* have workflow state isolation (equivalent to process isolation)
- ❌ **Incorrect:** Message passing maps directly to Ash effects (send/receive)
- ❌ Forced to use catch_unwind for every call
- ❌ Goes against Rust's error handling philosophy

**Verdict:** Infeasible without significant runtime infrastructure. **Note:** Initial assessment overestimated the gap—Ash has stronger Erlang parallels than recognized (see OTP-001, Section 8).

---

### 2.2 Option B: Capability-Based Supervision (Recommended for Exploration)

**Approach:** Model supervision as capability hierarchies with explicit error contracts.

**Core Concepts:**
- **Supervision Capabilities**: Grant authority to spawn/monitor children
- **Child Contracts**: Explicit error types and restart policies
- **Effect-Based Monitoring**: Children report status via effects

**Implementation Sketch:**
```ash
// Supervision capability
cap Supervision<S: Strategy> {
    effect Spawn<T: Task>(spec: ChildSpec<T>) -> ChildRef<T>;
    effect Monitor(child: ChildRef) -> Stream<ChildEvent>;
    effect Restart(child: ChildRef) -> Result<(), SupervisionError>;
}

// Child specification with typed error
type ChildSpec<T: Task> = {
    factory: fn() -> T,
    restart: RestartPolicy<T::Error>,
    max_restarts: u32,
    window_secs: u32,
};

// Restart policies as functions
enum RestartPolicy<E> {
    Always,                           // Restart on any error
    OnError(fn(&E) -> bool),         // Conditional restart
    Never,                            // Never restart
}
```

**Advantages:**
- ✅ Type-safe error handling
- ✅ Composable with Ash's effect system (message passing *is* effects)
- ✅ Workflow isolation equivalent to Erlang process isolation
- ✅ Explicit contracts
- ✅ Leverages Rust's Result types

**Challenges:**
- ⚠️ **Primary:** Generic/concrete split for OTP behaviors (OTP-001, Section 9)
- ⚠️ Different error model (Result types vs "let it crash")
- ⚠️ Uncertain: links/monitors equivalent for supervision trees

---

### 2.3 Option C: Runtime-Based Isolation (High Effort)

**Approach:** Build runtime support for isolated "Ash processes" similar to Erlang processes.

**Core Components:**
1. **Ash VM Extension**: Managed execution contexts
2. **Isolated Heaps**: Per-process memory regions
3. **Message Passing Runtime**: Built-in mailbox support
4. **Panic Isolation**: catch_unwind at process boundaries

**Advantages:**
- ✅ True Erlang-like semantics
- ✅ Can implement full OTP behaviors
- ✅ Strong isolation guarantees

**Disadvantages:**
- ❌ Massive implementation effort
- ❌ Diverges from Rust ecosystem
- ❌ Performance overhead
- ❌ Complexity explosion

**Verdict:** Out of scope for current Ash roadmap.

---

### 2.4 Option D: Structured Concurrency Integration

**Approach:** Leverage Rust's structured concurrency (nurseries, task scopes) with Ash extensions.

**Core Concepts:**
- **Nurseries**: Scoped task spawning
- **Cancellation Trees**: Parent cancellation propagates to children
- **Result Aggregation**: Collect child results/errors

**Implementation Sketch:**
```ash
// Structured scope with supervision
scope |supervisor| {
    // Spawn children - they live until scope ends
    let child1 = supervisor.spawn(worker1_spec);
    let child2 = supervisor.spawn(worker2_spec);
    
    // Supervisor handles child completions
    match supervisor.join_any().await {
        Ok(result) => { /* child completed successfully */ }
        Err(e) => {
            // Decide: restart, abort others, etc.
            supervisor.restart(&child1)?;
        }
    }
}
// All children cancelled when scope exits
```

**Advantages:**
- ✅ Idiomatic Rust patterns
- ✅ Existing ecosystem support (tokio::task, etc.)
- ✅ Clear lifetime semantics
- ✅ Cancellation support

**Challenges:**
- ❌ Different from Erlang supervision
- ❌ Limited restart strategy expressiveness
- ❌ Tied to async runtime

---

## 3. Recommended Hybrid Approach

### 3.1 Core Principles

1. **Explicit over Implicit**: Error handling via Result, not catch_unwind
2. **Capability-Based**: Supervision as capability hierarchies
3. **Composable**: Individual components can be used independently
4. **Async-Native**: Built for Rust's async ecosystem

### 3.2 Component Breakdown

#### 3.2.1 Task Trait

```ash
// Fundamental unit of work
trait Task: Send + 'static {
    type Output;
    type Error: std::error::Error;
    
    async fn run(self) -> Result<Self::Output, Self::Error>;
    
    // Optional lifecycle hooks
    async fn on_error(&mut self, err: &Self::Error) {}
    async fn shutdown(&mut self) {}
}
```

#### 3.2.2 Child Specification

```ash
type ChildSpec<T: Task> = {
    // Construction
    id: ChildId,
    factory: fn() -> T,
    
    // Restart configuration
    restart: RestartPolicy<T::Error>,
    max_restarts: u32,
    restart_window: Duration,
    
    // Shutdown configuration
    shutdown_timeout: Duration,
    shutdown_order: u32,  // For rest_for_one semantics
};

enum RestartPolicy<E> {
    Permanent,              // Always restart
    Transient(fn(&E) -> bool),  // Restart on matching errors
    Temporary,              // Never restart
}
```

#### 3.2.3 Supervisor Capabilities

```ash
// Basic supervision
cap Supervisor<S: Strategy> {
    // Lifecycle
    effect Start<T: Task>(spec: ChildSpec<T>) -> ChildHandle<T>;
    effect Stop(child: ChildHandle, timeout: Duration) -> Result<(), StopError>;
    
    // Monitoring
    effect Subscribe -> Stream<SupervisionEvent>;
    
    // Introspection
    effect ListChildren -> Vec<ChildInfo>;
    effect GetStatus(child: ChildHandle) -> ChildStatus;
}

// One-for-one strategy
cap OneForOne: Supervisor<OneForOne> {
    // Only specific to this strategy if needed
}

// One-for-all strategy  
cap OneForAll: Supervisor<OneForAll> {
    effect RestartAll(except: Option<ChildHandle>);
}

// Rest-for-one strategy
cap RestForOne: Supervisor<RestForOne> {
    // Implicitly tracks startup order
}
```

#### 3.2.4 Event Types

```ash
enum SupervisionEvent {
    ChildStarted { id: ChildId, at: Timestamp },
    ChildCompleted { id: ChildId, result: Result<(), Box<dyn Error>> },
    ChildRestarted { id: ChildId, attempt: u32, reason: RestartReason },
    ChildStopped { id: ChildId, reason: StopReason },
    IntensityExceeded { supervisor: SupervisorId, children: Vec<ChildId> },
}

enum RestartReason {
    Error(Box<dyn Error>),
    Panic(Option<String>),
    Requested,
}
```

### 3.3 Comparison to Erlang/OTP

| Feature | Erlang/OTP | Ash Hybrid | Notes |
|---------|-----------|------------|-------|
| Unit of work | Process | Workflow / Task | Both have isolated state |
| Error detection | Exit signals | Result + panic catch | Different error models |
| Restart trigger | Process crash | Explicit error + policy | Ash uses Result types, not crashes |
| State isolation | Process heap | Workflow isolation | **Equivalent** - both isolated per unit |
| Communication | Messages | Effects (send/receive) | **Equivalent** - both message passing |
| Process identity | Pid | Workflow address | **Equivalent** - both opaque identifiers |
| Supervision tree | Process hierarchy | Capability hierarchy | Uncertain: links/monitors equivalent? |
| Restart intensity | Built-in | Explicit configuration | Can be implemented |
| Hot code upgrade | Built-in | Not supported | Requires VM support |

**Revised Assessment:** The Ash/Erlang mapping is stronger than initially assessed. The primary uncertainty is the **generic/concrete split** (OTP-001, Section 9) not fundamental semantic gaps.

---

## 4. Open Questions

### 4.1 Error Handling Semantics

**Q:** Should panics be treated as restartable errors or fatal failures?

**Options:**
1. **Panics = Fatal**: Only Result::Err triggers restart
2. **Panics = Catchable**: catch_unwind at supervisor boundary
3. **Configurable**: Per-child panic policy

### 4.2 State Migration on Restart

**Q:** How should child state be handled across restarts?

**Options:**
1. **Clean slate**: New instance, fresh state (Erlang model)
2. **State preservation**: Pass previous state to new instance
3. **Snapshot/restore**: Explicit checkpoint mechanism

### 4.3 Dynamic vs Static Supervision

**Q:** Support for dynamic child addition (simple_one_for_one)?

**Considerations:**
- Static: All children defined at startup
- Dynamic: Children added at runtime
- Hybrid: Static base + dynamic pool

### 4.4 Cross-Node Supervision

**Q:** Should supervision work across distributed nodes?

**Considerations:**
- Network partitions
- Node failure detection
- Split-brain scenarios

### 4.5 Integration with Existing Ash Features

**Q:** How does supervision interact with:
- Capability revocation?
- Effect handlers?
- Resource management?

---

## 5. Implementation Phases

### Phase 1: Core Task Abstraction
- Define `Task` trait
- Implement basic child specification
- Create task spawning/monitoring

### Phase 2: Single Supervisor
- Implement one-for-one strategy
- Add restart intensity tracking
- Event streaming

### Phase 3: Multiple Strategies
- one-for-all
- rest-for-one
- Simple (dynamic) supervision

### Phase 4: Supervision Trees
- Nested supervisors
- Tree-wide operations
- Intensity propagation

### Phase 5: Advanced Features
- Graceful shutdown
- State snapshots
- Distributed supervision (if applicable)

---

## 6. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Semantic mismatch with Erlang | High | Document differences clearly; don't claim full OTP compatibility |
| Performance overhead | Medium | Benchmark early; optimize critical paths |
| Complexity explosion | High | Start minimal; add features incrementally |
| User confusion | Medium | Clear documentation; examples; migration guides |
| Maintenance burden | Medium | Good test coverage; clear specification |

---

## 7. Related Work

### 7.1 Rust Ecosystem

- **`bastion`**: Erlang-like runtime for Rust (highly ambitious)
- **`actix`**: Actor framework (more Akka-like than OTP)
- **`xtra`**: Actor framework with async/await
- **`tokio::task`**: Structured concurrency primitives
- **`besedarium`** (~/Projects/besedarium): Multiparty session types library with type-safe communication protocols. **Highly relevant**—demonstrates how typed channels can replace Erlang's untyped message passing, with compile-time protocol verification through role projection and duality checking.

### 7.2 Session Types and Typed Communication

**Besedarium** (~/Projects/besedarium) implements multiparty session types (MPST) for Rust, providing:

| Feature | Erlang | Besedarium | Relevance to Ash OTP |
|---------|--------|------------|---------------------|
| Message passing | Untyped (`pid() ! Msg`) | Typed channels (`EpChanSend<Msg, Cont>`) | Ash effects already typed; session types could enforce call/cast protocols |
| Protocol verification | Runtime pattern matching | Compile-time duality checking | Protocol violations caught at compile time |
| Role projection | N/A (no global protocol) | Global → local endpoint projection | Could model gen_server client/server contracts |
| State machine | Implicit in `receive` clauses | Explicit channel types (`EpChanSend` → `EpChanRecv` → `EpChanEnd`) | Workflow state transitions as types |
| Async runtime | BEAM processes | Tokio tasks | Ash uses similar runtime model |

**Key Insight:** Session types provide a **type-safe alternative** to Erlang's untyped message passing. Where Erlang uses `handle_call/handle_cast` with runtime pattern matching, session types encode the protocol directly in the channel type.

**Example Comparison:**

```erlang
% Erlang: untyped, runtime pattern matching
handle_call({get, Key}, _From, State) ->
    {reply, maps:get(Key, State), State};
handle_call({put, Key, Value}, _From, State) ->
    {reply, ok, maps:put(Key, Value, State)}.
```

```rust
// Besedarium: typed protocol as channel state machine
type ServerEndpoint = EpChanRecv<
    ServerIO,
    GetRequest,
    EpChanSend<ServerIO, Value, EpChanEnd<...>>,  // Get branch
    EpChanRecv<
        ServerIO,
        PutRequest,
        EpChanSend<ServerIO, Ack, EpChanEnd<...>>, // Put branch
        ...
    >
>;
```

**For Ash OTP:** Session types could:
1. Replace Erlang's untyped `gen_server:call/cast` with typed effect channels
2. Enforce client-server protocol contracts at compile time
3. Generate protocol documentation (Mermaid diagrams) automatically

**Constraint:** Besedarium uses type-state encoding (channel types carry protocol state as type parameters: `EpChanSend<Msg, Cont>`). This is **difficult to implement in Ash** due to [reasons TBD - dependent type complexity? linear type requirements?]. Alternative approaches may be needed.

### 7.3 Academic Research

- "Reliable Distributed Systems with Erlang/OTP" (Cesarini)
- "Languages and Patterns for Reliable Distributed Systems"
- **Honda, Vasconcelos, Kubo**: "Language primitives and type discipline for structured communication-based programming" (ESOP '98) — session types foundation
- **Yoshida, Carbone**: "Multiparty asynchronous session types" (POPL '15) — multiparty session types
- Session types and supervision (relevant to Ash's session types work)

### 7.4 Other Languages

- **Akka** (Scala/JVM): Actor model with supervision
- **Pony**: Actor-model language with capabilities
- **Elixir**: Erlang with modern syntax (built on same VM)

---

## 8. Recommendation

**Proceed with Option B (Capability-Based Supervision)** as a design exploration, with the following constraints:

**Revised Understanding:** The Ash/Erlang mapping is stronger than initially assessed (see OTP-001, Section 8). The primary architectural uncertainty is the **generic/concrete split** (OTP-001, Section 9), not fundamental semantic mismatches.

1. **Explicit non-goals**:
   - Full Erlang/OTP compatibility
   - Hot code loading
   - Massive parallelism (millions of processes)
   - Distributed supervision (initially)

2. **Core deliverables**:
   - Resolution of generic/concrete split (ad-hoc polymorphism without type classes)
   - `Task` / workflow trait definition
   - Basic supervisor capability
   - One-for-one restart strategy
   - Event streaming
   - Comprehensive documentation

3. **Success criteria**:
   - Clean generic/concrete separation (dictionary passing or monomorphization)
   - Type-safe supervision trees
   - Composable with existing Ash code
   - Clear error messages
   - Good performance characteristics

4. **Next steps**:
   - **First:** Resolve generic/concrete split design (OTP-001, Section 9)
   - Create proof-of-concept implementation
   - Write specification document
   - Build example applications
   - Gather community feedback

---

## 9. References

See OTP-001 for Erlang/OTP references.

Additional Rust-specific references:

1. **Bastion**: https://github.com/bastion-rs/bastion
2. **Actix**: https://actix.rs/
3. **Tokio**: https://tokio.rs/
4. **Structured Concurrency** (Martin Sústrik): http://250bpm.com/blog:71

---

## 10. Document History

| Date | Author | Change |
|------|--------|--------|
| 2026-03-31 | Hermes | Initial design exploration |
| 2026-03-31 | Hermes | Synced with OTP-001 corrections: state isolation, message passing as effects, PIDs vs addresses, generic/concrete split as primary uncertainty |
| 2026-03-31 | Hermes | Added besedarium session types analysis: typed communication as alternative to Erlang's untyped message passing |
| 2026-03-31 | Hermes | Added constraint: type-state encoding (used in besedarium) is difficult to implement in Ash |
