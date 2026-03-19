# Algebraic Data Types (ADTs) for Ash

Proposal: Add full sum and product types to Ash's type system.

## 1. Core Concepts

```
Type ::= 
    -- Basic (existing)
    | Int | String | Bool | Null | Time | Ref
    
    -- Products (new)
    | Tuple(Type...)                    -- Anonymous product
    | Struct { field: Type, ... }       -- Named product
    
    -- Sums (new)
    | Enum { Variant, ... }             -- Sum type
    
    -- Constructors (new)
    | TypeConstructor(Name, Type...)    -- Generic instantiation
    
    -- Existing
    | List(Type)
    | Cap { name, effect }
    | Fun(Args..., Ret, Effect)
```

## 2. Product Types

### Tuples (Anonymous)

```ash
-- Syntax: (T1, T2, ..., Tn)
let point: (Int, Int) = (10, 20);
let person: (String, Int, Bool) = ("Alice", 30, true);

-- Pattern matching
let (x, y) = point;
let (name, age, active) = person;

-- Nested
let nested: ((Int, Int), String) = ((1, 2), "center");
let ((a, b), label) = nested;
```

### Structs (Named Products)

```ash
-- Definition
type Point = { x: Int, y: Int };
type Person = { name: String, age: Int, active: Bool };

-- Construction
let p: Point = { x: 10, y: 20 };
let person = { name: "Alice", age: 30, active: true };

-- Field access
let x_coord = p.x;
let person_name = person.name;

-- Functional update (copy with changes)
let moved = { p with x: p.x + 5 };
let older = { person with age: person.age + 1 };

-- Pattern matching
let { x, y } = p;
let { name, age, .. } = person;  -- Ignore other fields
```

## 3. Sum Types (Enums)

### Basic Enum

```ash
-- Definition
type Status = 
    | Pending
    | Processing { started_at: Time }
    | Completed { result: Value, completed_at: Time }
    | Failed { error: String, retryable: Bool };

-- Construction
let s1: Status = Pending;
let s2 = Processing { started_at: now() };
let s3 = Completed { result: 42, completed_at: now() };

-- Pattern matching
decide { status } then {
    -- Matches only Completed (exhaustiveness check)
}

-- Or explicit match
match status {
    Pending => act log with "Waiting...",
    Processing { started_at: t } => act log with "Started at " + t,
    Completed { result: r } => ret r,
    Failed { error: e, retryable: true } => retry(),
    Failed { error: e, retryable: false } => abort(e)
}
```

### Generic Enums

```ash
-- Option<T>
type Option<T> = 
    | Some { value: T }
    | None;

-- Result<T, E>
type Result<T, E> =
    | Ok { value: T }
    | Err { error: E };

-- Using generics
let maybe_ctrl: Option<ControlLink<Worker>> = Some { value: ctrl };
let result: Result<Int, String> = Ok { value: 42 };

-- Pattern matching with generics
match maybe_ctrl {
    Some { value: link } => send_control link with msg,
    None => act log with "No control available"
}
```

### Recursive Types

```ash
-- Linked list
type List<T> = 
    | Nil
    | Cons { head: T, tail: List<T> };

-- Tree
type Tree<T> =
    | Leaf { value: T }
    | Node { left: Tree<T>, right: Tree<T> };

-- JSON-like value
type JsonValue =
    | Null
    | Bool { value: Bool }
    | Number { value: Float }
    | String { value: String }
    | Array { elements: List<JsonValue> }
    | Object { fields: List<(String, JsonValue)> };
```

## 4. Control Link with Option

```ash
-- Built-in or standard library
type Option<T> = 
    | Some { value: T }
    | None;

-- Spawn returns split result
spawn worker with {} as w;
let (w_addr, w_ctrl) = split w;

-- w_ctrl: Option<ControlLink<Worker>>
-- Initially: Some { value: actual_link }

-- Pattern match on Option
match w_ctrl {
    Some { value: link } => {
        send_control link with checkin;
    },
    None => {
        act log with "Control already transferred";
    }
}

-- Or with guard syntax
if let Some { value: link } = w_ctrl then {
    send_control link with shutdown;
} else {
    act log with "No control";
}
```

## 5. Result Type for Error Handling

```ash
type Result<T, E> =
    | Ok { value: T }
    | Err { error: E };

-- Capability that can fail
capability fetch_data : observe(id: String) returns Result<Data, FetchError>;

workflow process {
    let result = observe fetch_data with id: "123";
    
    match result {
        Ok { value: data } => {
            process_data(data);
        },
        Err { error: FetchError::NotFound } => {
            ret { error: "Data not found" };
        },
        Err { error: FetchError::Timeout } => {
            retry();
        },
        Err { error: e } => {
            escalate(e);
        }
    }
}

-- Or with try operator (?)
let data = observe fetch_data with id: "123" ?;  -- Returns early on Err
```

