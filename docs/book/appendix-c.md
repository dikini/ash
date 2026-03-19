# Appendix C: File Structure

This appendix describes the complete file structure for the Ash documentation and examples.

---

## Documentation Structure

```
docs/
├── book/                          # mdBook-style documentation
│   ├── SUMMARY.md                 # Table of contents (this file)
│   ├── 01-introduction.md
│   ├── 02-quickstart.md
│   ├── 03-observe.md
│   ├── 04-orient.md
│   ├── 05-decide.md
│   ├── 06-act.md
│   ├── 07-capabilities-overview.md
│   ├── 08-behaviours.md
│   ├── 09-streams.md
│   ├── 10-capability-integration.md
│   ├── 11-policies.md
│   ├── 12-roles.md
│   ├── 13-effect-system.md
│   ├── 14-basic-syntax.md
│   ├── 15-expressions.md
│   ├── 16-control-flow.md
│   ├── 17-modules.md
│   ├── 18-embedding-basics.md
│   ├── 19-setting-context.md
│   ├── 20-extending-runtime.md
│   ├── 21-advanced-embedding.md
│   ├── 22-cli-reference.md
│   ├── 23-repl-guide.md
│   ├── 24-configuration.md
│   ├── 25-examples-level1.md
│   ├── 26-examples-level2.md
│   ├── 27-examples-level3.md
│   ├── 28-examples-level4.md
│   ├── 29-examples-level5.md
│   ├── 30-examples-level6.md
│   ├── 31-examples-level7.md
│   ├── 32-provider-dev-guide.md
│   ├── 33-builtin-providers.md
│   ├── 34-mock-providers.md
│   ├── 35-advanced-patterns.md
│   ├── 36-common-patterns.md
│   ├── 37-testing.md
│   ├── 38-performance.md
│   ├── 39-grammar.md
│   ├── 40-api-reference.md
│   ├── 41-errors.md
│   ├── 42-glossary.md
│   ├── appendix-a.md              # Example Requirements Summary
│   ├── appendix-b.md              # Provider Implementation Summary
│   └── appendix-c.md              # File Structure (this file)
│
├── spec/                          # Formal specifications
│   ├── SPEC-001-IR.md
│   ├── SPEC-002-SURFACE.md
│   ├── SPEC-003-TYPE-SYSTEM.md
│   ├── SPEC-004-SEMANTICS.md
│   ├── SPEC-005-CLI.md
│   └── ...
│
├── design/                        # Design documents
│   └── ARCHITECTURE.md
│
├── plan/                          # Implementation plans
│   ├── PLAN-INDEX.md
│   └── tasks/
│       └── ...
│
├── API.md                         # API reference
├── TUTORIAL.md                    # Quick tutorial
├── SHARO_CORE_LANGUAGE.md         # Language specification
└── README.md                      # Documentation index
```

---

## Examples Structure

```
examples/
├── README.md                      # Examples index and running guide
│
├── workflows/                     # All .ash workflow files
│   ├── 01_hello_world.ash
│   ├── 02_variables.ash
│   ├── 03_control_flow.ash
│   ├── 04_patterns.ash
│   ├── 05_temperature_monitor.ash
│   ├── 06_data_processor.ash
│   ├── 07_policy_guard.ash
│   ├── 08_action_sequence.ash
│   ├── 09_file_processor.ash
│   ├── 10_event_counter.ash
│   ├── 11_sensor_dashboard.ash
│   ├── 12_message_router.ash
│   ├── 13_expense_approval.ash
│   ├── 14_access_control.ash
│   ├── 15_audit_logger.ash
│   ├── 16_multi_policy.ash
│   ├── 17_ecommerce_order.ash
│   ├── 18_customer_support.ash
│   ├── 19_iot_monitoring.ash
│   ├── 20_data_pipeline.ash
│   ├── 21_multi_agent.ash
│   ├── 22_state_machine.ash
│   ├── 23_circuit_breaker.ash
│   ├── 24_saga_pattern.ash
│   ├── 25_api_gateway.ash
│   ├── 26_ml_inference.ash
│   ├── 27_fraud_detection.ash
│   ├── 28_compliance_checker.ash
│   ├── 40_tdd_workflow.ash        # TDD process orchestration
│   ├── 40a_tdd_concrete_example.ash
│   └── 40_tdd_README.md
│
├── providers/                     # Rust provider implementations
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── console.rs             # ConsoleProvider
│       ├── filesystem.rs          # FileSystemProvider
│       ├── inmemory_queue.rs      # InMemoryQueueProvider
│       ├── sqlite.rs              # SQLiteProvider
│       ├── time.rs                # TimeProvider
│       ├── system_metrics.rs      # SystemMetricsProvider
│       ├── csv.rs                 # CSVProvider
│       ├── json_transform.rs      # JSONTransformProvider
│       ├── file_audit.rs          # FileAuditProvider
│       │
│       └── mock/                  # Mock providers for testing
│           ├── mod.rs
│           ├── sensor.rs          # MockSensorProvider
│           ├── email.rs           # MockEmailProvider
│           ├── payment.rs         # MockPaymentProvider
│           ├── crm.rs             # MockCRMProvider
│           ├── http.rs            # MockHTTPProvider
│           ├── mqtt.rs            # MQTTMockProvider
│           ├── sms.rs             # MockSMSProvider
│           ├── ml.rs              # MockMLProvider
│           └── fraud.rs           # MockFraudProvider
│
├── data/                          # Sample data files
│   ├── sample_orders.json
│   ├── sample_sensors.csv
│   ├── sample_customers.json
│   ├── sample_transactions.csv
│   └── README.md                  # Data file descriptions
│
└── tests/                         # Integration tests
    ├── example_runner.rs          # Test all examples
    └── README.md                  # Testing guide
```

