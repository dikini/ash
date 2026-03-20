# SPEC-015: Typed Providers (Runtime Type Safety)

## Status: Draft

## 1. Overview

Providers (behaviours and streams) declare their Ash type schema at registration. This enables runtime type validation at the Rust/Ash boundary, ensuring providers return values that match workflow expectations.

## 2. Motivation

Without type validation:
```rust
// Provider returns Int
impl BehaviourProvider for BadProvider {
    async fn sample(&self, _) -> ExecResult<Value> {
        Ok(Value::Int(42))  // Just a number
    }
}
```

But workflow expects:
```ash
observe sensor:temp as { value: Int, unit: String }
--                         ^^^^^^^^^^^^^^^^^^^^^
--                         Expects Record with 2 fields
```

**Runtime error**: Pattern match fails, workflow crashes or misbehaves.

With typed providers:
- Schema declared at registration
- Runtime validation on every sample/recv
- Clear error messages at the boundary

## 3. Type Schema Declaration

### 3.1 Basic Usage

```rust
use ash_engine::{TypedBehaviourProvider, Type, Registry};

// Declare schema
const TEMP_SCHEMA: Type = Type::Record(vec![
    ("value".into(), Type::Int),
    ("unit".into(), Type::String),
]);

// Create provider with schema
let provider = TypedBehaviourProvider::new(
    || async { Value::Record(hashmap! {...}) },
    TEMP_SCHEMA,
);

// Register - schema stored for validation
registry.register("sensor", "temp", provider)?;
```

### 3.2 Schema Types

```rust
pub enum Type {
    Int,
    String,
    Bool,
    Null,
    Time,
    Ref,
    List(Box<Type>),
    Record(Vec<(Box<str>, Type)>),
    Cap { name: Box<str>, effect: Effect },
}
```

### 3.3 Validation

```rust
impl TypedBehaviourProvider {
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.inner.sample(constraints).await?;
        
        // Runtime validation
        if !self.schema.matches(&value) {
            return Err(ExecError::TypeMismatch {
                provider: self.name(),
                expected: self.schema.to_string(),
                actual: value.to_string(),
            });
        }
        
        Ok(value)
    }
}
```

## 4. API Design

### 4.1 Typed Provider Wrapper

```rust
/// A behaviour provider with declared type schema
pub struct TypedBehaviourProvider {
    inner: Box<dyn BehaviourProvider>,
    schema: Type,
    name: String,
}

impl TypedBehaviourProvider {
    /// Create with runtime validation
    pub fn new<P>(provider: P, schema: Type) -> Self
    where
        P: BehaviourProvider + 'static,
    {
        Self {
            inner: Box::new(provider),
            schema,
            name: format!("{}:{}", provider.capability_name(), provider.channel_name()),
        }
    }
    
    pub fn schema(&self) -> &Type;
    pub fn validate_value(&self, value: &Value) -> Result<(), TypeError>;
}
```

### 4.2 Registry Integration

```rust
pub struct BehaviourRegistry {
    providers: HashMap<(Name, Name), TypedBehaviourProvider>,
    schemas: HashMap<(Name, Name), Type>,
}

impl BehaviourRegistry {
    pub fn register(&mut self, provider: TypedBehaviourProvider) {
        let key = (provider.capability_name().into(), provider.channel_name().into());
        self.schemas.insert(key.clone(), provider.schema().clone());
        self.providers.insert(key, provider);
    }
    
    /// Get schema for type checking workflows
    pub fn get_schema(&self, cap: &str, channel: &str) -> Option<&Type>;
}
```

### 4.3 IntoValue Trait (Optional Helper)

