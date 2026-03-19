# Appendix B: Provider Implementation Summary

This appendix provides a complete reference for all capability providers needed to run the examples, organized by implementation priority.

---

## Essential Providers (Must Have)

These providers are required for Level 1-3 examples and form the foundation of the capability system.

### ConsoleProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | SettableBehaviourProvider |
| **Capability** | `console:out`, `console:err` |
| **Purpose** | Write output to stdout/stderr |

**Usage**:
```ash
workflow hello {
    set console:out = "Hello, World!"
}
```

**Key Methods**:
- `set(channel, value)` - Write value to specified channel

---

### MockSensorProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Capability** | `sensor:temperature`, `sensor:pressure`, etc. |
| **Purpose** | Simulated sensor readings for testing |

**Usage**:
```ash
workflow monitor {
    observe sensor:temperature as temp;
    if temp > 100 then
        set console:out = "Overheating!"
}
```

**Key Methods**:
- `sample(constraints)` - Returns configured test value
- `has_changed(constraints)` - Returns true if value changed

**Configuration**:
- Set value via constructor or method
- Optional variation simulation

---

### FileSystemProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider + SettableBehaviourProvider |
| **Capability** | `fs:read`, `fs:write` |
| **Purpose** | Read from and write to files |

**Usage**:
```ash
workflow process {
    observe fs:read with path: "/input.txt" as content;
    let upper = content.uppercase();
    set fs:write with path: "/output.txt" = upper
}
```

**Key Methods**:
- `sample(constraints)` - Read file at given path
- `set(channel, value, constraints)` - Write to file

---

### InMemoryQueueProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | StreamProvider + SendableStreamProvider |
| **Capability** | `queue:orders`, `queue:events`, etc. |
| **Purpose** | In-memory message queue for testing |

**Usage**:
```ash
workflow processor {
    receive wait {
        queue:orders as order =>
            act process_order(order)
    }
}
```

**Key Methods**:
- `try_recv()` - Non-blocking receive
- `recv()` - Blocking receive
- `send(channel, value)` - Send to queue
- `is_closed()` - Check if queue closed

**Configuration**:
- Queue capacity
- Overflow strategy (drop oldest/newest)

---

### SQLiteProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider + SettableBehaviourProvider |
| **Capability** | `db:query`, `db:insert`, `db:update` |
| **Purpose** | Embedded database access |

**Usage**:
```ash
workflow query {
    observe db:query with sql: "SELECT * FROM users" as users;
    set db:insert with table: "logs" = { action: "query", count: len(users) }
}
```

**Key Methods**:
- `sample(constraints)` - Execute SELECT query
- `set(channel, value, constraints)` - Execute INSERT/UPDATE

---

## Testing Providers (High Priority)

These providers enable realistic testing scenarios.

### MockEmailProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | SendableStreamProvider |
| **Capability** | `email:send` |
| **Purpose** | Capture emails for testing |

**Features**:
- Store sent emails in memory
- Query sent emails for assertions
- Simulate failures

---

### MockPaymentProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Capability** | `payment:charge`, `payment:refund` |
| **Purpose** | Simulate payment processing |

**Features**:
- Configure success/failure responses
- Track transaction history
- Simulate network delays

---

### MockCRMProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider + SettableBehaviourProvider |
| **Capability** | `crm:customer`, `crm:ticket` |
| **Purpose** | Simulate CRM system |

---

### MockHTTPProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | StreamProvider + SendableStreamProvider |
| **Capability** | `http:requests`, `http:responses` |
| **Purpose** | Mock HTTP backend |

**Features**:
- Configure route responses
- Simulate latency
- Simulate failures

---

### MQTTMockProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | StreamProvider |
| **Capability** | `mqtt:sensors` |
| **Purpose** | Simulate MQTT broker |

---

### MockSMSProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | SendableStreamProvider |
| **Capability** | `sms:send` |
| **Purpose** | Capture SMS messages |

---

## Utility Providers (Medium Priority)

### TimeProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Capability** | `time:now`, `time:timestamp` |
| **Purpose** | Access system time |

**Features**:
- Current timestamp
- Formatted date/time strings
- Configurable for testing (mock time)

---

### CSVProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Capability** | `csv:parse`, `csv:generate` |
| **Purpose** | CSV file processing |

---

### JSONTransformProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | Function (not a capability) |
| **Capability** | N/A - called as function |
| **Purpose** | JSON transformations |

---

### SystemMetricsProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Capability** | `metrics:cpu`, `metrics:memory`, `metrics:disk` |
| **Purpose** | System resource monitoring |

---

### FileAuditProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | SettableBehaviourProvider |
| **Capability** | `audit:log` |
| **Purpose** | Write audit records to file |

**Features**:
- Append-only log format
- Timestamp automatically
- JSON or structured text output

---

## Advanced Providers (Low Priority)

These demonstrate advanced patterns and are needed for Level 6-7 examples.

### CircuitBreakerProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | Wrapper around other providers |
| **Purpose** | Fault tolerance wrapper |