## 6. Pattern Matching Integration

### In Receive

```ash
receive {
    -- Match enum variant with payload
    { status: Completed { result: r } } => process(r),
    
    -- Match with guard
    { status: Failed { error: e } } if is_retryable(e) => retry(),
    
    -- Wildcard
    _ => act log with "Unhandled"
}
```

### In Decide

```ash
decide { status } matches Completed { result: r } then {
    -- Only enters if Completed
    use_result(r);
} else {
    -- Handle other cases
}
```

### In Let

```ash
-- Destructuring
let Some { value: link } = maybe_ctrl;

-- With default
let value = match maybe_val {
    Some { value: v } => v,
    None => 0
};
```

## 7. Type Checking

### Constructor Check

```
Γ ⊢ e: T
-----------------------------------------------
Γ ⊢ Some { value: e }: Option<T>
```

### Pattern Check

```
Γ ⊢ e: Option<T>
Γ, x: T ⊢ body1: R
Γ ⊢ body2: R
-----------------------------------------------
Γ ⊢ match e {
    Some { value: x } => body1,
    None => body2
}: R
```

### Exhaustiveness Check

```ash
match status {
    Pending => ...,           -- OK if Status = Pending | Completed
    Completed { result } => ...
    -- ERROR: Missing Failed variant!
}
```

Must cover all variants or have wildcard `_`.

## 8. Governance with ADTs

```ash
-- Policy result as enum
type PolicyDecision =
    | Permit
    | Deny { reason: String }
    | RequireApproval { role: Role, deadline: Time }
    | Escalate { to: Role };

-- Audit log entry
type AuditEvent =
    | WorkflowStarted { id: WorkflowId, by: User }
    | CapabilityUsed { workflow: WorkflowId, cap: Capability, at: Time }
    | PolicyEvaluated { workflow: WorkflowId, policy: Policy, decision: PolicyDecision }
    | WorkflowCompleted { id: WorkflowId, result: CompletionStatus };

type CompletionStatus =
    | Success { value: Value }
    | Failure { error: Error }
    | Cancelled { by: User, reason: String };
```

## 9. Comparison: Before vs After

### Before (No ADTs)

```ash
-- Using records with nullable fields
type ControlHandle = { 
    link: ControlLink?,  -- Nullable reference
    has_control: Bool 
};

-- Manual checking
if handle.has_control and handle.link != null then {
    send_control handle.link with msg;
}
```

### After (With ADTs)

```ash
-- Type-safe enum
type ControlHandle = Option<ControlLink>;

-- Exhaustive pattern matching
match handle {
    Some { value: link } => send_control link with msg,
    None => act log with "No control"
}
```

## 10. Implementation Phases

### Phase 1: Product Types
- Tuples: `(Int, String)`
- Structs: `{ x: Int, y: Int }`
- Field access: `p.x`
- Functional update: `{ p with x: 5 }`

### Phase 2: Basic Sum Types
- Enum definition
- Variant construction
- Pattern matching
- Exhaustiveness checking

### Phase 3: Generics
- Type parameters: `Option<T>`, `Result<T, E>`
- Generic constraints (future)

### Phase 4: Advanced Features
- Recursive types
- Associated types (future)
- GADTs (future, maybe)

## 11. Syntax Summary

```ash
-- Type definitions
type Point = { x: Int, y: Int };
type Status = Pending | Processing | Completed { result: Value };
type Option<T> = Some { value: T } | None;
type Result<T, E> = Ok { value: T } | Err { error: E };

-- Construction
let p = { x: 1, y: 2 };
let s = Completed { result: 42 };
let o = Some { value: link };

-- Pattern matching
match value {
    Variant1 => expr1,
    Variant2 { field: x } => expr2,
    _ => default
}

-- Destructuring let
let { x, y } = point;
let Some { value: v } = opt;

-- If-let
if let Some { value: link } = ctrl then {
    use(link);
}
```

## Open Questions

1. **Syntax for generics**: `Option<T>` or `Option of T` or `Option<T>`?
2. **Nullary variants**: `Pending` or `Pending {}`?
3. **Tuple structs**: `Point(Int, Int)` or just use tuples?
4. **Newtype pattern**: `type UserId = String` - wrapper or alias?
5. **Deriving**: Auto-derive `equals`, `hash`, `show`?

This would significantly increase Ash's expressive power while maintaining type safety.
