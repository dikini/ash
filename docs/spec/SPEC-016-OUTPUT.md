# SPEC-016: Output Capabilities (Send and Set)

## Status: Draft

## 1. Overview

Output capabilities are the duals to input capabilities:

| Input | Output | Target |
|-------|--------|--------|
| `observe` (behaviour) | `set` | Continuous values |
| `receive` (stream) | `send` | Discrete events |

This enables workflows to affect the external world, not just observe it.

This specification governs workflow-level external outputs: `set` for behaviours and
`send` for streams. It does not define CLI diagnostics, REPL display text, help rendering,
or other developer-tool output. Those user-interface outputs are specified by `SPEC-005`
and `SPEC-011`.

## 2. Set (Output Behaviours)

### 2.1 Concept

Set the value of an external behaviour. Unlike `observe` which samples, `set` changes the value.

### 2.2 Syntax

```
set_stmt ::= "set" capability_ref "=" expr

-- Examples:
set hvac:target_temperature = 22
set light:living_room = { brightness: 80, color: "warm" }
set config:timeout = current_timeout + 30
```

### 2.3 Semantics

- **Effect**: Operational (modifies external state)
- **Returns**: Unit (or error if set fails)
- **Async**: May wait for acknowledgement

```rust
// Provider interface
#[async_trait]
pub trait SettableBehaviourProvider: BehaviourProvider {
    async fn set(&self, value: Value) -> ExecResult<()>;
    
    /// Validate value before setting (optional)
    fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        Ok(()) // Default: accept all
    }
}
```

### 2.4 Example: Thermostat Controller

```ash
workflow thermostat observes sensor:temp, sets hvac:target {
    loop {
        observe sensor:temp as current;
        
        if current > 75 then {
            set hvac:target = 72;
            act log::info("cooling to 72")
        } else if current < 65 then {
            set hvac:target = 68;
            act log::info("heating to 68")
        } else {
            act log::debug("temperature OK")
        };
        
        sleep(60s)
    }
}
```

## 3. Send (Output Streams)

### 3.1 Concept

Send an event to an external stream. Unlike `receive` which consumes, `send` produces.

### 3.2 Syntax

```
send_stmt ::= "send" capability_ref expr

-- Examples:
send kafka:orders { id: "123", items: cart }
send notification:alerts "System overload"
send metrics:timings { operation: "db_query", duration: elapsed }
```

### 3.3 Semantics

- **Effect**: Operational (produces external effect)
- **Returns**: Unit (or error if send fails)
- **Async**: May wait for buffer space or acknowledgement
- **Buffering**: Provider may buffer, drop, or block

```rust
// Provider interface
#[async_trait]
pub trait SendableStreamProvider: StreamProvider {
    async fn send(&self, value: Value) -> ExecResult<()>;
    
    /// Check if send would block
    fn would_block(&self) -> bool {
        false // Default: never blocks
    }
    
    /// Flush any buffered sends
    async fn flush(&self) -> ExecResult<()> {
        Ok(()) // Default: no-op
    }
}
```

### 3.4 Example: Order Processor

```ash
workflow order_processor receives kafka:orders, sends notification:alerts {
    loop {
        receive wait {
            kafka:orders as order => {
                act process::order(order);
                
                if order.total > 10000 then
                    send notification:alerts {
                        level: "high_value",
                        order_id: order.id,
                        amount: order.total
                    }
                else
                    ();
            }
        }
    }
}
```

## 4. Bidirectional Capabilities

Some capabilities support both read and write:

```ash
-- Bidirectional behaviour
workflow config_manager observes config:settings, sets config:settings {
    observe config:settings as current;
    
    if current.mode == "production" && current.debug then
        set config:settings = { current with debug: false }
    else
        done
}

-- Bidirectional stream
workflow bridge receives kafka:topic_a, sends kafka:topic_b {
    loop {
        receive wait {
            kafka:topic_a as msg => {
                let transformed = transform(msg);
                send kafka:topic_b transformed
            }
        }
    }
}
```

## 5. Type Safety

### 5.1 Schema Declaration

Output providers declare their accepted types:

```rust
pub struct SettableBehaviourProvider {
    read_schema: Type,   // For observe
    write_schema: Type,  // For set
    ...
}

impl TypedSettableProvider {
    pub fn new(
        read_schema: Type,
        write_schema: Type,
        provider: impl SettableBehaviourProvider,
    ) -> Self {
        ...
    }
}
```

### 5.2 Validation

```rust
async fn set(&self, value: Value) -> ExecResult<()> {
    // Validate against write_schema
    if !self.write_schema.matches(&value) {
        return Err(ExecError::TypeMismatch { ... });
    }
    
    self.inner.set(value).await
}
```

### 5.3 Workflow Type Checking

```ash
-- Type error: setting String to Int capability
set sensor:target_temperature = "hot"  -- ERROR: expected Int
```

## 6. Effects

| Operation | Effect | Notes |
|-----------|--------|-------|
| `observe` | Epistemic | Read-only observation |
| `receive` | Epistemic | Read-only event consumption |
| `set` | Operational | Modifies external state |
| `send` | Operational | Produces external event |

