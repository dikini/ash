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

### Build Acceleration

**sccache** - Compiler cache for faster builds
```bash
# Install sccache
cargo install sccache

# Configure Cargo to use sccache
cat >> ~/.cargo/config.toml << 'EOF'
[build]
rustc-wrapper = "sccache"
EOF

# Optional: Set cache location
export SCCACHE_DIR=~/.cache/sccache
export SCCACHE_CACHE_SIZE=10G
```

**Benefits:**
- Caches compiled crates across different project directories
- Shared cache between `cargo build`, `cargo test`, `cargo check`
- Significant speedup on CI and local development

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

### Coverage (No Sudo Required)

**cargo-tarpaulin** - Code coverage reporting
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage (no sudo needed for same-user processes)
cargo tarpaulin --workspace

# With specific features
cargo tarpaulin --workspace --features smt

# With HTML output
cargo tarpaulin --workspace --out Html
```

**Note**: Tarpaulin uses ptrace on test processes it spawns. This works without sudo in normal environments. If you encounter permission issues (e.g., in restricted containers), use the LLVM engine:

```bash
# Use LLVM engine instead of ptrace
cargo tarpaulin --workspace --engine llvm
```

Or set ptrace_scope (if system allows):
```bash
# Check current setting
cat /proc/sys/kernel/yama/ptrace_scope

# Temporarily allow (resets on reboot)
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
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

### Quick Setup (No Sudo Required)
```bash
# All Rust tools install without sudo
cargo install cargo-insta cargo-nextest cargo-tarpaulin sccache

# Configure sccache (optional but recommended)
echo '[build]\nrustc-wrapper = "sccache"' >> ~/.cargo/config.toml

# Install git hooks
./scripts/install-hooks.sh
```

### Full Setup (Minimal Sudo)
```bash
# These commands need sudo (system packages):
sudo apt-get update
sudo apt-get install -y libz3-dev  # Only if using smt feature

# These don't need sudo:
cargo install cargo-fuzz cargo-tarpaulin cargo-insta cargo-nextest sccache
rustup install nightly
rustup component add rust-src --toolchain nightly

# Configure sccache (optional but recommended)
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[build]
rustc-wrapper = "sccache"
EOF

./scripts/install-hooks.sh
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

### Build Cache (sccache)
```bash
# View cache statistics
sccache --show-stats

# Zero stats
sccache --zero-stats

# Stop daemon
sccache --stop-server
```

**Cache locations:**
- Linux: `~/.cache/sccache`
- macOS: `~/Library/Caches/sccache`
- Windows: `%LOCALAPPDATA%\sccache`

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
# Option 1: Use LLVM engine (no ptrace needed)
cargo tarpaulin --workspace --engine llvm

# Option 2: Temporarily relax ptrace_scope
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

# Option 3: Use Docker with --privileged
```

### sccache not working
```bash
# Check if sccache is properly configured
cat ~/.cargo/config.toml
# Should contain:
# [build]
# rustc-wrapper = "sccache"

# Verify sccache is in PATH
which sccache

# Check sccache stats
sccache --show-stats

# Restart sccache daemon
sccache --stop-server
# Next cargo command will restart it automatically
```
