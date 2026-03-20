# Semantics of `await`

What does `await` mean for workflow instances? This document clarifies the type and semantics.

## Core Question

When we have:
```ash
spawn worker with { config } as w;
let (addr, ctl) = split w;

-- What is the type of `result`?
-- What does await consume?
-- What about errors/timeouts?
let result = await addr;
```

---

## 1. What `await` Waits For

`await addr` waits for the **workflow instance to complete**:
- Reaches `ret value` (normal completion)
- Reaches `done` (returns null)
- Crashes/panics (error completion)
- Is killed/terminated (forced completion)

```ash
workflow worker {
    let input = $input;
    let result = do_work(input);
    ret { success: true, data: result }  -- await returns this value
}

workflow main {
    spawn worker with { config } as w;
    let (addr, _) = split w;
    
    -- Blocks here until worker reaches `ret` or `done`
    let result = await addr;  -- result: { success: Bool, data: Value }
}
```

---

## 2. Type Signature

```rust
// If workflow W has return type T
impl InstanceAddr<W> {
    async fn await(self, timeout: Option<Duration>) -> AwaitResult<W::Output>;
}

// Result type
enum AwaitResult<T> {
    Completed(T),           -- Normal completion with value
    Failed(ExitError),      -- Instance crashed/panicked
    Cancelled(String),      -- Instance was killed/cancelled
    Timeout,                -- Timeout reached
    AlreadyConsumed,        -- Address was already awaited
}
```

### In Ash Type Syntax

```ash
-- Given: workflow worker returns Result<T>
spawn worker with {} as w;
let (addr, _) = split w;

-- Type of result: AwaitResult<Result<T>>
let result = await addr timeout: 30s;

decide { result.completed } then {
    let value = result.value;  -- Type: Result<T>
} else {
    -- Handle error
    ret { error: result.reason }
}
```

---

## 3. What `await` Consumes

**`await` consumes the `InstanceAddr`** (affine move):

```ash
spawn worker with {} as w;
let (addr, ctl) = split w;

-- First await works
let result1 = await addr;  -- addr is now moved/consumed

-- ERROR: Second await fails
let result2 = await addr;  -- Type error: addr already consumed
```

**Why?** After the instance completes, there's nothing left to await. Using the address again would be meaningless.

---

## 4. What About the ControlLink?

**`await` does NOT consume the `ControlLink`**:

```ash
spawn worker with {} as w;
let (addr, ctl) = split w;

-- Await completion
let result = await addr;  -- addr consumed

-- Can still use control link? 
-- Depends on instance state:
send_control ctl with checkin;  -- May return "already exited"
```

### ControlLink After Instance Completes

Once an instance completes, the `ControlLink` becomes mostly useless:

```rust
impl ControlLink<W> {
    /// After instance exits, returns exit reason immediately
    fn check_status(&self) -> ExitStatus;
    
    /// No-op if already exited
    fn request_shutdown(&self, deadline: Duration) -> Result<(), LinkError>;
}
```

**Current direction**: `await` does not consume `ControlLink`.

Instead, the link remains reusable supervision authority while the runtime can still answer control
queries for the target instance. After completion, non-terminal queries such as status/health may
return an "already exited" style result, while terminal control requests are rejected explicitly.

---

## 5. Comparison: `await addr` vs `await_exit ctl`

Two different ways to wait:

### Await Address (Get Return Value)
```ash
let (addr, ctl) = split w;

-- Wait for completion, get return value
let result = await addr;  -- AwaitResult<T>

-- Use the returned value
decide { result.completed } then {
    process(result.value);
}
```

### Await Control (Supervision)
```ash
let (addr, ctl) = split w;

-- Can still send work via addr while waiting
signal addr with { work: 1 };
signal addr with { work: 2 };

-- Wait for exit via control link (no return value)
let exit_reason = await_exit ctl;  -- ExitReason

-- ExitReason includes:
-- - Normal (ret value)
-- - Error (panic/crash)
-- - Cancelled (forced shutdown)
```

**Key difference**:
- `await addr`: Consumes addr, returns workflow's return value
- `await_exit ctl`: Consumes ctl, returns exit reason (no business value)

---

## 6. Timeout Semantics

### Soft Timeout (Default)

```ash
let result = await addr timeout: 30s;

-- If timeout:
-- result = Timeout
-- Instance continues running!
-- addr is consumed (can't await again)
```

**Problem**: What if we want to try again later?

