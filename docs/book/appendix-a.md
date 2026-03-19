# Appendix A: Example Requirements Summary

This appendix details all 28 example workflows, their complexity levels, required providers, and the concepts they demonstrate.

---

## Level 1: Basic Workflows (Getting Started)

### 01_hello_world.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 5-10 |
| **Provider** | ConsoleProvider |
| **Concepts** | `set`, literals |

Simplest possible workflow that outputs "Hello, World!" to the console.

### 02_variables.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 10-15 |
| **Provider** | None (pure computation) |
| **Concepts** | `let`, types, variable binding |

Demonstrates variable declaration, assignment, and basic types.

### 03_control_flow.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 15-25 |
| **Provider** | None (pure computation) |
| **Concepts** | `if`, `for`, conditionals |

Shows conditional execution and iteration patterns.

### 04_patterns.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 15-25 |
| **Provider** | None (pure computation) |
| **Concepts** | destructuring, tuple patterns, record patterns |

Pattern matching basics with tuples, records, and lists.

---

## Level 2: OODA Pattern (Core Concepts)

### 05_temperature_monitor.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 20-40 |
| **Provider** | MockSensorProvider |
| **Concepts** | `observe`, `decide`, effect inference |

Simple observation from a sensor with a decision based on the value.

### 06_data_processor.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 25-50 |
| **Provider** | None (or ConsoleProvider for output) |
| **Concepts** | `orient`, expression evaluation |

Transforms data through the orient phase.

### 07_policy_guard.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 30-60 |
| **Provider** | None (demonstrates policy syntax) |
| **Concepts** | `decide`, policy definitions, guards |

Basic policy enforcement with permit/deny decisions.

### 08_action_sequence.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 30-50 |
| **Provider** | ConsoleProvider |
| **Concepts** | `act`, sequential composition |

Chains multiple actions together.

---

## Level 3: Capabilities (External Integration)

### 09_file_processor.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 30-60 |
| **Provider** | FileSystemProvider |
| **Concepts** | `observe` file, `set` output, file I/O |

Reads from a file, processes content, writes result.

### 10_event_counter.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 30-60 |
| **Provider** | InMemoryQueueProvider |
| **Concepts** | `receive`, state management, counting |

Processes events from a queue and maintains state.

### 11_sensor_dashboard.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 40-80 |
| **Providers** | SystemMetricsProvider, TimeProvider |
| **Concepts** | multiple observes, combining data |

Reads from multiple sensors and aggregates data.

### 12_message_router.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 40-80 |
| **Provider** | InMemoryQueueProvider |
| **Concepts** | `receive`, `send`, routing logic |

Receives messages and routes them to different destinations.

---

## Level 4: Governance (Policies & Roles)

### 13_expense_approval.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 50-90 |
| **Provider** | MockEmailProvider |
| **Concepts** | roles, `check`, approval workflow |

Expense request with role-based approval and email notification.

### 14_access_control.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 40-80 |
| **Provider** | None (demonstrates role system) |
| **Concepts** | role hierarchies, authority checks |

Demonstrates hierarchical roles and access control.

### 15_audit_logger.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 50-100 |
| **Provider** | FileAuditProvider |
| **Concepts** | provenance, audit trails |

Records all actions to an audit log.

### 16_multi_policy.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 60-120 |
| **Provider** | None (demonstrates policy combinators) |
| **Concepts** | policy combinators (and, or, not) |

Combines multiple policies using logical operators.

---

## Level 5: Real-World Systems (Production Examples)

### 17_ecommerce_order.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 80-150 |
| **Providers** | SQLiteProvider, MockPaymentProvider |
| **Concepts** | full pipeline, transactions, error handling |

Complete order processing: inventory check, payment, fulfillment.

### 18_customer_support.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 80-150 |
| **Providers** | MockCRMProvider, MockEmailProvider |
| **Concepts** | escalation, SLA tracking |

Ticket routing based on priority with escalation paths.

### 19_iot_monitoring.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 80-150 |
| **Providers** | MQTTMockProvider, MockSMSProvider |
| **Concepts** | alerting, threshold monitoring |

Sensor data monitoring with SMS alerts for anomalies.

### 20_data_pipeline.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 80-150 |
| **Providers** | CSVProvider, JSONTransformProvider |
| **Concepts** | ETL, error handling, retries |

Extract-transform-load pipeline with format conversion.

