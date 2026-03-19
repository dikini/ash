# Type Checking Affine Workflow Instances

How to type check affine types (InstanceAddr, ControlLink) in Ash's async workflow language.

## Design Philosophy

**Keep it minimal**: No lifetimes, no borrowing, explicit moves only.

> Rationale: This is a workflow orchestration language, not systems programming. Simplicity over expressiveness.

---

## 1. Type Hierarchy

```rust
// Base kinds
kind Value;        // Regular values (Copy, Clone)
kind Affine;       // Linear/owned values (move only)

// Types
Type ::= 
    | Int | String | Bool | Null           // Value types
    | List<T> | Record<...>                // Compound value types
    | InstanceAddr<W>                      // Affine: address to workflow W
    | ControlLink<W>                       // Affine: control over workflow W
    | SpawnResult<W>                       // Affine bundle: { address, control }
    | ShareableAddr<W>                     // Value: cloneable address
    ;
```

### Key: Workflow-Parameterized Types

```rust
// Address to a "worker" workflow
InstanceAddr<worker>  

// Address to a "validator" workflow  
InstanceAddr<validator>

// These are different types!
```

This prevents sending wrong message types to wrong workflows.

---

## 2. Typing Rules (Simplified)

### Environment

```
Γ ::= ∅ | Γ, x: T @ L

L ::= available | moved | split(T1, T2)
```

Each variable has a **type** and a **linearity status**.

### Spawn

```
Γ ⊢ spawn W with { args } as x : SpawnResult<W>
-----------------------------------------------
Γ, x: SpawnResult<W> @ available ⊢ rest
```

### Split

```
Γ ⊢ x: SpawnResult<W> @ available
let (a, c) = split x;
-----------------------------------------------
Γ, x: SpawnResult<W> @ moved, 
   a: InstanceAddr<W> @ available, 
   c: ControlLink<W> @ available ⊢ rest
```

`split` consumes the `SpawnResult` and produces two new affine values.

### Signal (Send)

```
Γ ⊢ addr: InstanceAddr<W> @ available
Γ ⊢ payload: T where T: Message<W>
-----------------------------------------------
signal addr with payload : ()
Γ ⊢ addr: InstanceAddr<W> @ available  // NOT consumed!
```

**Key**: `signal` does NOT consume the address. It's like `&T` in Rust - you can use it multiple times.

### Await (Consume)

```
Γ ⊢ addr: InstanceAddr<W> @ available
-----------------------------------------------
await addr : Result<W::Output>
Γ ⊢ addr: InstanceAddr<W> @ moved  // CONSUMED
```

`await` consumes the address - you can't use it after.

### Transfer Control

```
Γ ⊢ ctl: ControlLink<W> @ available
-----------------------------------------------
signal target with { take_control: ctl }
Γ ⊢ ctl: ControlLink<W> @ moved  // CONSUMED - transferred!
```

Control links are consumed when sent to another instance.

---

## 3. The Challenge: Async and Receive

### Problem

```ash
workflow example {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    
    receive {
        -- In this branch, we use addr
        { work: x } => {
            signal addr with { task: x };  -- OK
        },
        
        -- In this branch, we also use addr
        { other: y } => {
            signal addr with { task: y };  -- OK
        }
    };
    
    -- Can we use addr here?
    signal addr with { more: work };  -- Is this OK?
}
```

### Solution: Branch Join Points

At the end of `receive`, all branches must agree on the **linearity state**.

```
Γ ⊢ receive {
    pat1 => Γ1 ⊢ stmt1,
    pat2 => Γ2 ⊢ stmt2
} : T
-----------------------------------------------
Γ' = join(Γ1, Γ2)
```

**Join rules**:
- If `x` is `available` in all branches → `available` in join
- If `x` is `moved` in all branches → `moved` in join
- If `x` is `available` in some, `moved` in others → **TYPE ERROR**

So the example above **fails** if one branch moves `addr` and another doesn't.

### Valid Example

```ash
workflow example {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    
    receive {
        -- Both branches use addr the same way (signal, don't consume)
        { work: x } => {
            signal addr with { task: x };
        },
        { other: y } => {
            signal addr with { task: y };
        }
    };
    
    -- OK: addr still available in both branches
    signal addr with { more: work };
}
```

### Invalid Example

```ash
workflow example {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    
    receive {
        { should_wait: true } => {
            let result = await addr;  -- CONSUMES addr
        },
        { should_wait: false } => {
            signal addr with { task: x };  -- addr still available
        }
    };
    
    -- ERROR: addr is moved in one branch, available in other
    signal addr with { more: work };
}
```

