# Ash Development Tools

This document lists all development tools for the Ash project, organized by installation method.

## No Sudo Required (Rust Crates)

These tools are installed via `cargo install` or included in the workspace.

### Included in Workspace

| Tool | Crate | Purpose |
|------|-------|---------|
| `cargo insta` | `insta` | Snapshot testing for parser/error output |
| `cargo criterion` | `criterion` | Benchmarking (ash-bench crate) |
| `proptest` | `proptest` | Property-based testing |

### Custom Ash Tools

| Tool | Path | Purpose |
|------|------|---------|
| `ash-lint` | `crates/ash-lint` | Custom lints for Ash workflows |
| `ash-cli` | `crates/ash-cli` | Main CLI (check, run, trace, repl) |

### Documentation Tools

```bash
# Install cargo-insta for snapshot testing
cargo install cargo-insta

# Install cargo-nextest for better test output (optional)
cargo install cargo-nextest
```

## System Tools (Requires Sudo)

These tools require system-level installation with administrator privileges.

### Required for Fuzzing

**cargo-fuzz** - Fuzz testing infrastructure
```bash
# Requires nightly Rust
rustup install nightly
rustup component add rust-src --toolchain nightly

# Install cargo-fuzz (requires compilation)
cargo install cargo-fuzz
```

### Required for Coverage

**cargo-tarpaulin** - Code coverage reporting
```bash
# On Debian/Ubuntu - requires sudo for ptrace permissions
sudo apt-get install libssl-dev pkg-config  # dependencies

# Install tarpaulin
cargo install cargo-tarpaulin

# To run coverage (may need sudo on some systems)
cargo tarpaulin --workspace

# Or with specific features
cargo tarpaulin --workspace --features smt
```

**Note**: On Linux, tarpaulin uses `ptrace` which may require:
```bash
# Option 1: Run with sudo
sudo $(which cargo) tarpaulin --workspace

# Option 2: Add ptrace capability (one-time)
sudo setcap cap_sys_ptrace+ep $(which cargo-tarpaulin)

# Option 3: Use --privileged in Docker
```

### Optional: Z3 SMT Solver

**libz3** - System library for SMT solving (required when `smt` feature enabled)
```bash
# Debian/Ubuntu
sudo apt-get install libz3-dev

# macOS
brew install z3

# Arch Linux
sudo pacman -S z3
```

## Tool Installation Summary

### Quick Setup (No Sudo)
```bash
# All tools that can be installed without sudo
cargo install cargo-insta cargo-nextest

# Install git hooks
./scripts/install-hooks.sh
```

### Full Setup (With Sudo)
```bash
# Run these commands with sudo access:

# 1. System dependencies
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config libz3-dev

# 2. Install tools
cargo install cargo-fuzz cargo-tarpaulin

# 3. Setup git hooks
./scripts/install-hooks.sh

# 4. Install nightly for fuzzing
rustup install nightly
rustup component add rust-src --toolchain nightly
```

## Tool Usage

### Snapshot Testing (insta)
```bash
# Run tests with snapshots
cargo insta test --accept

# Review pending snapshots
cargo insta review
```

### Benchmarking
```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench effect_lattice
```

### Fuzzing
```bash
# Run fuzz target (requires nightly)
cd crates/ash-fuzz
cargo +nightly fuzz run fuzz_effect_lattice
```

### Coverage
```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html

# Open report
open tarpaulin-report.html
```

### Linting
```bash
# Run Ash custom lints
cargo run --bin ash-lint -- workflow.ash

# With warnings as errors
cargo run --bin ash-lint -- --deny-warnings workflow.ash
```

## CI/CD Tools

These tools are used in CI but not required locally:

| Tool | CI Purpose |
|------|------------|
| `cargo-deny` | License checking |
| `cargo-audit` | Security advisories |
| `cargo-outdated` | Dependency updates |

Install locally if desired:
```bash
cargo install cargo-deny cargo-audit cargo-outdated
```

## Troubleshooting

### "cargo-fuzz not found"
```bash
rustup install nightly
cargo install cargo-fuzz
```

### "z3.h not found" (with smt feature)
```bash
# Ubuntu/Debian
sudo apt-get install libz3-dev

# Or disable SMT feature
cargo check --workspace
```

### Tarpaulin permission denied
```bash
# Option 1: Use Docker
# Option 2: Add capability
sudo setcap cap_sys_ptrace+ep $(which cargo-tarpaulin)
# Option 3: Skip coverage locally, rely on CI
```
