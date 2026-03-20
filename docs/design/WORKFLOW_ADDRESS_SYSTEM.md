# Workflow Address System: Minimal Design

Based on discussion: Combine Rust affine types + process calculus + Ash capabilities.

## Core Principle

> **Addresses are affine capabilities**: move by default, clone only when explicitly allowed.

This gives us:
- **Safety**: Can't accidentally use address after passing it
- **Control**: Supervisor knows exactly who has references to supervised workflows
- **Composability**: Can build complex routing topologies safely

This affine-address story does not imply one-shot control operations. `ControlLink` is a separate
reusable supervision authority whose validity is governed by instance lifecycle, not by automatic
consumption on first use.

---

## 1. Address Type (Opaque)

```rust
// Public API - opaque
pub struct Address(AddressInner);

// Private implementation
enum AddressInner {
    Local { 
        id: Uuid,
        generation: u32,
    },
    // Future: Distributed variant (opaque to users)
    Remote(OpaqueBytes),
}

// Trait for address capabilities
pub trait AddressCapabilities {
    fn send(&self, msg: Message) -> Result<(), SendError>;
    fn is_alive(&self) -> bool;
    fn await_exit(&self) -> ExitReason;  // Async wait
}
```

**Key**: Users cannot inspect, construct, or serialize addresses directly. They're truly opaque capabilities.

---

## 2. Affine by Default

```rust
// Affine marker trait - move semantics
pub trait Affine: Sized {
    // Cannot be cloned, must be moved
}

impl Affine for Address {}

// Can be explicitly made Copyable
pub trait Shareable: Affine {
    fn share(&self) -> (Self, Self);  // Split into two
}

// Some addresses are shareable (e.g., public services)
impl Shareable for ServiceAddress { }
```

### In Ash Syntax

```ash
-- By default, addresses are affine (move semantics)
receive {
    worker:address as w => {
        send w:task with { work: 1 };  -- OK, using w
        send w:task with { work: 2 };  -- OK, still have w
        
        send delegate:forward with { worker: w };  -- MOVED! w is now gone
        
        send w:task with { work: 3 };  -- ERROR: w was moved
    }
}

-- Shareable addresses can be split
receive {
    service:shareable_address as s => {
        let (s1, s2) = share(s);  -- Split into two addresses
        
        send pool:register with { service: s1 };
        send client:provide with { service: s2 };
        -- Both valid, both point to same workflow
    }
}
```

---

## 3. Control Link ≠ Mailbox (Supervision)

**Separation of concerns**:

- **Mailbox**: For business logic messages (`send`, `receive`)
- **Control Link**: For supervision (supervisor ↔ supervised)

```rust
// Control link is a separate, bidirectional channel
pub struct ControlLink {
    supervised: Address,
    supervisor: Address,
    
    // Control channel (guaranteed delivery, bypasses mailbox limits)
    control_tx: mpsc::Sender<ControlMsg>,
    control_rx: mpsc::Receiver<ControlMsg>,
    
    // Link properties
    heartbeat_expected: Duration,
    heartbeat_timeout: Duration,
}

// Control messages (system-level, not application-level)
pub enum ControlMsg {
    // Supervisor → Supervised
    Shutdown { deadline: Instant },
    HeartbeatRequest { sequence: u64 },
    InspectState,  -- Request state dump for debugging
    
    // Supervised → Supervisor
    HeartbeatReply { sequence: u64, state_hash: u64 },
    Progress { milestone: String },  -- Application-level progress
    Exiting { reason: ExitReason },
    StuckAlert { condition: StuckCondition },
}
```

### Why Separate?

1. **Reliability**: Control messages bypass mailbox backpressure
2. **Priority**: Control always processed before business messages
3. **Simplicity**: Mailbox can be full, but supervisor can still shutdown worker
4. **Trust**: Control link is established by runtime, not user code

### Control Authority Semantics

`ControlLink` is reusable while the supervised instance remains valid.

- `check_health` is non-terminal and reusable
- `pause` is non-terminal and reusable
- `resume` is non-terminal and reusable
- `kill` is terminal and invalidates future control operations

After `kill`, the current runtime contract retains a terminal tombstone for the lifetime of the
owning `RuntimeState` so later control attempts fail explicitly as terminal control rather than as
unknown-link lookup. See [Control-Link Retention Policy](../reference/control-link-retention-policy.md).

So authority transfer is explicit, but successful control use is not globally one-shot.

---

## 4. Minimal Supervisor Pattern (Library)

