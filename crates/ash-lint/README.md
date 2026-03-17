# ash-lint - Custom Lints for Ash

Custom lint rules for the Ash workflow language, detecting issues beyond standard clippy checks.

## Installation

```bash
cargo install --path crates/ash-lint
```

## Usage

### Lint a file
```bash
ash-lint workflow.ash
```

### Lint a directory
```bash
ash-lint src/workflows/
```

### Treat warnings as errors
```bash
ash-lint --deny-warnings workflow.ash
```

### JSON output (for CI)
```bash
ash-lint --format json workflow.ash
```

### GitHub Actions format
```bash
ash-lint --format github workflow.ash
```

## Lint Rules

### OODA Pattern Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `ooda-missing-decide` | Warning | OBSERVE without explicit DECIDE step |
| `ooda-missing-orient` | Warning | OBSERVE result never used in ORIENT |
| `ooda-out-of-order` | Error | OODA steps in wrong order |

### Effect System Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `effect-operational-without-decide` | Error | Operational effect without DECIDE approval |
| `effect-missing-provenance` | Warning | Operational effect without provenance tracking |

### Policy Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `policy-conflict-potential` | Warning | Potential policy conflict detected |
| `policy-unreachable` | Info | Policy guard is always false |

### Code Quality Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `unused-capability` | Warning | Capability bound but never used |
| `empty-workflow` | Warning | Workflow with no operations |
| `dead-code` | Info | Unreachable code detected |

## Configuration

Create `.ash-lint.toml` in project root:

```toml
[lints]
ooda-missing-decide = "warn"
effect-operational-without-decide = "error"
policy-conflict-potential = "allow"
```

## CI Integration

### GitHub Actions
```yaml
- name: Run ash-lint
  run: |
    cargo install --path crates/ash-lint
    ash-lint --format github --deny-warnings src/
```

### Pre-commit Hook
Add to `.pre-commit-config.yaml`:
```yaml
- repo: local
  hooks:
  - id: ash-lint
    name: Ash Linter
    entry: ash-lint
    language: system
    files: '\.ash$'
```
