# SPEC-013: Streams and Event Processing

## Status: Draft

## 1. Overview

Streams provide discrete event processing capabilities for Ash workflows. Unlike continuous behaviours (SPEC-014), streams are sequences of events that occur at specific points in time. The `receive` construct enables workflows to consume events from multiple sources with pattern matching. The authoritative workflow-level `receive` grammar is defined in SPEC-002; this spec defines the stream-specific runtime contract for that grammar.

## 2. Concepts

### 2.1 Stream vs Behaviour

| Aspect | Stream (Event) | Behaviour |
|--------|---------------|-----------|
| Nature | Discrete occurrences | Continuous value |
| Time | Point-in-time | Time-varying |
| Consumption | Sequential, destructive | Sampling, non-destructive |
| Example | Kafka message, button click | Current temperature, system status |

### 2.2 Mailbox Model

Each workflow has a **mailbox** that buffers incoming events:
- Events from all subscribed streams accumulate in the mailbox
- Pattern matching selects which events to process
- Non-matching events remain in mailbox for future matching
- Mailbox has configurable size limits and overflow strategies

### 2.3 Source Scheduling Terminology

This specification uses the term **source scheduling modifier** for the language-level rule
that determines how `receive` chooses among eligible stream sources. The runtime implements
that rule with a scheduler.

The current default source scheduling modifier is **priority**:
- arm order is significant
- earlier arms and sources may starve later ones
- fairer modifiers such as `round_robin`, `random`, or `fair` are possible future extensions,
  but are not yet standardized surface syntax

## 3. Syntax

### 3.1 Workflow Declaration

```
workflow_def ::= "workflow" IDENTIFIER 
                 ("observes" capability_list)?
                 ("receives" stream_list)?
                 workflow_body

capability_ref ::= IDENTIFIER (":" IDENTIFIER)?
capability_list ::= capability_ref ("," capability_ref)*
stream_list  ::= stream_decl ("," stream_decl)*
stream_decl  ::= IDENTIFIER (":" IDENTIFIER)?
               | IDENTIFIER "{" IDENTIFIER ("," IDENTIFIER)+ "}"
```

Examples:
```
workflow handler receives sensor:temperature, kafka:orders {
    ...
}

workflow multi receives kafka{orders, metrics, alerts} {
    ...
}

workflow hybrid
    observes sensor:temperature
    receives sensor:events
{
    ...
}
```

### 3.2 Receive Construct

Three modes:

```
receive_expr ::= "receive" ("control")? receive_mode? "{" receive_arm+ "}"

receive_mode ::= "wait" (DURATION)?

receive_arm  ::= receive_pattern ("if" expr)? "=>" workflow
               | "_" "=>" workflow

receive_pattern ::= IDENTIFIER ":" IDENTIFIER "as" pattern
                  | literal           -- control receive patterns
```

**Mode 1: Non-blocking (default)**
```
receive {
    sensor:temperature as t if t > 100 => act alert(t),
    kafka:orders as order => act process(order),
    _ => act log("no events")
}
```

**Mode 2: Blocking forever**
```
receive wait {
    sensor:temperature as t => act handle(t),
    kafka:orders as order => act handle(order)
}
```

**Mode 3: Blocking with timeout**
```
receive wait 30s {
    sensor:temperature as t => act handle(t),
    _ => act heartbeat()
}
```

### 3.3 Control Stream

Every workflow has an implicit control stream:

```ash
receive control {
    "shutdown" => break,
    "pause" => sleep(60s),
    "resume" => done,
    "restart" => restart,
    _ => done
}
```

Common control messages:
- `"shutdown"` - Graceful termination
- `"pause"` - Suspend processing
- `"resume"` - Resume processing
- `"restart"` - Restart from beginning
- `"status"` - Return current status

Control-arm semantics are canonical:

- `receive control` polls only the implicit control mailbox.
- Control arms do not require `receives ...` declarations.
- Control arms are matched in declaration order, just like normal receive arms.
- A control message is consumed only when an arm matches and its guard succeeds.
- Normal `receive` and `receive control` are separate operations; a single `receive` block never mixes control and normal stream selectors.

## 4. Pattern Matching

### 4.1 Simple Binding
```
sensor:temperature as temp => ...
```

### 4.2 Record Destructuring
```
sensor:temperature as { value: t, unit: u } => ...
```

### 4.3 Nested Destructuring
```
kafka:orders as {
    id: order_id,
    customer: { name: cust_name, tier: "gold" },
    items: [first, ..rest]
} => ...
```

### 4.4 List Patterns
```
queue:batch as [item1, item2, ..others] => ...
queue:batch as [] => act log("empty")
```

### 4.5 Wildcard
```
_ => act log("unhandled event")
```

ADT variant patterns are also valid stream-message patterns:

```ash
queue:jobs as Ok { value: job } => act process(job)
queue:jobs as Err { error: reason } => act log::error(reason)
```

These variant patterns follow the canonical ADT contract from `SPEC-020`: matching is based on
constructor name plus named fields, not on synthetic tag fields.

## 5. Guard Clauses

Patterns can have boolean guards:

```
receive {
    sensor:temperature as t if t > 100 && t < 200 =>
        act cool::start(),
    
    kafka:orders as order if order.priority == "urgent" && order.amount > 10000 =>
        act notify::manager(order),
    
    sensor:temperature as t if typeof(t) == "number" =>
        act log::metric(t)
}
```