```ash
-- Library: std/supervision.ash

-- Supervision strategy
type Strategy = OneForOne | OneForAll | RestForOne;

-- Spawn with supervision
pub capability supervisor_spawn {
    effect: act,
    params: [
        workflow: WorkflowDef,
        init: Value,
        strategy: Strategy,
        max_restarts: Int,
        max_time: Duration
    ],
    returns: SupervisedChild
}

-- Control operations
pub capability supervisor_control {
    effect: act,
    params: [
        operation: ControlOp,
        child: SupervisedChild
    ]
}

-- Library workflow: one_for_one supervisor
pub workflow one_for_one_supervisor {
    -- Parameters from parent
    let specs = $input.child_specs;  -- List of child specifications
    let max_restarts = $input.max_restarts or 5;
    let max_time = $input.max_time or 60s;
    
    -- State
    let children = [];
    let restart_counts = {};
    
    -- Initialize children
    for spec in specs do {
        spawn spec.workflow with {
            init: spec.init,
            -- Runtime establishes control link automatically
            link_as: supervised
        } as child;
        
        let children = children + [{
            spec: spec,
            address: child,
            restart_count: 0,
            last_restart: now()
        }];
    };
    
    -- Supervision loop (minimal)
    loop {
        receive control wait heartbeat_interval {
            -- Child exited
            { from: child, event: exit, reason: r } => {
                let entry = find(children, child);
                
                decide { should_restart(entry, max_restarts, max_time) } then {
                    restart_child(entry);
                    let children = update_restart_count(children, child);
                } else {
                    -- Too many restarts, escalate
                    exit({ error: max_restarts_exceeded, child: child });
                }
            },
            
            -- Child stuck (no heartbeat)
            { from: child, event: stuck } => {
                send_control child with shutdown;
                kill child;  -- Force kill if graceful fails
                restart_child(find(children, child));
            },
            
            -- External scale command
            { command: scale, add: n } => {
                repeat n times {
                    spawn default_spec.workflow with {
                        init: default_spec.init,
                        link_as: supervised
                    } as new_child;
                    let children = children + [{...}];
                }
            },
            
            -- Timeout: just continue (heartbeats happen automatically)
            _ => continue
        }
    }
}

-- Helper workflows (private to library)
workflow restart_child(entry) {
    spawn entry.spec.workflow with {
        init: entry.spec.init,
        link_as: supervised
    } as new_child;
    
    ret { old: entry.address, new: new_child }
}

workflow should_restart(entry, max_r, max_t) {
    -- Check if within restart window
    let recent_restarts = count_recent_restarts(entry, max_t);
    ret recent_restarts < max_r
}
```

---

## 5. Address Passing Example (Process Calculus Style)

```ash
-- Ping-pong with address passing (like π-calculus name passing)

workflow ping {
    -- Start pong, give it our address
    spawn pong with { return_to: self } as pong_ref;
    
    receive wait 5s {
        pong:ready => {
            send pong_ref:ball with { count: 0, return_to: self };
        }
    };
    
    receive wait 5s {
        { from: sender, ball: { count: n, return_to: return_addr } } => {
            if n >= 10 then {
                ret "done"
            } else {
                -- Pass the ball back, forwarding the return address
                send return_addr:ball with { 
                    count: n + 1, 
                    return_to: self 
                };
            }
        }
    }
}

workflow pong {
    let return_to = $input.return_to;
    
    send return_to:ready with {};
    
    loop {
        receive {
            { ball: b } => {
                -- Forward to whoever sent the ball
                send b.return_to:ball with b;
            }
        }
    }
}
```

---

## 6. Runtime Pulse (Minimal)

Runtime only provides **heartbeat pulse** to top-level supervisor:

```rust
// Runtime does this periodically
pub fn runtime_pulse(supervisor: Address) {
    // Just check supervisor is alive
    if !supervisor.is_alive() {
        panic!("Top-level supervisor failed");
    }
    
    // Supervisor handles all downstream monitoring
}
```

Everything else is **application-level**:
- Workers decide their own progress milestones
- Supervisors decide restart strategies
- Heartbeats are control channel messages, not runtime magic

---

## 7. Minimal API Summary

### Language Extensions (Minimal)

```ash
-- Spawn (returns affine Address)
spawn workflow_expr with { init: value } as name;

-- Send (consumes nothing, Address is Copy for sending)
send address:channel with payload;
send_control address with control_msg;  -- Uses control link

-- Receive (mailbox)
receive [wait timeout] { patterns }

-- Receive control (control link)
receive control [wait timeout] { control_patterns }

-- Linking (runtime establishes control link)
link address with { options };
monitor address;  -- Receive exit notifications

-- Address type annotation
let worker:address = ...;
let shared:shareable_address = ...;
```

### Standard Library (Minimal)

```ash
-- std/supervision.ash
pub workflow one_for_one_supervisor;
pub workflow one_for_all_supervisor;
pub workflow rest_for_one_supervisor;

-- std/address.ash
pub fn share(addr: shareable_address) -> (shareable_address, shareable_address);
pub fn is_alive(addr: address) -> Bool;
pub fn await_exit(addr: address) -> ExitReason;
```

---

## 8. Implementation Phases

### Phase 1: Core Address System
- `Address` type (affine, opaque)
- `spawn` expression
- `send` with address
- Basic `receive`
- No supervision yet

### Phase 2: Control Links
- Separate control channel
- `receive control`
- `send_control`
- Heartbeat protocol (application-level)

### Phase 3: Supervision Library
- `one_for_one_supervisor` workflow
- Restart counting
- Link lifecycle management

### Phase 4: Advanced Patterns
- Address sharing (`Shareable` trait)
- Dynamic supervision trees
- Hot code upgrades (maybe)

---

## Comparison with Other Models

| Feature | Erlang | Rust (Tokio) | π-calculus | **Ash (Proposed)** |
|---------|--------|--------------|------------|-------------------|
| Spawn | `spawn` | `tokio::spawn` | `new channel` | `spawn with` |
| Address | Pid (copy) | `JoinHandle` (copy) | Channel name | `Address` (affine) |
| Send | `!` | `tx.send()` | `x̄<y>` | `send addr:ch` |
| Receive | `receive` | `rx.recv()` | `x(y)` | `receive { }` |
| Control | Exit signals | `AbortHandle` | - | `control` channel |
| Passing | Copy | Clone | Name passing | Affine move |
| Supervision | OTP | None | - | Library pattern |

---

## Open Questions

1. **Cloneable addresses**: Use `Shareable` trait, or `Arc<Address>` pattern?
2. **Address comparison**: Should `==` work on addresses? (Probably not - use `same_workflow()`)
3. **Control link lifetime**: Tied to supervision, or can exist independently?
4. **Mailboxes per address**: One mailbox per workflow, or multiple channels?

Let's implement Phase 1 and iterate.
