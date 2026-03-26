# Ash

**A reference implementation of the Sharo Core Language (SHC)**

Ash is an executable semantics and runtime for the Sharo Core workflow language, designed for governed AI systems with formal verification capabilities.

## Overview

Ash provides:
- **Parser**: Surface language → IR
- **Type Checker**: Effect tracking and obligation verification
- **Interpreter**: Big-step operational semantics
- **Provenance Tracker**: Complete audit trails
- **Policy Engine**: Deontic logic evaluation

## Project Structure

```
ash/
├── crates/
│   ├── ash-core/       # IR and semantics definitions
│   ├── ash-parser/     # Surface language parser
│   ├── ash-typeck/     # Type checker and effect analysis
│   ├── ash-interp/     # Interpreter and runtime
│   ├── ash-provenance/ # Audit trail and provenance
│   └── ash-cli/        # Command-line interface
├── examples/           # Example workflows
├── tests/              # Test suite
└── docs/               # Documentation
```

## Quick Start

```bash
# Build
cargo build --release

# Run a workflow
ash run examples/support_ticket.ash

# Type check
ash check examples/code_review.ash

# Run with provenance tracking
ash run --trace examples/multi_agent_research.ash
```

## Language Example

```ash
workflow support_ticket {
  observe search_kb with query: ticket.subject as docs;
  orient analyze(docs, ticket) as analysis;
  
  decide { analysis.confidence > 0.8 } under external_comm then {
    act send_email(to: ticket.customer, body: analysis.reply);
  } else {
    act escalate(to: senior_agent);
  }
}
```

## Status

🚧 Work in progress - implementing core semantics from Sharo Core Language specification.

## License

MIT OR Apache-2.0
