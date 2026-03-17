# CI/CD Plan for Ash

## Overview

This document outlines the planned CI/CD structure for the Ash project. The goal is to provide fast feedback on PRs while ensuring thorough testing before merge.

## Workflow Strategy

Split CI into multiple workflows for efficiency and clarity:

### 1. `ci-fast.yml` - Quick Feedback (3-5 min)
**Triggers**: Push to PR branches, push to main
**Purpose**: Fast feedback on common issues

**Jobs**:
- `format` - cargo fmt --check
- `clippy` - cargo clippy --workspace
- `check` - cargo check --workspace
- `unit-tests` - cargo test --lib --workspace

**Properties**:
- Runs on ubuntu-latest
- Uses sccache for speed
- No sudo required

### 2. `ci-full.yml` - Comprehensive Testing (10-15 min)
**Triggers**: Push to main, PR labeled "ready-for-review"
**Purpose**: Full test suite before merge

**Jobs**:
- `unit-tests` - Full test suite (--all-targets)
- `doc-tests` - cargo test --doc + ash-doc-tests
- `integration-tests` - End-to-end tests
- `lint` - ash-lint on example files
- `build` - Release build verification

**Properties**:
- Matrix: ubuntu-latest, macos-latest
- Uses sccache
- Produces build artifacts

### 3. `ci-fuzz.yml` - Fuzz Testing (30+ min, scheduled)
**Triggers**: Nightly cron, manual dispatch
**Purpose**: Find bugs through fuzzing

**Jobs**:
- `fuzz-effect-lattice` - Fuzz effect operations
- `fuzz-value-roundtrip` - Fuzz serialization
- `fuzz-parser` - Fuzz parser (when ready)

**Properties**:
- Runs on ubuntu-latest
- Uses nightly Rust
- 30 min per target
- Uploads crashes as artifacts

### 4. `ci-coverage.yml` - Code Coverage (15 min)
**Triggers**: Push to main, PR labeled "ready-for-review"
**Purpose**: Track code coverage

**Jobs**:
- `coverage` - cargo tarpaulin
- `report` - Upload to codecov or similar

**Properties**:
- Uses cargo-tarpaulin
- Generates HTML report
- Uploads to external service (optional)

### 5. `ci-bench.yml` - Benchmarks (10 min)
**Triggers**: Push to main, PRs touching performance-critical code
**Purpose**: Detect performance regressions

**Jobs**:
- `bench` - Run criterion benchmarks
- `compare` - Compare against baseline

**Properties**:
- Runs on dedicated runner (consistent hardware)
- Stores results for comparison
- Comments on PR with results

### 6. `ci-security.yml` - Security Audit (5 min)
**Triggers**: Weekly cron, manual dispatch
**Purpose**: Check for security vulnerabilities

**Jobs**:
- `audit` - cargo audit
- `deny` - cargo deny (licenses)

**Properties**:
- Runs cargo-audit
- Checks for known vulnerabilities

## Implementation Phases

### Phase 1: Basic CI (Immediate)
- [ ] ci-fast.yml - Essential checks
- [ ] ci-full.yml - Full test suite

### Phase 2: Extended Testing (Week 2)
- [ ] ci-coverage.yml - Coverage reporting
- [ ] ci-security.yml - Security audit

### Phase 3: Advanced (Week 4+)
- [ ] ci-fuzz.yml - Fuzz testing
- [ ] ci-bench.yml - Performance tracking

## Configuration Details

### Caching Strategy
1. **sccache**: Shared compiler cache
   - Key: `${{ runner.os }}-sccache-${{ hashFiles('**/Cargo.lock') }}`
   - Shared across workflows

2. **cargo registry**: Crate download cache
   - Key: `${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}`

3. **cargo target**: Build cache (per workflow)
   - Key: `${{ runner.os }}-cargo-target-${{ hashFiles('**/Cargo.lock') }}`

### Rust Version Strategy
- **stable**: Default for most workflows
- **nightly**: Required for fuzzing
- **MSRV**: Test on MSRV periodically (ci-full.yml)

### Secrets Required
- `CODECOV_TOKEN` - For coverage upload (optional)
- `CARGO_REGISTRY_TOKEN` - For releases (future)

## PR Requirements

### Required Checks (must pass before merge)
- [ ] ci-fast / format
- [ ] ci-fast / clippy
- [ ] ci-fast / check
- [ ] ci-fast / unit-tests
- [ ] ci-full / unit-tests
- [ ] ci-full / doc-tests

### Optional Checks (informative)
- [ ] ci-coverage / coverage
- [ ] ci-security / audit
- [ ] ci-bench / bench (on performance PRs)

## Rollout Plan

1. **This PR**: Add ci-fast.yml only
   - Minimal, provides immediate value
   - Test configuration

2. **Follow-up PR**: Add ci-full.yml
   - More comprehensive testing
   - Matrix builds

3. **Later PRs**: Add specialized workflows
   - Fuzzing, benchmarks, security
   - Each tested individually

## Testing the CI

To test CI changes:

1. Push to a feature branch
2. Create a draft PR
3. Verify workflows trigger correctly
4. Check timing and caching
5. Iterate on configuration
6. Mark PR ready for review

## Future Enhancements

- [ ] Auto-merge on green (with proper checks)
- [ ] PR size labeling
- [ ] Changelog verification
- [ ] Release automation
- [ ] Documentation deployment
