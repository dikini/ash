# Ash Language Documentation

This is the comprehensive documentation for the Ash workflow language - a language for governed AI systems.

---

## Part I: Introduction & Getting Started

- [1. Introduction](01-introduction.md)
  - What is Ash? (workflow language for governed AI systems)
  - The OODA pattern (Observe-Orient-Decide-Act)
  - Effect system at a glance (Epistemic < Deliberative < Evaluative < Operational)
  - Core concepts: Capabilities, Policies, Roles
  - Ash as an embeddable DSL

- [2. Installation & Quick Start](02-quickstart.md)
  - Installing the CLI (`ash`)
  - Available commands: `check`, `run`, `trace`, `repl`, `dot`
  - Your first workflow
  - Running with `ash run`
  - Using the REPL

---

## Part II: The OODA Pattern (Ash's Core)

- [3. Observe - Reading from the World](03-observe.md)
  - The `observe` statement for behaviours (continuous values)
  - The `receive` statement for streams (discrete events)
  - Pattern matching on observations
  - Guards and constraints
  - Effect: Epistemic

- [4. Orient - Analysis & Transformation](04-orient.md)
  - The `orient` block
  - Transforming data
  - The `propose` statement
  - Effect: Deliberative

- [5. Decide - Governance & Policy](05-decide.md)
  - The `decide` statement
  - Policy evaluation
  - Decision outcomes (permit, deny, require_approval, escalate)
  - The `check` statement
  - Effect: Evaluative

- [6. Act - Side Effects](06-act.md)
  - The `act` statement
  - Guards on actions
  - Effect: Operational

---

## Part III: Capabilities (External Interfaces)

- [7. Understanding Capabilities](07-capabilities-overview.md)
  - What are capabilities? (external resource interfaces)
  - Four capability operations: `observe`, `set`, `receive`, `send`
  - Declaring capabilities
  - Capability safety (must declare to use)

- [8. Behaviours (Observable Values)](08-behaviours.md)
  - Sampling with `observe`
  - Setting values with `set`
  - Change detection
  - Use cases: sensors, configuration, state

- [9. Streams (Event Processing)](09-streams.md)
  - Receiving events with `receive`
  - Sending events with `send`
  - Pattern matching with guards
  - Blocking vs non-blocking modes
  - Timeouts
  - Mailbox model and overflow strategies

- [10. Capability Integration](10-capability-integration.md)
  - Effect tracking for all capabilities
  - Policy evaluation for input/output
  - Provenance tracking
  - Type safety at provider boundary

---

## Part IV: Governance & Safety

- [11. Understanding Policies](11-policies.md)
  - Policy declarations
  - Policy combinators (and, or, not, implies, forall, exists)
  - Decision types
  - Policy evaluation context

- [12. Understanding Roles & Obligations](12-roles.md)
  - Role declarations (authority, obligations)
  - Flat named approval roles
  - The `check` statement for obligations

- [13. The Effect System](13-effect-system.md)
  - The effect lattice
  - Effect inference
  - Effect safety guarantees

---

## Part V: Language Features

- [14. Basic Syntax & Types](14-basic-syntax.md)
  - Literals (int, float, string, bool, null)
  - Variables and `let` bindings
  - Records and lists
  - Pattern matching (tuple, record, list destructuring)
  - Comments

- [15. Expressions & Operators](15-expressions.md)
  - Arithmetic, comparison, logical operators
  - Operator precedence
  - Field access and indexing
  - Function calls

- [16. Control Flow](16-control-flow.md)
  - Sequential composition
  - Conditionals (`if-then-else`)
  - Loops (`for`)
  - Parallel execution (`par`)
  - Error handling (`attempt`, `retry`, `timeout`)

- [17. Modules & Visibility](17-modules.md)
  - Module declarations (`mod`)
  - File-based vs inline modules
  - Visibility modifiers (`pub`, `pub(crate)`, `pub(super)`)
  - Path resolution (`crate::`, `super::`, `self::`)
  - Import statements (`use`)

---

## Part VI: Embedding Ash in Rust

- [18. Getting Started with Embedding](18-embedding-basics.md)
  - The `ash-engine` crate
  - Basic embedding pattern
  - Creating an Engine instance
  - Parsing and executing workflows from Rust

- [19. Setting the Environment/Context](19-setting-context.md)
  - Creating the runtime context
  - Registering capability providers
  - Configuring the type environment
  - Setting up policy evaluators
  - Managing provenance trackers
  - Example: Custom context for testing vs production