### 40_tdd_workflow.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 300-500 |
| **Providers** | task_manager, test_runner, code_repo, review_system |
| **Concepts** | Multi-role orchestration, TDD lifecycle, obligation tracking, iterative development |

Complete Test-Driven Development workflow with developer, tester, and reviewer roles. Demonstrates red/green/refactor cycles, policy enforcement, and code review process.

**Files**:
- `40_tdd_workflow.ash` - Generic TDD workflow template
- `40a_tdd_concrete_example.ash` - Concrete Stack implementation example

---

## Level 6: Advanced Patterns (Complex Scenarios)

### 21_multi_agent.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 100-180 |
| **Provider** | AgentCoordinatorProvider |
| **Concepts** | `par`, parallel execution, coordination |

Multiple agents working in parallel with result aggregation.

### 22_state_machine.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 100-180 |
| **Provider** | SQLiteProvider |
| **Concepts** | state persistence, transitions |

Workflow as a state machine with persisted state.

### 23_circuit_breaker.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 100-180 |
| **Provider** | CircuitBreakerProvider |
| **Concepts** | resilience, failure handling |

Fault tolerance using the circuit breaker pattern.

### 24_saga_pattern.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 120-200 |
| **Providers** | SagaCoordinatorProvider, MockPaymentProvider |
| **Concepts** | distributed transactions, compensations |

Long-running transaction with rollback on failure.

---

## Level 7: Complete Applications (Full Systems)

### 25_api_gateway.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 150-300 |
| **Providers** | RateLimiterProvider, MockHTTPProvider |
| **Concepts** | rate limiting, request routing |

API gateway with rate limiting and backend routing.

### 26_ml_inference.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 150-300 |
| **Providers** | MockMLProvider, FeatureFlagProvider |
| **Concepts** | A/B testing, model versioning |

Model serving with A/B testing and feature flags.

### 27_fraud_detection.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 150-300 |
| **Providers** | MockFraudProvider, RedisMockProvider |
| **Concepts** | real-time scoring, blocking |

Real-time fraud detection with risk scoring.

### 28_compliance_checker.ash
| Attribute | Value |
|-----------|-------|
| **Lines** | 150-300 |
| **Providers** | RulesEngineProvider, ReportGeneratorProvider |
| **Concepts** | regulatory validation, reporting |

Validates transactions against regulatory rules.

---

## Summary by Provider

| Provider | Examples Using It | Priority |
|----------|-------------------|----------|
| ConsoleProvider | 01, 08, many for debugging | Essential |
| MockSensorProvider | 05, 11, 19 | Essential |
| FileSystemProvider | 09, 15 | Essential |
| InMemoryQueueProvider | 10, 12, 21 | Essential |
| SQLiteProvider | 17, 22 | High |
| MockEmailProvider | 13, 18 | High |
| TimeProvider | 11, 25 | Medium |
| MockPaymentProvider | 17, 24 | Medium |
| MockHTTPProvider | 20, 25 | Medium |
| MQTTMockProvider | 19 | Medium |
| MockSMSProvider | 19 | Medium |
| SystemMetricsProvider | 11 | Medium |
| FileAuditProvider | 15 | Medium |
| CSVProvider | 20 | Medium |
| JSONTransformProvider | 20 | Medium |
| MockCRMProvider | 18 | Medium |
| AgentCoordinatorProvider | 21 | Advanced |
| CircuitBreakerProvider | 23 | Advanced |
| SagaCoordinatorProvider | 24 | Advanced |
| RateLimiterProvider | 25 | Advanced |
| MockMLProvider | 26 | Advanced |
| FeatureFlagProvider | 26 | Advanced |
| MockFraudProvider | 27 | Advanced |
| RedisMockProvider | 27 | Advanced |
| RulesEngineProvider | 28 | Advanced |
| ReportGeneratorProvider | 28 | Advanced |

---

## Complexity Distribution

| Level | Examples | Lines Each | Total Lines |
|-------|----------|------------|-------------|
| 1 | 4 | 10-25 | 60-100 |
| 2 | 4 | 20-60 | 160-280 |
| 3 | 4 | 30-80 | 280-560 |
| 4 | 4 | 40-120 | 400-720 |
| 5 | 4 | 80-150 | 640-1200 |
| 6 | 4 | 100-200 | 800-1600 |
| 7 | 4 | 150-300 | 1200-2400 |
| **Total** | **28** | | **3540-6860** |
