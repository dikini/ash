# SPEC-014: Behaviours (Observable Values)

## Status: Draft

## 1. Overview

Behaviours represent time-varying values in the external environment that workflows can sample. Unlike streams (SPEC-013) which are discrete events, behaviours are continuous - they have a value at every point in time. Two observations of the same behaviour at different times may return different values.

This concept is inspired by Functional Reactive Programming (FRP), but simplified for workflow contexts.

## 2. Core Concepts

### 2.1 Behaviour vs Stream

| Aspect | Behaviour | Stream |
|--------|-----------|--------|
| Time model | Continuous | Discrete |
| Values | Always have a current value | Events occur at specific times |
| Consumption | Sampling (non-destructive) | Consumption (destructive) |
| Operations | `observe` (pull) | `receive` (pull/push hybrid) |
| Example | Temperature sensor, stock price | Button click, order placed |

### 2.2 Observable Values

An observable is an external resource that exposes a behaviour:
- Can be sampled at any time
- Value may change between observations
- Changes originate from outside the workflow

## 3. Syntax

### 3.1 Basic Observation

```
observation ::= "observe" capability_ref ("where" constraint_list)? "as" pattern

capability_ref ::= IDENTIFIER (":" IDENTIFIER)?
```

Example:
```
observe sensor:temperature as temp;
observe market:stock where symbol = "AAPL" as price;
```

### 3.2 Observation in Context

```
workflow monitor {
    -- Sample once
    observe sensor:temperature as temp;
    
    if temp > 100 then
        act alert::trigger("overheating")
    else
        done
}
```

### 3.3 Repeated Observation

```
workflow continuous_monitor {
    loop {
        observe sensor:temperature as temp;
        act log::metric("temperature", temp);
        sleep(5s)
    }
}
```

## 4. Semantics

### 4.1 Sampling Model

`observe` captures the behaviour's value **at the moment of observation**:

```
workflow sampler {
    observe sensor:temperature as t1;  -- Sample at time t
    sleep(1s);
    observe sensor:temperature as t2;  -- Sample at time t+1
    
    -- t1 and t2 may differ
    act log::delta(t2 - t1)
}
```

### 4.2 No Caching

Each `observe` is independent:
- No implicit caching between observations
- No guarantee that sequential observations see the same value
- Provider may optimize, but workflow should not rely on it

### 4.3 Constraints

Observations can specify constraints to filter/select:

```
-- Observe specific field
observe sensor:temperature where field = "celsius" as celsius;

-- Observe with freshness requirement
observe cache:data where max_age = "5s" as data;

-- Multiple constraints
observe market:stock where symbol = "AAPL" and exchange = "NASDAQ" as price;
```

## 5. Provider Interface

```rust
/// A provider of observable behaviours
#[async_trait]
pub trait BehaviourProvider: Send + Sync {
    fn capability_name(&self) -> &str;
    fn channel_name(&self) -> &str;
    
    /// Sample the current value
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value>;
    
    /// Check if value has changed since last sample (optional optimization)
    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool> {
        Ok(true) // Default: assume changed
    }
}
```

## 6. Change Detection (Optional)

Workflows can check if a behaviour has changed:

```
workflow efficient_monitor {
    loop {
        -- Check if changed first (optional optimization)
        if changed sensor:temperature then {
            observe sensor:temperature as temp;
            act process(temp)
        } else {
            act log::debug("no change")
        };
        
        sleep(1s)
    }
}
```

Note: `changed` is a hint - providers may always return `true`.

## 7. Effect System

Observations have **Epistemic** effect:
- They read from the environment
- They don't modify external state
- Safe to call repeatedly

```
observe sensor:temperature;  -- Effect: epistemic
```

## 8. Comparison with Streams

### 8.1 When to Use Behaviours

- Current state that can be sampled (temperature, price, status)
- Values where "current" is meaningful
- No need to process every change
- Can miss intermediate values

### 8.2 When to Use Streams

- Discrete events (clicks, orders, alerts)
- Every occurrence matters
- Must process sequentially
- Cannot miss events

### 8.3 Hybrid Usage

Workflows can use both:

```
workflow hybrid receives sensor:events {
    -- Sample current baseline (behaviour)
    observe sensor:temperature as baseline;
    
    loop {
        -- Process events (stream)
        receive wait 30s {
            sensor:events as { type: "spike", value: v } =>
                act alert::spike(baseline, v),
            
            _ => {
                -- Re-sample baseline periodically
                observe sensor:temperature as baseline;
                act log::baseline(baseline)
            }
        }
    }
}
```

## 9. Provider Examples

### 9.1 Sensor Provider

