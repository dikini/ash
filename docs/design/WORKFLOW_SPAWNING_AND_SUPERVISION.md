# Workflow Spawning and Supervision (OTP-Inspired Design)

This document proposes extensions to Ash for dynamic workflow spawning, control hierarchies, and supervision - inspired by Erlang/OTP but adapted for Ash's capability-safety and governance model.

## Core Concepts

### 1. Workflow Addresses (WFID)

Every workflow instance has a unique, unforgeable address:

```
WFID = { 
    id: UUID,           -- Unique instance ID
    node: NodeId,       -- Runtime node (for distribution)
    generation: Int     -- For restarts
}
```

Addresses are **capabilities** - you can only send messages to workflows whose address you possess.

### 2. Spawning Workflows

```ash
-- Spawn a new workflow instance
spawn child_worker {
    workflow: worker_module,      -- Workflow definition to spawn
    init: { id: 123, config: {} }, -- Initial arguments
    links: [parent],               -- Control links to establish
    monitors: [supervisor_pid]     -- Monitoring relationships
} as child_ref;

-- child_ref is now a capability that allows:
-- - Sending messages to the child
-- - Checking if child is alive
-- - Terminating the child (if link permits)
```

### 3. Address Passing (π-calculus Inspired)

```ash
-- Receive an address via message
receive wait {
    worker:address as worker_ref => {
        -- Now we can send messages to this worker
        send worker:task with { work: "process_this" };
    }
}

-- Forward an address to another workflow
send other_workflow:delegate with {
    worker: worker_ref,  -- Passing the capability
    priority: "high"
};
```

**Key property**: Addresses are **linear capabilities** - transferring them removes them from the sender's possession (unless explicitly cloned via a `Shareable` trait).

### 4. Enhanced Mailbox with Addressing

```rust
struct Message {
    to: WFID,           -- Destination address
    from: WFID,         -- Source address (automatically set)
    payload: Value,     -- Message content
    kind: MessageKind,  -- Regular, Control, or Urgent
    timestamp: Instant,
    correlation_id: Option<UUID>,  -- For request/response patterns
}
```

### 5. Control Links and Guarantees

Control links provide specific guarantees:

```ash
-- Establish a control link with specified properties
link child_ref with {
    -- Fault detection
    heartbeat_interval: 5s,       -- Expected heartbeat frequency
    heartbeat_timeout: 15s,       -- Time before considering stuck
    
    -- Lifecycle notifications
    notify_on: [exit, error, stuck, restart],
    
    -- Message guarantees
    control_message_delivery: guaranteed,  -- Control messages are reliable
    regular_message_delivery: best_effort, -- Regular messages may drop
    
    -- Termination
    trap_exit: false,             -- If true, child exits don't kill parent
    graceful_shutdown_timeout: 30s -- Time to clean up before SIGKILL
};
```

### 6. Control Messages

Special message type that bypasses normal mailbox limits:

```ash
-- Send control message (always deliverable, even if mailbox full)
send_control child_ref with {
    command: "graceful_shutdown",
    deadline: now() + 30s
};

-- Or via receive pattern
receive control {
    "graceful_shutdown" with deadline: d => {
        perform_cleanup();
        exit(normal);
    },
    "heartbeat_check" => {
        send_control parent with "heartbeat_reply";
    },
    "inspect_state" => {
        send_control parent with { state: current_state() };
    }
}
```

## Supervision Hierarchies

### Supervisor Workflow Example