Fix: Move `await` outside, or structure differently.

---

## 4. Loop Invariants

### Problem

```ash
workflow looper {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    
    loop {
        signal addr with { work: next() };  -- Is addr available here?
    }
}
```

### Solution: Loop Preconditions

```
Γ ⊢ loop { body } : ()
-----------------------------------------------
Γ must be preserved by body
```

A variable must have the **same status** at the end of the loop body as at the start.

So the example is valid: `addr` is `available` before and after each iteration.

### Invalid Loop

```ash
workflow bad_looper {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    
    loop {
        signal addr with { work: next() };
        let result = await addr;  -- CONSUMES addr
        -- ERROR: addr is moved, but loop expects it available
    }
}
```

Fix:
```ash
workflow good_looper {
    loop {
        spawn worker with {} as w;  -- Fresh spawn each iteration
        let (addr, ctl) = split w;
        signal addr with { work: next() };
        let result = await addr;  -- OK: consumed but we don't need it again
        -- addr is moved, but we're at end of loop iteration
    }
}
```

---

## 5. Sharing (Explicit Clone)

### Shareable Types

```rust
trait Shareable where Self: Affine {
    fn share(self) -> (Self, Self);
}

// Some addresses are shareable
impl Shareable for InstanceAddr<service_worker> {}

// Control links are NEVER shareable
// impl Shareable for ControlLink<W> {}  // NOT IMPLEMENTED
```

### Type Checking Share

```
Γ ⊢ x: T @ available
T: Shareable
let (x1, x2) = share x;
-----------------------------------------------
Γ, x: T @ moved,
   x1: T @ available,
   x2: T @ available ⊢ rest
```

### Example

```ash
workflow router {
    spawn service_worker with {} as svc;
    let (addr, ctl) = split svc;
    
    -- Make address shareable
    let shareable_addr = promote_to_shareable(addr);  -- Type: ShareableAddr<service_worker>
    let (addr1, addr2) = share shareable_addr;
    
    -- Give to two different workflows
    signal handler1 with { service: addr1 };
    signal handler2 with { service: addr2 };
    
    -- I still have control
    send_control ctl with checkin;
}
```

---

## 6. Type Inference Algorithm

### Simplified Algorithm

```rust
fn type_check_workflow(workflow: &Workflow) -> Result<TypeEnv, TypeError> {
    let mut env = TypeEnv::new();
    
    for stmt in workflow.body {
        match stmt {
            Stmt::Spawn { workflow: w, args, as: name } => {
                // Infer arg types
                let arg_types = infer_args(&args, &env)?;
                // Check against workflow parameter types
                check_args(&arg_types, &w.params)?;
                // Add to environment
                env.insert(name, Type::SpawnResult(w.clone()), Status::Available);
            }
            
            Stmt::Split { source, into: (a, b) } => {
                let (ty, status) = env.lookup(source)?;
                match (ty, status) {
                    (Type::SpawnResult(w), Status::Available) => {
                        env.mark_moved(source);
                        env.insert(a, Type::InstanceAddr(w.clone()), Status::Available);
                        env.insert(b, Type::ControlLink(w.clone()), Status::Available);
                    }
                    _ => return Err(TypeError::CannotSplit { ty, status }),
                }
            }
            
            Stmt::Signal { target, payload } => {
                let (ty, status) = env.lookup(target)?;
                match (ty, status) {
                    (Type::InstanceAddr(w), Status::Available) => {
                        // Check payload matches workflow's message type
                        check_message_type(&payload, &w)?;
                        // addr NOT consumed
                    }
                    _ => return Err(TypeError::CannotSignal { ty, status }),
                }
            }
            
            Stmt::Await { target } => {
                let (ty, status) = env.lookup(target)?;
                match (ty, status) {
                    (Type::InstanceAddr(_), Status::Available) => {
                        env.mark_moved(target);  // CONSUMED
                    }
                    _ => return Err(TypeError::CannotAwait { ty, status }),
                }
            }
            
            Stmt::Receive { arms } => {
                // Check each arm
                let branch_envs: Vec<TypeEnv> = arms.iter()
                    .map(|arm| type_check_arm(arm, &env))
                    .collect::<Result<Vec<_>, _>>()?;
                
                // Join all branches
                env = join_environments(branch_envs)?;
            }
            
            Stmt::Loop { body } => {
                let initial_env = env.clone();
                let final_env = type_check_workflow(body)?;
                
                // Check invariant
                check_loop_invariant(&initial_env, &final_env)?;
            }
            
            // ... other statements
        }
    }
    
    Ok(env)
}

fn join_environments(envs: Vec<TypeEnv>) -> Result<TypeEnv, TypeError> {
    let mut result = TypeEnv::new();
    
    // Get all variables from all environments
    let all_vars: HashSet<String> = envs.iter()
        .flat_map(|e| e.vars.keys())
        .cloned()
        .collect();
    
    for var in all_vars {
        let statuses: Vec<Status> = envs.iter()
            .map(|e| e.get_status(&var))
            .collect();
        
        // All branches must agree
        let first = &statuses[0];
        if statuses.iter().all(|s| s == first) {
            result.insert(var, envs[0].get_type(&var), first.clone());
        } else {
            return Err(TypeError::BranchMismatch { var, statuses });
        }
    }
    
    Ok(result)
}
```

