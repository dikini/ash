# Workflow Instances and Structured Coordination

Ash workflows can be instantiated and coordinated dynamically. This document describes the minimal mechanisms for spawning, addressing, and supervising workflow instances.

> **Key principle**: Separate **communication** (`InstanceAddr`) from **supervision** (`ControlLink`). The control link is wrapped in `Option` and becomes `None` after transfer.

---

## 1. Two Handles, Different Semantics

When you spawn an instance, you get **two separate handles**:

```rust
pub struct SpawnResult<W> {
    /// Send signals/messages to the instance
    pub address: InstanceAddr<W>,
    
    /// Supervise the instance (lifecycle control)
    /// Wrapped in Option - becomes None after transfer
    pub control: Option<ControlLink<W>>,
}
```

### InstanceAddr<W>
- **Purpose**: Send application signals to the instance
- **Type**: Opaque handle
- **Lifespan**: Valid until instance completes
- **Operations**: `signal()`, `is_complete()`, `await_completion()`

### Option<ControlLink<W>>
- **Purpose**: Supervise the instance (shutdown, health checks)
- **Type**: `Some(link)` initially, `None` after transfer
- **Constraint**: **At most one** `Some` exists at any time (1-1)
- **Transfer**: Sending consumes the `Some`, becomes `None`

---

## 2. Spawning and the Split Operation

### Basic Spawn

```ash
spawn process_order with { order_id: 123 } as order;

-- 'order' has both address and control
-- Type: SpawnResult<process_order>
--   order.address: InstanceAddr<process_order>
--   order.control: Option<ControlLink<process_order>> = Some(link)
```

### Explicit Split (Recommended)

```ash
spawn background_worker with { config: cfg } as worker;
let (w_addr, w_ctrl) = split worker;

-- w_addr: InstanceAddr<background_worker>
-- w_ctrl: Option<ControlLink<background_worker>> = Some(link)
```

### Transferring Control

```ash
workflow parent {
    spawn worker with { id: 1 } as w;
    let (w_addr, w_ctrl) = split w;
    
    -- Check if we have control
    if w_ctrl.is_some() then {
        -- Transfer to supervisor
        send_control supervisor with w_ctrl;
        
        -- After transfer, w_ctrl is None
        assert w_ctrl.is_none();
    }
    
    -- I can still communicate
    signal w_addr with { work: "task-1" };
    signal w_addr with { work: "task-2" };
    
    -- Cannot control anymore (type-safe)
    -- send_control w_ctrl with shutdown;  
    -- ERROR: w_ctrl is Option<ControlLink>, not ControlLink
}
```

### Receiving Control

```ash
workflow supervisor {
    receive {
        { take_control: ctrl } => {
            -- ctrl: Option<ControlLink<W>>
            -- It's Some(link) when received
            
            if ctrl.is_some() then {
                -- Now I supervise this worker
                send_control ctrl with checkin;
                
                -- I can transfer again
                send_control another_supervisor with ctrl;
                -- Now ctrl is None
            }
        }
    }
}
```

---

## 3. Control Link Guarantees

### 1-1 Property

At any moment, **at most one** party has `Some(ControlLink)`:

```
Time 0: [Parent] Some(link)  [Supervisor] None
            |
            | send_control supervisor with w_ctrl
            v
Time 1: [Parent] None        [Supervisor] Some(link)
```

### What This Enables

```ash
workflow load_balancer {
    let backend_servers = [];
    
    -- Spawn backends, keep control, give addresses to clients
    for i in 0..3 do {
        spawn backend_server with { id: i } as backend;
        let (addr, ctrl) = split backend;
        
        -- Store control for health checking
        let backend_servers = backend_servers + [{
            id: i,
            control: ctrl,  -- Option<ControlLink>
            address: addr
        }];
        
        -- Advertise address to clients (via service discovery)
        advertise_service("api", addr);  -- Only address, not control
    };
    
    -- Health check loop (I have all the control links)
    loop {
        for server in backend_servers do {
            -- Only check if we still have control
            if server.control.is_some() then {
                send_control server.control with checkin;
                
                receive control wait 5s {
                    { from: server.control, status: healthy } => {
                        mark_healthy(server.id);
                    },
                    _ => {
                        -- Unhealthy - restart
                        restart_server(server);
                    }
                }
            }
        }
    }
}
```

**Key**: Clients have addresses (can send work), but only load_balancer has control links (can restart, shutdown).

---

## 4. Control Link Operations

### Available Methods

