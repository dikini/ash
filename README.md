# Ash

**A reference implementation of the Sharo Core Language (SHC)**

Ash is an executable semantics and runtime for the Sharo Core function-and-effect-row language, designed for governed AI systems with formal verification capabilities.

## Overview

Ash provides:

- **Parser**: Surface language → IR
- **Type Checker**: Effect tracking and obligation verification
- **Engine**: Local checked Core/CPS execution for each client route
- **Provenance Tracker**: Complete audit trails
- **Policy Engine**: Deontic logic evaluation

## Project Structure

```
ash/
├── crates/
│   ├── ash-core/       # IR and semantics definitions
│   ├── ash-parser/     # Surface language parser
│   ├── ash-typeck/     # Type checker and effect analysis
│   ├── ash-runtime/    # Runtime support for Engine execution
│   ├── ash-provenance/ # Audit trail and provenance
│   └── ash-cli/        # Command-line interface
├── examples/           # Target Ash examples
├── tests/              # Test suite
└── docs/               # Documentation
```

## Quick Start

`ash run`, `ash test`, and the REPL each execute through their own local Engine instance. They
do not connect to the daemon. The daemon accepts submitted descriptors, executes them through its
own local Engine instance, and manages long-running programs. These routes share implementation
and contracts; there is no Engine service.

```bash
# Build
cargo build --release

# Check a target Ash example
ash check examples/10-testing-helpers/testing_helpers.ash

# Check process/channel helper examples
ash check examples/11-process-channel-helpers/process_channel_helpers.ash

# Run the example corpus gate
cargo test -p ash-cli --test example_corpus_check -- --nocapture
```

Current examples are listed in [examples/README.md](examples/README.md). Phase 201 removed older
workflow-era examples from productive repository paths; new examples must use target Ash only.

Target Ash entries use ordinary `fn main` definitions:

```ash
fn main() -> Bool {
  do {
    return true;
  }
}
```

## Language Example

```ash
fn support_ticket_ready(confidence: Int) -> Bool {
  confidence > 80
}

fn main() -> Bool {
  do {
    let ready = support_ticket_ready(95);
    return ready;
  }
}
```

## Status

🚧 Work in progress - implementing core semantics from Sharo Core Language specification.

## License

MIT OR Apache-2.0