**Features**:
- Track failure counts
- Open circuit after threshold
- Half-open state for recovery
- Configurable timeouts

---

### RateLimiterProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | Wrapper |
| **Purpose** | Rate limiting for requests |

**Features**:
- Token bucket algorithm
- Configurable rate
- Per-client tracking

---

### AgentCoordinatorProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | Custom |
| **Purpose** | Coordinate multiple parallel agents |

---

### SagaCoordinatorProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | Custom |
| **Purpose** | Manage distributed transactions |

**Features**:
- Track saga steps
- Execute compensations on failure
- Persist saga state

---

### MockMLProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Purpose** | Simulate ML model inference |

---

### FeatureFlagProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Purpose** | Feature flag evaluation |

---

### MockFraudProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Purpose** | Fraud risk scoring |

---

### RedisMockProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider + SettableBehaviourProvider |
| **Purpose** | In-memory key-value store |

---

### RulesEngineProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Purpose** | Evaluate business rules |

---

### ReportGeneratorProvider
| Attribute | Value |
|-----------|-------|
| **Trait** | SendableStreamProvider |
| **Purpose** | Generate compliance reports |

---

## Provider Trait Summary

```rust
// Read-only observable values
#[async_trait]
pub trait BehaviourProvider: Send + Sync {
    fn capability_name(&self) -> &str;
    fn channel_name(&self) -> &str;
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value>;
    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool>;
}

// Read-write observable values
#[async_trait]
pub trait SettableBehaviourProvider: BehaviourProvider + Send + Sync {
    async fn set(&self, value: Value, constraints: &[Constraint]) -> ExecResult<()>;
}

// Read-only event streams
#[async_trait]
pub trait StreamProvider: Send + Sync {
    fn capability_name(&self) -> &str;
    fn channel_name(&self) -> &str;
    fn try_recv(&self) -> Option<ExecResult<Value>>;
    async fn recv(&self) -> ExecResult<Value>;
    fn is_closed(&self) -> bool;
}

// Read-write event streams
#[async_trait]
pub trait SendableStreamProvider: StreamProvider + Send + Sync {
    async fn send(&self, value: Value, constraints: &[Constraint]) -> ExecResult<()>;
}
```

---

## TDD/Process Providers (for 40_tdd_workflow)

These providers support the TDD workflow orchestration example.

### TaskManager
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider + SettableBehaviourProvider |
| **Capability** | `task:fetch`, `task:update` |
| **Purpose** | Manage development tasks |

### TestRunner
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider |
| **Capability** | `test:run` |
| **Purpose** | Execute test suites and return results |

### CodeRepo
| Attribute | Value |
|-----------|-------|
| **Trait** | BehaviourProvider + SettableBehaviourProvider |
| **Capability** | `code:read`, `code:write` |
| **Purpose** | Read and write source code |

### ReviewSystem
| Attribute | Value |
|-----------|-------|
| **Trait** | StreamProvider + SettableBehaviourProvider |
| **Capability** | `review:assign`, `review:submit`, `review:read` |
| **Purpose** | Manage code review workflow |

---

## Implementation Priority Summary

| Priority | Providers | Count |
|----------|-----------|-------|
| **Essential** | Console, MockSensor, FileSystem, InMemoryQueue, SQLite | 5 |
| **High** | MockEmail, MockPayment, MockCRM, MockHTTP, MQTTMock, MockSMS | 6 |
| **Medium** | Time, CSV, JSONTransform, SystemMetrics, FileAudit | 5 |
| **TDD/Process** | TaskManager, TestRunner, CodeRepo, ReviewSystem | 4 |
| **Advanced** | CircuitBreaker, RateLimiter, AgentCoordinator, SagaCoordinator, MockML, FeatureFlag, MockFraud, RedisMock, RulesEngine, ReportGenerator | 10 |
| **Total** | | **30** |

---

## Provider Dependencies

```
Essential (Level 1-3)
    ├── ConsoleProvider
    ├── MockSensorProvider
    ├── FileSystemProvider
    ├── InMemoryQueueProvider
    └── SQLiteProvider

Testing (Level 4-5)
    ├── MockEmailProvider
    ├── MockPaymentProvider
    ├── MockCRMProvider
    ├── MockHTTPProvider
    ├── MQTTMockProvider
    └── MockSMSProvider

Utility (Level 3-5)
    ├── TimeProvider
    ├── CSVProvider
    ├── JSONTransformProvider
    ├── SystemMetricsProvider
    └── FileAuditProvider

TDD/Process (Level 5)
    ├── TaskManager
    ├── TestRunner
    ├── CodeRepo
    └── ReviewSystem

Advanced (Level 6-7)
    ├── CircuitBreakerProvider (wraps others)
    ├── RateLimiterProvider (wraps others)
    ├── AgentCoordinatorProvider
    ├── SagaCoordinatorProvider
    ├── MockMLProvider
    ├── FeatureFlagProvider
    ├── MockFraudProvider
    ├── RedisMockProvider
    ├── RulesEngineProvider
    └── ReportGeneratorProvider
```
