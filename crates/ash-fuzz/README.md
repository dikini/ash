# ash-fuzz - Fuzz Testing for Ash

This crate contains fuzz tests for the Ash workflow language. It uses `cargo-fuzz` with libfuzzer.

## Prerequisites

Rust nightly toolchain is required for fuzzing:

```bash
rustup install nightly
rustup component add rust-src --toolchain nightly
cargo install cargo-fuzz
```

## Running Fuzz Tests

### List available targets
```bash
cd crates/ash-fuzz
cargo fuzz list
```

### Run a specific target
```bash
cd crates/ash-fuzz
cargo fuzz run fuzz_effect_lattice
```

### Run with time limit
```bash
cargo fuzz run fuzz_effect_lattice -- -max_total_time=60
```

### Run smoke test (quick check)
```bash
cargo fuzz run fuzz_effect_lattice -- -max_total_time=1
```

## Available Targets

- `fuzz_effect_lattice`: Tests effect lattice operations (join, meet) for algebraic properties
- `fuzz_value_roundtrip`: Tests Value serialization/deserialization roundtrip

## Adding New Targets

1. Create a new file in `fuzz_targets/`
2. Add the binary to `Cargo.toml` `[[bin]]` section
3. Implement the `fuzz_target!` macro

Example:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzz test here
});
```

## CI Integration

The fuzz tests are run in CI via the `scripts/check-fuzz.sh` script:

- **Pre-commit**: Smoke test (`-max_total_time=1`)
- **Pre-push**: Full 60-second fuzz run

## Corpus

Fuzz corpus is stored in `crates/ash-fuzz/corpus/`. Interesting inputs found during fuzzing are automatically saved here.