---

## 7. Error Messages

### Move After Use

```
error[E001]: value moved here
  --> example.ash:15:9
   |
10 |     let (addr, ctl) = split w;
   |          ---- binding defined here
...
14 |     let result = await addr;
   |                  ---------- value moved here
15 |     signal addr with { more: work };
   |            ^^^^ value used here after move
   |
   = help: `addr` has type `InstanceAddr<worker>` which is affine
   = note: consider restructuring so `await` happens after all signals
```

### Branch Mismatch

```
error[E002]: variable `addr` has different status in branches
  --> example.ash:20:5
   |
11 |     receive {
12 |         { fast: true } => {
13 |             let result = await addr;
   |                            -------- `addr` moved here
14 |         }
15 |         { fast: false } => {
16 |             signal addr with { work: x };
   |                   ---- `addr` still available here
17 |         }
18 |     }
19 |     
20 |     signal addr with { more: work };
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot use `addr` here
   |
   = help: all branches of `receive` must treat affine values consistently
   = note: either move `addr` in all branches or in none
```

### Loop Invariant Violation

```
error[E003]: loop does not preserve affine invariant
  --> example.ash:10:5
   |
8  |     let (addr, ctl) = split w;
   |          ---- `addr` is available here
9  |     loop {
10 |         let result = await addr;
   |                        -------- `addr` moved here
11 |     }
   |     ^ loop ends with `addr` moved, but started with it available
   |
   = help: move `await` outside the loop, or spawn inside the loop
```

---

## 8. Simplifications for MVP

### Option A: No Split (Bundle Only)

```ash
spawn worker with {} as w;

-- w has both address and control, cannot split
signal w with { work: x };           -- Uses address
send_control w with shutdown;        -- Uses control
await w;                             -- Consumes both
```

**Pros**: No linearity tracking needed
**Cons**: Less flexible, can't delegate supervision

### Option B: Explicit Split, No Borrowing

What we've described above.

**Pros**: Flexible, type-safe delegation
**Cons**: More complex type checking

### Option C: No Affine Types (GC Everything)

```ash
spawn worker with {} as w;
-- w is regular value, garbage collected
```

**Pros**: Simple
**Cons**: No deterministic cleanup, no delegation guarantees

### Recommendation: Option B (Explicit Split)

The type checking is manageable:
1. Track status per variable (available/moved)
2. Check branch joins agree
3. Check loop invariants

No lifetimes, no borrowing, just explicit moves.

---

## 9. Future Extensions

### Borrowing (Post-MVP)

```ash
signal &addr with { work: x };  -- Borrow, don't move
```

Requires lifetime tracking. Significantly more complex.

### Region-Based Cleanup

```ash
region {
    spawn worker with {} as w;
    let (addr, ctl) = split w;
    -- use w...
}  -- All affine values in region are dropped here
```

### Scoped Instances

```ash
with spawn worker with {} as w {
    -- w available here
    signal w with { work };
}  -- w automatically awaited/dropped here
```

---

## Summary

| Feature | Status | Complexity |
|---------|--------|------------|
| Affine types (move-only) | Required | Medium |
| Split | Required | Medium |
| Signal (non-consuming) | Required | Low |
| Await (consuming) | Required | Low |
| Branch join checking | Required | Medium |
| Loop invariant checking | Required | Medium |
| Shareable | Nice to have | Low |
| Borrowing | Future | High |
| Lifetimes | Future | Very High |

The key insight: **explicit moves, no borrowing, join point checking**. This gives us affine safety without Rust's lifetime complexity.
