# Control Link Transfer: Mutation-Based Model

A simpler approach: `ControlLink` is like `Option<ControlLink>` - present until transferred away.

## Core Idea

```rust
// After spawn, we have:
let (w_addr, w_ctrl) = spawn worker with {};

// w_addr: InstanceAddr<Worker>     -- Opaque, usable, keepable
// w_ctrl: Option<ControlLink<Worker>>  -- Some(link) initially
```

When we transfer control, `w_ctrl` becomes `None`:

```ash
send_control supervisor with w_ctrl;
-- After this: w_ctrl == None
```

## Type System

### InstanceAddr<W>

```rust
pub struct InstanceAddr<W> {
    instance_id: InstanceId,
    _phantom: PhantomData<W>,
}

impl<W> InstanceAddr<W> {
    /// Send signal to instance mailbox
    pub fn signal(&self, msg: Msg<W>) -> Result<(), SignalError>;
    
    /// Check if instance has completed
    pub fn is_complete(&self) -> bool;
    
    /// Block until completion, get return value
    pub async fn await_completion(self) -> CompletionResult<W::Output>;
}
```

**Properties**:
- Opaque (can't inspect internal ID)
- Copyable/clonable (or at least usable multiple times)
- `await_completion` consumes it (can't await twice)

### ControlLink<W>

```rust
pub struct ControlLink<W> {
    instance_id: InstanceId,
    _phantom: PhantomData<W>,
}

// Actually wrapped in Option after spawn
type ControlHandle<W> = Option<ControlLink<W>>;

impl<W> ControlLink<W> {
    /// Send control signal
    pub fn send_control(&self, msg: ControlMsg) -> Result<(), LinkError>;
    
    /// Wait for instance exit
    pub async fn await_exit(self) -> ExitReason;
}
```

## Operations

### Spawn

```ash
spawn worker with { config: cfg } as w;
let (w_addr, w_ctrl) = split w;

-- Types:
-- w_addr: InstanceAddr<worker>
-- w_ctrl: Option<ControlLink<worker>> = Some(link)
```

### Signal (to address)

```ash
-- w_addr is unchanged after use
signal w_addr with { work: data };
signal w_addr with { more: work };  -- Can use multiple times
```

### Send Control (transfer)

```ash
-- Check if we have control
if w_ctrl.is_some() then {
    -- Transfer to supervisor
    send_control supervisor with w_ctrl;
    
    -- After transfer, w_ctrl is None
    assert w_ctrl.is_none();
}

-- Try to use control (error if None)
send_control w_ctrl with shutdown;  -- Type error: w_ctrl is Option<ControlLink>
```

### Take Control (receive)

```ash
workflow supervisor {
    receive {
        { take_control: ctrl } => {
            -- ctrl has type Option<ControlLink<W>>
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

## Pattern: Delegate Supervision

```ash
workflow parent {
    spawn worker with { id: 1 } as w;
    let (w_addr, w_ctrl) = split w;
    
    -- Keep address to send work
    -- Give away control to supervisor
    
    if w_ctrl.is_some() then {
        send_control supervisor with w_ctrl;
        -- w_ctrl is now None
    }
    
    -- Can still send work
    signal w_addr with { task: "process" };
    
    -- Cannot control anymore
    -- w_ctrl.send_control(shutdown) would fail type check
    -- (w_ctrl is Option<ControlLink>, not ControlLink)
}
```

## Pattern: Load Balancer

```ash
workflow load_balancer {
    let backends = [];
    
    -- Spawn backends
    for i in 0..3 do {
        spawn backend with { id: i } as b;
        let (addr, ctrl) = split b;
        
        -- Store both
        let backends = backends + [{
            address: addr,
            control: ctrl,  -- Option<ControlLink>
            healthy: true
        }];
    };
    
    -- Health check loop
    loop {
        for be in backends do {
            -- Only check if we still have control
            if be.control.is_some() then {
                send_control be.control with checkin;
                
                receive control wait 5s {
                    { from: be.control, healthy: true } => continue,
                    _ => {
                        -- No response, mark unhealthy
                        let be.healthy = false;
                        
                        -- Try to restart
                        restart_backend(be);
                    }
                }
            }
        }
    }
}
```

## Type Checking Rules

### 1. Send Control Transfers

```
Γ ⊢ x: Option<ControlLink<W>>
Γ ⊢ target: InstanceAddr<Supervisor>
-----------------------------------------------
send_control target with x;
Γ ⊢ x: None  -- x is now None
```

### 2. Cannot Use None

```
Γ ⊢ x: None
-----------------------------------------------
send_control x with msg;  -- TYPE ERROR
```

### 3. Must Check Before Use

```
Γ ⊢ x: Option<ControlLink<W>>
-----------------------------------------------
if x.is_some() then {
    -- In this branch, x: ControlLink<W>
    send_control x with msg;  -- OK
} else {
    -- In this branch, x: None
    send_control x with msg;  -- TYPE ERROR (dead code or error)
}
```

## Alternative Syntax: Explicit Take

Instead of `Option`, use explicit `take`:

```ash
spawn worker with {} as w;
let (w_addr, w_ctrl) = split w;

-- w_ctrl: ControlLink<W>

-- Transfer with take (moves and invalidates)
send_control supervisor with take w_ctrl;

-- After take, w_ctrl is invalidated
-- w_ctrl.send_control(...)  -- ERROR: w_ctrl moved

-- Can check if we have it
if has_control(w) then {  -- Check via original spawn handle?
    ...
}
```

But the `Option` model is cleaner:

```ash
if w_ctrl.is_some() then {
    send_control supervisor with w_ctrl;  -- Implicit take
}
```

## Comparison: Affine vs Option Models

| Aspect | Affine Types | Option<ControlLink> |
|--------|--------------|---------------------|
| Type complexity | High (linear logic) | Low (familiar Option) |
| Error messages | "value moved" | "expected ControlLink, found None" |
| Check if have control | Type-level | Runtime-level (is_some) |
| Transfer safety | Compile-time | Compile-time + runtime check |
| Learning curve | Steep | Gentle |

## Recommendation: Option<ControlLink>

The `Option<ControlLink>` model is **simpler** and **sufficient**:

1. **Type check**: Ensure `w_ctrl` is `Some` before use
2. **Transfer**: `send_control target with w_ctrl` consumes the `Some`
3. **Check**: `w_ctrl.is_some()` to test
4. **Error**: Clear error messages

```ash
workflow example {
    spawn worker with {} as w;
    let (w_addr, w_ctrl) = split w;
    
    -- Type: w_ctrl: Option<ControlLink<worker>>
    
    -- Can check
    if w_ctrl.is_some() then {
        act log with "I have control";
    }
    
    -- Transfer
    send_control supervisor with w_ctrl;
    -- Type: w_ctrl is now None
    
    -- Try to use (type error)
    -- send_control w_ctrl with msg;  -- ERROR
    
    -- Can still use address
    signal w_addr with { work: data };
}
```

## Implementation Notes

### Rust Implementation

```rust
// Runtime representation
pub struct InstanceHandle<W> {
    instance_id: InstanceId,
    address: InstanceAddr<W>,
    control: Arc<Mutex<Option<ControlLink<W>>>>,
}

impl<W> InstanceHandle<W> {
    pub fn split(self) -> (InstanceAddr<W>, Option<ControlLink<W>>) {
        let control = self.control.lock().unwrap().take();
        (self.address, control)
    }
}

// Transfer operation
pub fn transfer_control<W>(
    from: &Arc<Mutex<Option<ControlLink<W>>>>,
    to: &InstanceAddr<Supervisor>
) -> Result<(), TransferError> {
    let link = from.lock().unwrap().take()
        .ok_or(TransferError::AlreadyTransferred)?;
    
    // Send to supervisor
    to.signal(ControlTransfer { link })
}
```

### Type Checker

```rust
fn check_send_control(
    env: &mut TypeEnv,
    target: &Expr,
    control_var: &Expr,
) -> Result<(), TypeError> {
    // Check target is InstanceAddr
    let target_ty = type_check_expr(env, target)?;
    ensure_instance_addr(&target_ty)?;
    
    // Check control_var is Option<ControlLink>
    let ctrl_ty = type_check_expr(env, control_var)?;
    ensure_option_control_link(&ctrl_ty)?;
    
    // Mark control_var as None in environment
    env.update_type(control_var, Type::None);
    
    Ok(())
}
```

## Summary

| Concept | Representation |
|---------|---------------|
| Instance Address | `InstanceAddr<W>` - opaque, reusable |
| Control Handle | `Option<ControlLink<W>>` - Some until transferred |
| Transfer | `send_control target with ctrl` - ctrl becomes None |
| Check ownership | `ctrl.is_some()` |
| Use control | Must check `is_some()` first |

This is **simpler than affine types** but still **type-safe**.