## 6. Execution Semantics

### 6.1 Pattern Matching Order

The current default source scheduling modifier is `priority`. Under `priority`, arms are
checked in declaration order and later arms may never be reached if earlier arms continue to
match available messages.

```
receive {
    sensor:temperature as t if t > 100 => act alert(),  -- Checked first
    sensor:temperature as t => act log(t),              -- Checked second
    _ => act default()                                   -- Checked last
}
```

### 6.2 Mailbox Processing

```
execute_receive(patterns, mode, source_scheduling_modifier):
    1. Ask the runtime scheduler for the next eligible source according to
       source_scheduling_modifier.
    2. For each arm in order:
       a. Search that source's mailbox for the oldest matching entry
       b. If guard present, evaluate guard
       c. If match found:
          - Remove from mailbox
          - Execute corresponding workflow
          - Return
    
    3. If wildcard pattern exists:
       - Execute wildcard workflow
       - Return
    
    4. If mode is non-blocking:
       - Return (continue to next statement)
    
    5. If mode is blocking:
       - Wait for new events (with optional timeout)
       - Add events to mailbox
       - Retry from step 1
```

The runtime owns the concrete mailbox and transport organization. The language contract only
requires the observable behavior above.

### 6.3 Timeout Behavior

With `receive wait DURATION`:
- Wait up to DURATION for a matching event
- If timeout expires, execute `_` pattern (if present)
- If no `_` pattern, continue to the next workflow step without binding a value

The timeout path is not an error by itself.

## 7. Mailbox Configuration

Optional mailbox limits:

```
workflow handler receives sensor:temperature, kafka:orders
    mailbox_limit 1000
    overflow_strategy drop_oldest
{
    ...
}
```

Overflow strategies:
- `drop_oldest` - Remove oldest messages (default)
- `drop_newest` - Drop new incoming messages
- `error` - Raise overflow error
- `block_sender` - Backpressure on event sources

These limits apply to declared stream mailboxes, not to the implicit control mailbox.

## 8. Stream Provider Interface

```rust
/// A provider of event streams
#[async_trait]
pub trait StreamProvider: Send + Sync {
    fn capability_name(&self) -> &str;
    fn channel_name(&self) -> &str;
    
    /// Try to receive without blocking
    fn try_recv(&self) -> Option<ExecResult<Value>>;
    
    /// Block until message available
    async fn recv(&self) -> ExecResult<Value>;
    
    /// Check if stream is closed
    fn is_closed(&self) -> bool;
}
```

## 9. Examples

### 9.1 Simple Event Processor

```
workflow processor receives kafka:orders {
    loop {
        receive control {
            "shutdown" => break
        }
        
        receive wait {
            kafka:orders as order =>
                act db::insert(order)
        }
    }
}
```

### 9.2 Multi-Source Handler

```
workflow handler receives sensor:temperature, sensor:pressure, alert:config {
    loop {
        receive control {
            "shutdown" => break,
            "status" => act log::status("running"),
            _ => ()
        }
        
        receive wait 30s {
            sensor:temperature as { value: t } if t > 100 =>
                act alert::trigger("overheat", t),
            
            sensor:pressure as { value: p } if p < 10 =>
                act alert::trigger("pressure_low", p),
            
            alert:config as new_config =>
                act config::update(new_config),
            
            _ => act heartbeat::ping()
        }
    }
}
```

### 9.3 Pattern Matching with Destructuring

```
workflow order_processor receives kafka:orders {
    loop {
        receive wait {
            kafka:orders as {
                id: id,
                customer: { tier: "gold", email: email },
                items: [..items],
                total: total
            } if total > 10000 =>
                act process::vip_order(id, email, items),
            
            kafka:orders as { id: id, total: total } if total > 1000 =>
                act process::large_order(id, total),
            
            kafka:orders as order =>
                act process::standard_order(order),
            
            _ => act log::debug("no orders")
        }
    }
}
```

## 10. Relationship to Other Features

### 10.1 With Observe (Behaviours)

Workflows can mix streams and behaviours:

```
workflow mixed receives sensor:events {
    -- Sample current temperature (behaviour)
    observe sensor:temperature as current_temp;
    
    loop {
        receive wait 10s {
            sensor:events as event =>
                act handle(event, current_temp),
            
            _ => {
                -- Re-sample temperature on timeout
                observe sensor:temperature as current_temp;
                act log::metric(current_temp)
            }
        }
    }
}
```

### 10.1 With Parallel Composition

Streams within `par` branches have independent mailboxes:

```
workflow parallel {
    par {
        -- Branch 1: Process orders
        for order in stream kafka:orders {
            act process::order(order)
        }
    } {
        -- Branch 2: Handle alerts
        for alert in stream alert:critical {
            act notify::immediate(alert)
        }
    }
}
```

## 11. Error Handling

### 11.1 Stream Closed

When a finite stream closes:
- `try_recv()` returns `None`
- `recv()` returns error `StreamClosed`
- Workflow can handle with pattern or error recovery

### 11.2 Mailbox Overflow

When mailbox exceeds limit:
- Default: Drop oldest messages
- Configurable per workflow
- Can emit warning log

## 12. Future Extensions

- Stream transformations (map, filter, buffer)
- Stream merging with priorities
- Windowed aggregations (tumbling, sliding)
- Exactly-once processing guarantees