```ash
-- A supervisor that manages worker workflows
workflow worker_supervisor {
    -- Supervisor configuration
    let strategy = $input.strategy;  -- one_for_one, one_for_all, rest_for_one
    let max_restarts = $input.max_restarts or 5;
    let max_time = $input.max_time or 60s;
    
    -- Track children
    let children = [];
    
    -- Spawn initial workers
    for spec in $input.worker_specs do {
        spawn worker with {
            workflow: spec.module,
            init: spec.args,
            links: [self]  -- Link to supervisor
        } as worker_ref;
        
        monitor worker_ref;  -- Receive notifications about this worker
        let children = children + [{ ref: worker_ref, spec: spec }];
    };
    
    -- Supervision loop
    loop {
        receive control wait 30s {
            -- Child exited normally
            { from: child, event: "exit", reason: normal } => {
                act log with "Child exited normally: " + child;
                let children = remove_child(children, child);
                
                -- Maybe restart based on strategy
                if strategy == "permanent" then {
                    restart_child(child, children);
                }
            },
            
            -- Child crashed
            { from: child, event: "exit", reason: error, details: e } => {
                act log with "Child crashed: " + child + " error: " + e;
                decide { restart_count(child) < max_restarts } then {
                    restart_child(child, children);
                } else {
                    -- Too many restarts, escalate
                    exit({ error: "max_restarts_exceeded", child: child });
                }
            },
            
            -- Child appears stuck (no heartbeat)
            { from: child, event: "stuck", duration: d } => {
                act log with "Child stuck: " + child + " for " + d;
                -- Attempt graceful termination first
                send_control child with "graceful_shutdown";
                
                receive wait 5s {
                    { from: child, event: "exit" } => {
                        restart_child(child, children);
                    },
                    _ => {
                        -- Force kill
                        kill child;
                        restart_child(child, children);
                    }
                }
            },
            
            -- System shutdown request
            "shutdown" => {
                -- Graceful shutdown of all children
                for child in children do {
                    send_control child.ref with "graceful_shutdown";
                };
                
                -- Wait for all to exit (with timeout)
                wait_for_all_exits(children, 30s);
                exit(normal);
            },
            
            -- Scale up request
            { command: "scale_up", count: n } => {
                repeat n times {
                    spawn worker with {
                        workflow: default_worker_module,
                        init: default_args,
                        links: [self]
                    } as new_worker;
                    monitor new_worker;
                    let children = children + [{ ref: new_worker }];
                }
            },
            
            -- Timeout - check all children healthy
            _ => {
                check_all_children_healthy(children);
            }
        }
    }
}
```

### Worker Workflow Example

```ash
workflow worker {
    -- Initialization
    let config = $input;
    let state = initialize(config);
    
    -- Link to parent (supervisor)
    link $parent with {
        heartbeat_interval: 5s,
        notify_on: [exit, stuck]
    };
    
    -- Start heartbeat
    spawn heartbeat_sender with {
        workflow: heartbeat_loop,
        init: { parent: $parent, interval: 5s }
    } as heartbeat;
    
    -- Main processing loop
    loop {
        receive wait 30s {
            -- Regular work message
            { from: client, payload: work_item, correlation_id: cid } => {
                -- Process work
                orient process_work(work_item, state) as result;
                
                -- Reply to client
                send client:reply with {
                    correlation_id: cid,
                    result: result
                };
                
                -- Update state
                let state = update_state(state, work_item);
            },
            
            -- Graceful shutdown
            "graceful_shutdown" => {
                -- Stop accepting new work
                act log with "Shutting down gracefully...";
                
                -- Finish current work if any
                finish_in_progress(state);
                
                -- Cleanup resources
                cleanup(state);
                
                -- Notify supervisor
                exit(normal);
            },
            
            -- State inspection (for debugging)
            "inspect_state" => {
                send_control $parent with {
                    state_summary: summarize(state),
                    queue_depth: queue_length()
                };
            },
            
            -- Timeout - continue heartbeat, maybe do idle work
            _ => {
                perform_idle_tasks(state);
            }
        }
    }
}
```

## Runtime Guarantees

### 1. Stuck Workflow Detection

```rust
// Runtime monitors each workflow
struct WorkflowMonitor {
    last_heartbeat: Instant,
    last_progress: Instant,  -- Did the workflow make any state change?
    current_mailbox_size: usize,
    mailbox_growth_rate: f64,  -- Is mailbox growing unboundedly?
}

// Detection heuristics
enum StuckCondition {
    NoHeartbeat { expected: Duration, actual: Duration },
    NoProgress { duration: Duration },
    MailboxFull,
    MailboxGrowing { rate: f64 },  -- Potential memory leak
    CpuBound { cpu_time: Duration },  -- Possible infinite loop
}
```

### 2. Guaranteed Control Message Delivery

Control messages use a separate, prioritized channel:

```rust
struct WorkflowMailbox {
    regular_queue: VecDeque<Message>,      -- Bounded, may drop oldest
    control_queue: VecDeque<Message>,      -- Unbounded, never drops
    urgent_queue: VecDeque<Message>,       -- Processed before regular
}

// Always possible to send control messages
fn send_control(target: WFID, msg: Message) -> Result<(), SendError> {
    // Bypasses normal mailbox limits
    // Uses separate memory pool
    // Guaranteed delivery or explicit error
}
```

### 3. Lifecycle Event Notifications