The effect system tracks that `set` and `send` have side effects.

## 7. Error Handling

### 7.1 Set Errors

```rust
pub enum SetError {
    #[error("capability is read-only")]
    ReadOnly,
    
    #[error("value rejected: {reason}")]
    Rejected { reason: String },
    
    #[error("timeout waiting for acknowledgement")]
    Timeout,
    
    #[error("disconnected")]
    Disconnected,
}
```

### 7.2 Send Errors

```rust
pub enum SendError {
    #[error("stream is full")]
    BufferFull,
    
    #[error("stream is closed")]
    Closed,
    
    #[error("timeout")]
    Timeout,
}
```

### 7.3 Workflow Error Handling

```ash
workflow resilient_sends {
    attempt {
        send kafka:orders order
    } retry 3 on SendError::BufferFull {
        sleep(100ms)  -- Back off
    } on error {
        act log::error("failed to send order");
        send dlq:failed_orders { original: order, error: "send_failed" }
    }
}
```

## 8. Provider Implementation

### 8.1 Settable Sensor

```rust
pub struct ThermostatProvider {
    target: Arc<AtomicI64>,
}

#[async_trait]
impl SettableBehaviourProvider for ThermostatProvider {
    fn capability_name(&self) -> &str { "hvac" }
    fn channel_name(&self) -> &str { "target" }
    
    async fn sample(&self, _: &[Constraint]) -> ExecResult<Value> {
        Ok(Value::Int(self.target.load(Ordering::Relaxed)))
    }
    
    async fn set(&self, value: Value) -> ExecResult<()> {
        match value {
            Value::Int(temp) => {
                self.target.store(temp, Ordering::Relaxed);
                Ok(())
            }
            _ => Err(ExecError::TypeMismatch { ... }),
        }
    }
}
```

### 8.2 Sendable Stream

```rust
pub struct KafkaProducerProvider {
    producer: KafkaProducer,
    topic: String,
}

#[async_trait]
impl SendableStreamProvider for KafkaProducerProvider {
    fn capability_name(&self) -> &str { "kafka" }
    fn channel_name(&self) -> &str { "orders" }
    
    async fn send(&self, value: Value) -> ExecResult<()> {
        let bytes = serialize(&value)?;
        self.producer.send(&self.topic, bytes).await
            .map_err(|e| ExecError::SendFailed(e.to_string()))?;
        Ok(())
    }
    
    fn would_block(&self) -> bool {
        self.producer.buffer_full()
    }
}
```

## 9. Declaration Syntax

### 9.1 Read-Only

```ash
workflow observer observes sensor:temp {
    -- Can only observe
}
```

### 9.2 Write-Only

```ash
workflow sender sends notification:alerts {
    -- Can only send
}
```

### 9.3 Read-Write

```ash
workflow controller observes sensor:temp, sets hvac:target {
    -- Can both observe and set
}
```

### 9.4 Multiple Streams

```ash
workflow bridge 
    receives kafka:input
    sends kafka:output, kafka:audit 
{
    -- Receives from input, sends to both outputs
}
```

## 10. Relationship to Other Concepts

### 10.1 vs Act

| Feature | `act` | `set`/`send` |
|---------|-------|--------------|
| Target | Action/capability | Behaviour/stream |
| Timing | Immediate execution | State change/event production |
| Effect | Operational | Operational |

`act` is imperative ("do this"). `set`/`send` is declarative ("this is the new value/event").

### 10.2 vs Obligations

Obligations are requirements on others. Set/send are actions by this workflow.

```ash
-- This workflow promises to do something (obligation)
oblig role:monitor to workflow { ... }

-- This workflow actually does it (action)
set sensor:config = { ... }
```

## 11. Examples

### 11.1 Configuration Propagation

```ash
workflow config_sync observes config:master, sets config:slaves {
    loop {
        receive control {
            "sync" => {
                observe config:master as master_config;
                set config:slaves = master_config;
                act log::info("config synced")
            },
            _ => ()
        }
    }
}
```

### 11.2 Event Transformation

```ash
workflow transformer 
    receives raw:events
    sends processed:events 
{
    loop {
        receive wait {
            raw:events as raw => {
                let processed = {
                    id: raw.id,
                    timestamp: now(),
                    normalized: normalize(raw.data)
                };
                send processed:events processed
            }
        }
    }
}
```

### 11.3 Request-Response

```ash
workflow request_handler 
    receives http:requests
    sends http:responses 
{
    loop {
        receive wait {
            http:requests as req => {
                let result = handle_request(req);
                send http:responses {
                    request_id: req.id,
                    status: 200,
                    body: result
                }
            }
        }
    }
}
```

## 12. Implementation Tasks

- TASK-101: Settable behaviour provider trait
- TASK-102: Sendable stream provider trait
- TASK-103: Parse set statement
- TASK-104: Parse send statement
- TASK-105: Set execution
- TASK-106: Send execution
- TASK-107: Bidirectional provider wrappers