```rust
/// Convert Rust types to Ash Values with schema validation
pub trait IntoValue {
    fn into_value(self) -> Value;
    fn ash_type() -> Type;
}

impl IntoValue for i64 {
    fn into_value(self) -> Value { Value::Int(self) }
    fn ash_type() -> Type { Type::Int }
}

impl IntoValue for String {
    fn into_value(self) -> Value { Value::String(self) }
    fn ash_type() -> Type { Type::String }
}

// Derive macro for structs
#[derive(IntoValue)]
#[ash_type(schema = "TEMP_SCHEMA")]
struct Temperature {
    value: i64,
    unit: String,
}
```

## 5. Type Checking Flow

```
Workflow: observe sensor:temp as { value, unit }
                |
                v
Type checker: Look up schema for sensor:temp
                |---> Schema: { value: Int, unit: String }
                |
                v
Type checker: Pattern { value, unit } matches schema?
                |---> Yes: value is Int, unit is String
                |
                v
Execution: provider.sample()
                |
                v
Runtime validation: Does returned Value match schema?
                |---> Yes: continue
                |---> No: ExecError::TypeMismatch
```

## 6. Error Messages

```
Type Mismatch in provider 'sensor:temp'
  Expected: { value: Int, unit: String }
  Actual:   Int(42)
  
  Hint: Provider returned Int instead of Record
```

## 7. Relationship to Schema-First (Option 2)

This spec implements **Option 1** (runtime validation).

**Option 2** (code generation) builds on this:
```rust
// Option 2 generates this from Ash schema:
generated! {
    schema Temperature {
        value: Int,
        unit: String,
    }
}

// Generates:
// 1. const TEMPERATURE_SCHEMA: Type
// 2. struct Temperature { value: i64, unit: String }
// 3. impl IntoValue for Temperature
```

Option 2 is future work. Option 1 gives us type safety now.

## 8. Examples

### 8.1 Sensor Provider

```rust
const SENSOR_SCHEMA: Type = Type::Record(vec![
    ("value".into(), Type::Int),
    ("unit".into(), Type::String),
    ("timestamp".into(), Type::Time),
]);

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

// Registration
let provider = TypedBehaviourProvider::new(
    SensorProvider::new(sensor),
    SENSOR_SCHEMA,
);
registry.register(provider)?;
```

### 8.2 Market Data Provider

```rust
const STOCK_SCHEMA: Type = Type::Record(vec![
    ("symbol".into(), Type::String),
    ("price".into(), Type::Int),  // cents
    ("volume".into(), Type::Int),
    ("timestamp".into(), Type::Time),
]);

let provider = TypedBehaviourProvider::new(
    || async {
        let data = fetch_stock_data("AAPL").await;
        Value::Record(hashmap! {
            "symbol".into() => Value::String(data.symbol),
            "price".into() => Value::Int(data.price_cents),
            "volume".into() => Value::Int(data.volume),
            "timestamp".into() => Value::Time(data.timestamp),
        })
    },
    STOCK_SCHEMA,
);
```

### 8.3 Stream Provider

```rust
const ORDER_SCHEMA: Type = Type::Record(vec![
    ("id".into(), Type::String),
    ("customer".into(), Type::String),
    ("total".into(), Type::Int),
]);

let stream_provider = TypedStreamProvider::new(
    kafka_consumer,
    ORDER_SCHEMA,
);
stream_registry.register(stream_provider)?;
```

## 9. Implementation Tasks

- TASK-096: Typed provider wrapper structs
- TASK-097: Schema validation logic
- TASK-098: Registry integration with schemas
- TASK-099: Runtime validation in sample/recv
- TASK-100: Error reporting for type mismatches

## 10. Future: Schema-First Code Generation

After this is working, add:

```bash
# Generate Rust types from Ash schemas
$ ash generate --schema sensor.ash --output src/providers/

# Generates:
# - src/providers/sensor_schema.rs (const SCHEMA)
# - src/providers/sensor_types.rs (Rust structs)
# - src/providers/sensor_provider.rs (trait)
```

The schema-first code-generation path remains future work and does not change the runtime
validation contract defined in this spec.