---

## Complete Repository Structure

```
ash/                              # Project root
│
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock
├── README.md                     # Project README
├── LICENSE-MIT
├── LICENSE-APACHE
├── CHANGELOG.md
├── AGENTS.md                     # AI agent guidelines
├── TOOLS.md                      # Tool descriptions
│
├── crates/                       # Rust crate packages
│   ├── ash-core/                 # Core types and IR
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ast.rs
│   │       ├── effect.rs
│   │       ├── value.rs
│   │       ├── provenance.rs
│   │       ├── stream.rs
│   │       └── ...
│   │
│   ├── ash-parser/               # Lexer and parser
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lexer.rs
│   │       ├── surface.rs
│   │       ├── parse_workflow.rs
│   │       └── ...
│   │
│   ├── ash-typeck/               # Type checker
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── ...
│   │
│   ├── ash-interp/               # Interpreter
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── behaviour.rs
│   │       ├── stream.rs
│   │       ├── capability.rs
│   │       └── ...
│   │
│   ├── ash-provenance/           # Audit trails
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── ...
│   │
│   ├── ash-engine/               # Embedding API
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── ...
│   │
│   ├── ash-repl/                 # Interactive REPL
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── ...
│   │
│   ├── ash-cli/                  # Command-line interface
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── check.rs
│   │           ├── run.rs
│   │           ├── trace.rs
│   │           ├── repl.rs
│   │           └── dot.rs
│   │
│   └── ...                       # Additional crates
│
├── docs/                         # Documentation (see above)
│   ├── book/
│   ├── spec/
│   ├── design/
│   └── plan/
│
├── examples/                     # Example workflows (see above)
│   ├── workflows/
│   ├── providers/
│   ├── data/
│   └── tests/
│
├── tests/                        # Integration tests
│   └── ...
│
├── scripts/                      # Utility scripts
│   └── install-hooks.sh
│
├── .githooks/                    # Git hooks
│   └── ...
│
└── .github/                      # GitHub configuration
    └── workflows/
        └── ...
```

---

## Key Files Reference

### Documentation
| File | Purpose |
|------|---------|
| `docs/book/SUMMARY.md` | Table of contents |
| `docs/book/*.md` | 42 documentation chapters |
| `docs/book/appendix-*.md` | Reference appendices |
| `docs/spec/SPEC-*.md` | Formal specifications |

### Examples
| Directory | Contents |
|-----------|----------|
| `examples/workflows/*.ash` | 28 runnable workflow files |
| `examples/providers/src/*.rs` | ~15 provider implementations |
| `examples/data/*` | Sample data files |

### Source Code
| Crate | Purpose |
|-------|---------|
| `ash-core` | IR, effects, values, provenance types |
| `ash-parser` | Lexer, parser, surface AST |
| `ash-typeck` | Type checker, effect inference |
| `ash-interp` | Interpreter, provider traits |
| `ash-engine` | Unified embedding API |
| `ash-repl` | Interactive REPL |
| `ash-cli` | Command-line tool |

---

## Building the Documentation

```bash
# Build mdBook (if using mdBook)
cd docs/book
mdbook build

# Or generate from SUMMARY.md
cargo doc --document-private-items

# Run all examples
cd examples
./run-all.sh

# Test specific example
ash run examples/workflows/01_hello_world.ash
```
