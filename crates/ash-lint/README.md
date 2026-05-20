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

### OODA Compatibility Rules

OODA lint rules are library/template compatibility guidance for historical
Observe/Orient/Decide/Act material. They point users toward the visible tower algebra
and explicit `Act`, `Proc`, and `Workflow` operations; they are not primitive
alpha execution semantics.

| Rule | Severity | Description |
|------|----------|-------------|
| `ooda-missing-decide` | Warning | Compatibility OODA template lacks an explicit decision marker |
| `ooda-missing-orient` | Warning | Compatibility OODA template reaches action-shaped work without an orientation marker |
| `ooda-out-of-order` | Error | Compatibility OODA markers appear in an unexpected order |

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