```ash
-- Supervisor receives these events automatically
enum LifecycleEvent {
    Spawned { child: WFID, at: Timestamp },
    Linked { child: WFID, link_type: LinkType },
    Unlinked { child: WFID, reason: UnlinkReason },
    
    -- Progress events
    Progress { child: WFID, milestone: String },
    Heartbeat { child: WFID, sequence: Int },
    
    -- Issue events
    Stuck { child: WFID, condition: StuckCondition },
    MailboxFull { child: WFID, dropped_count: Int },
    HighMemory { child: WFID, usage_bytes: Int },
    
    -- Terminal events
    Exited { child: WFID, reason: ExitReason },
    Crashed { child: WFID, error: ErrorInfo },
    Killed { child: WFID, killer: WFID, reason: String },
}
```

## Capability-Passing Style

Addresses are first-class capabilities that can be passed around:

```ash
-- Create a worker pool
workflow pool_manager {
    let workers = [];
    
    -- Spawn workers
    for i in 0..$input.pool_size do {
        spawn worker with { id: i } as w;
        let workers = workers + [w];
    };
    
    -- Hand out worker addresses to clients
    loop {
        receive {
            { from: client, request: "get_worker" } => {
                -- Select worker (round-robin, least-loaded, etc.)
                let worker = select_worker(workers);
                
                -- Send the address to the client
                -- This is a capability transfer!
                send client:worker_assigned with {
                    worker: worker,
                    lease_duration: 60s  -- Client can use it for 60 seconds
                };
            }
        }
    }
}

-- Client using assigned worker
workflow client {
    send pool_manager:request with { request: "get_worker" };
    
    receive wait 10s {
        { worker: assigned_worker, lease_duration: dur } => {
            -- Now we can directly send work to this worker
            send assigned_worker:task with { work: "process_data" };
            
            -- Or forward it to another workflow
            send delegate:process with {
                worker: assigned_worker,  -- Capability passing
                data: load_data()
            };
            
            -- Or even return it (if this is a function-like workflow)
            ret { worker_ref: assigned_worker };
        }
    }
}
```

## Comparison with Erlang/OTP

| Feature | Erlang/OTP | Ash (Proposed) |
|---------|-----------|----------------|
| Process ID | Pid | WFID |
| Spawning | `spawn(Mod, Fun, Args)` | `spawn workflow with {...}` |
| Linking | `link(Pid)` | `link ref with {options}` |
| Monitoring | `monitor(process, Pid)` | `monitor ref` |
| Message send | `Pid ! Msg` | `send ref:channel with Msg` |
| Receive | `receive ... end` | `receive { ... }` |
| Control messages | Exit signals | Typed control messages |
| Supervision | supervisor behavior | `workflow supervisor` pattern |
| Address passing | Pids are values | Addresses are capabilities |
| Guarantees | "Let it crash" | Capability-safe + governance |

## Governance Integration

Supervision integrates with Ash's policy system:

```ash
policy supervisor_authority:
    when kill_workflow and actor.role != supervisor
    then deny
    else permit

policy max_workers:
    when spawn and current_workers >= 100
    then require_approval(role: ops)
    else permit

workflow critical_supervisor {
    -- All supervision actions are logged
    oblige ops audit_all_actions {
        log_level: detailed,
        retention: 90_days
    };
    
    -- Restarts require authorization after threshold
    decide { restart_count > 3 } then {
        act notify with { 
            to: ops_team, 
            message: "Frequent restarts detected" 
        };
    };
}
```

## Implementation Notes

### Required Runtime Features

1. **Workflow Registry**: Global (per-node) registry of active workflows
2. **Address Allocator**: Generates unique, unforgeable WFIDs
3. **Mailbox Manager**: Separate queues for regular/control/urgent messages
4. **Monitor Service**: Detects stuck workflows via heartbeats/progress checks
5. **Link Manager**: Tracks control links, delivers notifications
6. **Supervision Runtime**: Handles restart strategies

### Memory Model

```rust
// Each workflow has
struct WorkflowInstance {
    id: WFID,
    definition: WorkflowDef,
    state: WorkflowState,
    mailbox: Mailbox,
    links: Vec<Link>,
    monitors: Vec<Monitor>,
    parent: Option<WFID>,  -- Supervisor
    children: Vec<WFID>,   -- Supervised workflows
}
```

This design enables building reliable, distributed systems in Ash while maintaining capability safety and governance.