```rust
impl<W> ControlLink<W> {
    /// Request graceful shutdown
    pub fn request_shutdown(&self, deadline: Duration) -> Result<(), LinkError>;
    
    /// Force terminate immediately
    pub fn force_terminate(&self) -> Result<(), LinkError>;
    
    /// Suspend processing
    pub fn suspend(&self) -> Result<(), LinkError>;
    
    /// Resume processing
    pub fn resume(&self) -> Result<(), LinkError>;
    
    /// Wait for instance to exit
    pub async fn await_exit(&self) -> ExitReason;
}

-- Must check Option first
if w_ctrl.is_some() then {
    send_control w_ctrl with shutdown deadline: 30s;
}
```

### In Ash Syntax

```ash
-- Shutdown gracefully (if we have control)
if w_ctrl.is_some() then {
    send_control w_ctrl with shutdown deadline: 30s;
}

-- Force kill (if we have control)
if w_ctrl.is_some() then {
    terminate w_ctrl;
}

-- Check if responsive
if w_ctrl.is_some() then {
    send_control w_ctrl with checkin;
    receive control wait 5s {
        { from: w_ctrl } => { /* healthy */ },
        _ => { /* dead */ }
    }
}

-- Wait for natural completion (if we have control)
if w_ctrl.is_some() then {
    let reason = await_exit w_ctrl;
}
```

---

## 5. InstanceAddr Operations

```rust
impl<W> InstanceAddr<W> {
    /// Send signal to instance mailbox
    pub fn signal(&self, msg: Signal) -> Result<(), SignalError>;
    
    /// Check if instance has completed
    pub fn is_complete(&self) -> bool;
    
    /// Wait for completion, get return value
    pub async fn await_completion(self) -> AwaitResult<W::Output>;
}
```

### In Ash Syntax

```ash
-- Send work (can do this any time, as long as instance lives)
signal worker_addr with { task: "process", data: records };

-- Wait for result (consumes the address)
let result = await worker_addr timeout: 60s;
-- After await, worker_addr is consumed (cannot use again)

-- Check status without blocking
if worker_addr.is_complete() then {
    act log with "Worker already done";
}
```

---

## 6. Complete Example: Worker Pool with Supervision

```ash
-- main.ash: Entry point
workflow main {
    -- Start supervisor
    spawn pool_supervisor with { 
        min_workers: 2,
        max_workers: 10,
        target_queue_depth: 5
    } as supervisor;
    
    -- Get handle to submit work (address only, no control)
    let (supervisor_addr, _) = split supervisor;
    
    -- Submit work
    for task in load_tasks() do {
        signal supervisor_addr with { submit: task };
    }
    
    -- Wait for all work to complete
    signal supervisor_addr with { await_completion: true };
    let summary = await supervisor_addr;
    
    ret summary;
}

-- pool_supervisor.ash: Manages workers
workflow pool_supervisor {
    let config = $input;
    let workers = [];  -- Each has address + Option<control>
    let pending_work = [];
    
    -- Initialize minimum workers
    for i in 0..config.min_workers do {
        spawn worker with { id: i } as w;
        let (addr, ctrl) = split w;
        
        -- Store both
        let workers = workers + [{ 
            id: i, 
            address: addr, 
            control: ctrl,  -- Some(link)
            busy: false 
        }];
    };
    
    -- Main loop
    loop {
        receive {
            -- New work submitted
            { submit: task } => {
                let available = find_available(workers);
                
                decide { available.found } then {
                    signal available.worker.address with { work: task };
                    mark_busy(workers, available.worker.id);
                } else {
                    -- Queue work
                    let pending_work = pending_work + [task];
                    
                    -- Scale up if needed
                    decide { 
                        len(pending_work) > config.target_queue_depth and
                        len(workers) < config.max_workers 
                    } then {
                        spawn worker with { id: len(workers) } as new_w;
                        let (new_addr, new_ctrl) = split new_w;
                        let workers = workers + [{ 
                            id: len(workers), 
                            address: new_addr, 
                            control: new_ctrl,  -- Some(link)
                            busy: false 
                        }];
                    }
                }
            },
            
            -- Work completed
            { from: worker_addr, completed: result } => {
                let worker = find_by_address(workers, worker_addr);
                mark_available(workers, worker.id);
                
                -- Assign pending work if any
                decide { len(pending_work) > 0 } then {
                    let next_task = pop(pending_work);
                    signal worker.address with { work: next_task };
                    mark_busy(workers, worker.id);
                }
            },
            
            -- Health check via control link
            receive control wait 30s {
                -- Check each worker we control
                for w in workers do {
                    if w.control.is_some() then {
                        send_control w.control with checkin;
                        
                        receive control wait 5s {
                            { from: w.control } => { /* healthy */ },
                            _ => {
                                -- Stuck - restart
                                restart_worker(w);
                            }
                        }
                    }
                }
            }
        }
    }
}

-- worker.ash: Does the actual work
workflow worker {
    let id = $input.id;
    
    loop {
        receive {
            { work: task } => {
                -- Process
                let result = process(task);
                
                -- Report completion
                signal parent with { 
                    from: self,  -- InstanceAddr of this worker
                    completed: result 
                };
            }
        };
        
        -- Periodic health check via control channel
        receive control {
            checkin => {
                send_control parent with { 
                    status: healthy,
                    queue_depth: mailbox_length()
                };
            }
        }
    }
}
```