- [20. Extending the Runtime with Capabilities](20-extending-runtime.md)
  - The capability provider traits:
    - `CapabilityProvider` (legacy)
    - `BehaviourProvider` / `SettableBehaviourProvider`
    - `StreamProvider` / `SendableStreamProvider`
  - Implementing a custom behaviour provider
  - Implementing a custom stream provider
  - Typed providers with runtime validation
  - Provider registration and lookup
  - Example: Building a database capability provider
  - Example: Building a message queue capability provider

- [21. Advanced Embedding Patterns](21-advanced-embedding.md)
  - Sandboxing workflows
  - Custom error handling
  - Async execution patterns
  - Managing workflow lifecycle
  - Hot-reloading workflows

---

## Part VII: Tooling

- [22. CLI Reference](22-cli-reference.md)
  - `ash check` - type checking
  - `ash run` - executing workflows
  - `ash trace` - execution with provenance
  - `ash repl` - interactive development
  - `ash dot` - AST visualization
  - Note: `fmt` and `lsp` are planned but not implemented

- [23. REPL Guide](23-repl-guide.md)
  - Starting the REPL
  - Commands (`:help`, `:type`, `:quit`)
  - Multiline input

- [24. Configuration](24-configuration.md)
  - `.ash.toml` configuration file
  - Environment variables
  - Capability bindings

---

## Part VIII: Example Workflows

- [25. Level 1: Basic Workflows](25-examples-level1.md)
  - `01_hello_world.ash`
  - `02_variables.ash`
  - `03_control_flow.ash`
  - `04_patterns.ash`

- [26. Level 2: OODA Pattern](26-examples-level2.md)
  - `05_temperature_monitor.ash`
  - `06_data_processor.ash`
  - `07_policy_guard.ash`
  - `08_action_sequence.ash`

- [27. Level 3: Capabilities](27-examples-level3.md)
  - `09_file_processor.ash`
  - `10_event_counter.ash`
  - `11_sensor_dashboard.ash`
  - `12_message_router.ash`

- [28. Level 4: Governance](28-examples-level4.md)
  - `13_expense_approval.ash`
  - `14_access_control.ash`
  - `15_audit_logger.ash`
  - `16_multi_policy.ash`

- [29. Level 5: Real-World Systems](29-examples-level5.md)
  - `17_ecommerce_order.ash`
  - `18_customer_support.ash`
  - `19_iot_monitoring.ash`
  - `20_data_pipeline.ash`
  - `40_tdd_workflow.ash` - Test-Driven Development process orchestration

- [30. Level 6: Advanced Patterns](30-examples-level6.md)
  - `21_multi_agent.ash`
  - `22_state_machine.ash`
  - `23_circuit_breaker.ash`
  - `24_saga_pattern.ash`

- [31. Level 7: Complete Applications](31-examples-level7.md)
  - `25_api_gateway.ash`
  - `26_ml_inference.ash`
  - `27_fraud_detection.ash`
  - `28_compliance_checker.ash`

---

## Part IX: Example Provider Implementations

- [32. Provider Development Guide](32-provider-dev-guide.md)
  - When to write a custom provider
  - Choosing the right trait
  - Provider lifecycle and state management
  - Error handling strategies
  - Testing providers

- [33. Built-in Example Providers](33-builtin-providers.md)
  - `ConsoleProvider`
  - `FileSystemProvider`
  - `InMemoryQueueProvider`
  - `TimeProvider`
  - `SQLiteProvider`

- [34. Mock Providers for Testing](34-mock-providers.md)
  - `MockSensorProvider`
  - `MockEmailProvider`
  - `MockPaymentProvider`
  - `MockHTTPProvider`

- [35. Advanced Provider Patterns](35-advanced-patterns.md)
  - Composing providers
  - Caching providers
  - Rate-limiting wrappers
  - Circuit breaker pattern
  - Provider discovery

---

## Part X: Practical Topics

- [36. Common Patterns](36-common-patterns.md)
  - Request-response workflows
  - Event processors
  - State machines
  - Multi-agent coordination
  - Error recovery patterns

- [37. Testing Workflows](37-testing.md)
  - Using mock providers
  - Testing in isolation
  - Integration testing with real providers

- [38. Performance Tuning](38-performance.md)
  - Effect inference optimization
  - Provider caching strategies
  - Parallel execution tuning

---

## Part XI: Reference

- [39. Language Grammar (EBNF)](39-grammar.md)

- [40. API Reference](40-api-reference.md)

- [41. Error Messages Guide](41-errors.md)

- [42. Glossary](42-glossary.md)

---

## Appendix

- [A. Example Requirements Summary](appendix-a.md)
- [B. Provider Implementation Summary](appendix-b.md)
- [C. File Structure](appendix-c.md)