```rust
pub struct SensorProvider {
    sensor: Arc<dyn Sensor>,
}

#[async_trait]
impl BehaviourProvider for SensorProvider {
    fn capability_name(&self) -> &str { "sensor" }
    fn channel_name(&self) -> &str { "temperature" }
    
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let reading = self.sensor.read().await;
        Ok(Value::Record(hashmap! {
            "value".into() => Value::Int(reading.value),
            "unit".into() => Value::String(reading.unit),
            "timestamp".into() => Value::Time(reading.timestamp),
        }))
    }
}
```

### 9.2 Agent Environment Provider

```rust
pub struct AgentEnvironmentProvider {
    state: Arc<RwLock<AgentState>>,
}

#[async_trait]
impl BehaviourProvider for AgentEnvironmentProvider {
    fn capability_name(&self) -> &str { "agent" }
    fn channel_name(&self) -> &str { "environment" }
    
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let state = self.state.read().await;
        
        // Return specific field if constrained
        if let Some(field) = constraints.iter().find(|c| c.name == "field") {
            match field.value.as_str() {
                "temperature" => Ok(Value::Int(state.temperature)),
                "status" => Ok(Value::String(state.status.clone())),
                _ => Err(ExecError::InvalidConstraint(field.value.clone())),
            }
        } else {
            // Return full state
            Ok(state.to_value())
        }
    }
}
```

## 10. Error Handling

### 10.1 Unavailable Behaviour

```
workflow resilient {
    attempt {
        observe sensor:temperature as temp;
        act process(temp)
    } retry 3 timeout 5s on error {
        act log::error("sensor unavailable");
        act use::default_value(25)
    }
}
```

### 10.2 Stale Data

```
workflow fresh_only {
    attempt {
        observe cache:data where max_age = "1s" as data;
        act process(data)
    } on error {
        act fetch::fresh_data()
    }
}
```

## 11. Relationship to FRP

This is a **simplified** FRP model:

| FRP Concept | Ash Equivalent | Notes |
|-------------|----------------|-------|
| Behaviour | Behaviour | Sample-able, time-varying |
| Event | Stream | Discrete occurrences (SPEC-013) |
| Signal | N/A | Not implemented |
| lift/fmap | Transform in workflow | Explicit in workflow body |
| foldp | State variable | Use `let` bindings |
| merge/combine | N/A | Explicit in workflow |

We intentionally avoid:
- Automatic re-evaluation on change (no push-based behaviours)
- Continuous-time semantics (discrete sampling only)
- Signal function composition (explicit workflow steps)

## 12. Examples

### 12.1 Simple Thermostat

```
workflow thermostat observes sensor:temperature {
    loop {
        observe sensor:temperature as temp;
        
        if temp > 75 then
            act hvac::cooling(on)
        else if temp < 65 then
            act hvac::heating(on)
        else
            act hvac::idle();
        
        sleep(10s)
    }
}
```

### 12.2 Stock Price Monitor

```
workflow stock_monitor {
    observe market:stock where symbol = "AAPL" as initial;
    
    loop {
        sleep(60s);
        observe market:stock where symbol = "AAPL" as current;
        
        let change = (current.price - initial.price) / initial.price * 100;
        
        if abs(change) > 5 then
            act alert::price_movement("AAPL", change)
        else
            act log::price("AAPL", current.price)
    }
}
```

### 12.3 Agent with Environment

```
workflow agent_workflow observes agent:environment {
    -- Sample initial state
    observe agent:environment as env;
    
    if env.workload > 0.8 then
        act queue::defer()
    else {
        act process::task(env.next_task);
        
        -- Re-check after processing
        observe agent:environment as updated_env;
        act log::completed(updated_env.tasks_completed)
    }
}
```

## 13. Implementation Notes

### 13.1 No Built-in Caching

Providers may implement caching, but workflows must not rely on it:

```rust
// Provider may cache, but workflow can't assume it
impl BehaviourProvider for CachedProvider {
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        // Check cache first
        if let Some(cached) = self.cache.get(constraints) {
            if cached.is_fresh() {
                return Ok(cached.value);
            }
        }
        
        // Fetch fresh value
        let value = self.fetch(constraints).await;
        self.cache.store(constraints, value.clone());
        Ok(value)
    }
}
```

### 13.2 Thread Safety

Behaviours are sampled asynchronously:
- Provider must be `Send + Sync`
- Sample operation is async
- Multiple workflows can sample same behaviour

## 14. Future Extensions

- **Interpolated behaviours**: Sample between discrete updates
- **Historical values**: `observe sensor:temperature at time-5min`
- **Aggregated behaviours**: `observe avg(sensor:temperature over 5min)`
- **Behaviour streams**: Convert behaviour changes to stream events