### Hard Timeout (With Cancellation)

```ash
let result = await addr timeout: 30s with cancel;

-- If timeout:
-- 1. Send cancel signal to instance
-- 2. Wait for graceful shutdown
-- 3. If still running, force kill
-- 4. result = Cancelled("timeout")
```

### Using Control Link for Timeout

Better pattern: use control link to manage timeout:

```ash
workflow main_with_timeout {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    
    -- Race: await completion vs timeout
    par {
        -- Branch 1: Wait for completion
        let result = await addr;
        ret { completed: true, value: result }
    } {
        -- Branch 2: Timeout
        sleep 30s;
        send_control ctl with cancel;
        let exit = await_exit ctl;
        ret { completed: false, reason: "timeout" }
    };
}
```

---

## 7. Error Handling

### Instance Panics/Crashes

```ash
spawn risky_worker with {} as w;
let (addr, _) = split w;

let result = await addr;

decide { 
    result.completed or 
    result.cancelled 
} then {
    -- Normal or cancelled - both "expected" outcomes
} else {
    -- Instance crashed unexpectedly
    act alert with { 
        severity: "error",
        message: "Worker crashed: " + result.error 
    };
}
```

### Type Safety

```ash
-- Type error: trying to use failed result
let result = await addr;
let value = result;  -- Type: AwaitResult<T>, not T

-- Must check
let value = if result.completed then result.value else default;
```

---

## 8. Multiple Awaits (Error)

```ash
workflow bad_example {
    spawn worker with {} as w;
    let (addr, _) = split w;
    
    par {
        -- Two parallel awaits on same address - ERROR
        let r1 = await addr,
        let r2 = await addr
    };
}
```

**Type error**: `addr` is affine, cannot be used in two places.

### Correct Pattern: Single Await

```ash
workflow good_example {
    spawn worker with {} as w;
    let (addr, _) = split w;
    
    -- Single await
    let result = await addr;
    
    -- Share the result, not the address
    par {
        process_result_a(result),
        process_result_b(result)
    };
}
```

---

## 9. Await Without Spawn?

Can we await something that isn't a spawned instance?

### Proposal: Await Any Async Capability

```ash
-- Await a future returned by capability
capability fetch_data : observe(id: String) returns Future<Data>;

workflow main {
    let data_future = observe fetch_data with id: "123";
    
    -- Await the future
    let data = await data_future;
}
```

**But this blurs the line**: Is `await` for instances only, or any async operation?

### Recommendation: Instances Only (For Now)

```ash
-- Valid
spawn workflow with {} as w;
await w;  -- OK

-- Not valid (for now)
await some_capability_result;  -- Error
```

Keep `await` specific to workflow instances for clarity.

---

## 10. Summary Table

| Expression | Waits For | Returns | Consumes |
|------------|-----------|---------|----------|
| `await addr` | Instance completion | `AwaitResult<T>` | `InstanceAddr` |
| `await_exit ctl` | Instance exit | `ExitReason` | implementation-defined by later control contract |
| `await addr timeout: t` | Completion or timeout | `AwaitResult<T>` | `InstanceAddr` |

### Type Signatures

```rust
// InstanceAddr<W> where W returns T
fn await(self, timeout: Option<Duration>) -> AwaitResult<T>;

// ControlLink<W>
fn await_exit(&self) -> ExitReason;
```

### AwaitResult<T>

```rust
enum AwaitResult<T> {
    Completed(T),
    Failed(ExitError),
    Cancelled(String),
    Timeout,
}

impl<T> AwaitResult<T> {
    fn completed(&self) -> bool { matches!(self, Completed(_)) }
    fn value(self) -> Option<T> { /* extract if completed */ }
}
```

---

## 11. Open Questions

1. **How much post-exit status remains observable through `ControlLink`?**

2. **What about `await` on SpawnResult before split?**
   ```ash
   spawn worker with {} as w;
   await w;  -- Implicitly splits and awaits both?
   ```

3. **Should we support `await` in `receive` arms?**
   ```ash
   receive {
       { should_wait } => { await addr; }  -- Valid?
   }
   ```
   Yes, but need branch join checking.

4. **What if instance never completes (infinite loop)?**
   - `await` without timeout blocks forever
   - Need timeout or external cancellation

5. **Can we `await` a completed instance?**
   - If we kept the address: type error (already consumed)
   - If we got address from elsewhere: immediate return with cached result?