---

## 7. Type Checking

### Spawn

```
Γ ⊢ spawn W with { args } as x : SpawnResult<W>
-----------------------------------------------
Γ, x: SpawnResult<W> ⊢ rest
```

### Split

```
Γ ⊢ x: SpawnResult<W>
let (a, c) = split x;
-----------------------------------------------
Γ, a: InstanceAddr<W>, c: Option<ControlLink<W>> ⊢ rest
```

### Signal (Non-consuming)

```
Γ ⊢ addr: InstanceAddr<W>
Γ ⊢ payload: T where T: Message<W>
-----------------------------------------------
signal addr with payload : ()
Γ ⊢ addr: InstanceAddr<W>  -- NOT consumed
```

### Await (Consuming)

```
Γ ⊢ addr: InstanceAddr<W>
-----------------------------------------------
await addr : AwaitResult<W::Output>
Γ ⊢ addr: consumed  -- Cannot use again
```

### Send Control (Transfer)

```
Γ ⊢ target: InstanceAddr<Supervisor>
Γ ⊢ ctrl: Option<ControlLink<W>> = Some(link)
-----------------------------------------------
send_control target with ctrl : ()
Γ ⊢ ctrl: None  -- Transferred away
```

### Check Option

```
Γ ⊢ ctrl: Option<ControlLink<W>>
-----------------------------------------------
if ctrl.is_some() then {
    -- In this branch: ctrl: ControlLink<W>
    send_control ctrl with msg;  -- OK
} else {
    -- In this branch: ctrl: None
}
```

---

## 8. Error Messages

### Using None Control

```
error[E001]: expected ControlLink<worker>, found None
  --> example.ash:15:5
   |
10 |     send_control supervisor with w_ctrl;
   |                               ------- control transferred here
...
15 |     send_control w_ctrl with shutdown;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected ControlLink, found None
   |
   = help: control link was transferred to supervisor on line 10
   = note: check `w_ctrl.is_some()` before using
```

### Missing Option Check

```
error[E002]: cannot use Option<ControlLink> directly
  --> example.ash:12:5
   |
11 |     let (_, w_ctrl) = split w;
   |              ------ has type Option<ControlLink<worker>>
12 |     send_control w_ctrl with msg;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected ControlLink, found Option<ControlLink>
   |
   = help: check if control is available first:
   |
   |     if w_ctrl.is_some() then {
   |         send_control w_ctrl with msg;
   |     }
```

---

## 9. Comparison with Other Models

| Feature | Erlang/OTP | Rust (Tokio) | **Ash (Proposed)** |
|---------|-----------|--------------|-------------------|
| Spawn | `spawn` | `tokio::spawn` | `spawn with` |
| Communication | `!` (copyable Pid) | Channel send | `signal` (`InstanceAddr`) |
| Supervision | OTP supervisor | Manual | `Option<ControlLink>` |
| Separation | Link + monitor | None explicit | **Addr vs Option<Control>** |
| Delegation | Supervisor tree | N/A | **Explicit transfer (becomes None)** |
| Check ownership | Runtime | N/A | **Compile-time + runtime (is_some)** |

---

## 10. Governance Integration

```ash
policy control_link_transfer:
    when send_control and receiver.role != supervisor
    then require_approval(role: ops)
    else permit

policy instance_limit_per_holder:
    when hold_control and controlled_count > 10
    then require_approval(role: ops)
    else permit

workflow audited_pool {
    -- All control link transfers are logged
    oblige governance audit_control_transfers;
    
    spawn worker with {} as w;
    let (addr, ctrl) = split w;
    
    if ctrl.is_some() then {
        -- Transfer logged: who, what, when, to whom
        send_control supervisor with ctrl;
    }
}
```

---

## Summary

| Handle | Type | Initial Value | After Transfer | Use For |
|--------|------|---------------|----------------|---------|
| `InstanceAddr` | Opaque handle | Valid address | Still valid (until await) | Send signals |
| `ControlLink` | `Option<ControlLink>` | `Some(link)` | `None` | Supervise |

Key properties:
1. **Explicit split**: `let (addr, ctrl) = split w`
2. **Option wrapper**: `ctrl: Option<ControlLink<W>>`
3. **Transfer consumes**: `send_control target with ctrl` → `ctrl` becomes `None`
4. **Type-safe**: Must check `is_some()` before use
5. **1-1 invariant**: At most one `Some(ControlLink)` exists at any time
