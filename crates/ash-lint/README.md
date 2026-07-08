# ash-lint - Custom Lints for Ash

Custom lint rules for Ash source files, detecting issues beyond standard clippy checks.

## Installation

```bash
cargo install --path crates/ash-lint
```

## Usage

### Lint a file
```bash
ash-lint main.ash
```

### Lint a directory
```bash
ash-lint src/
```

### Treat warnings as errors
```bash
ash-lint --deny-warnings main.ash
```

### JSON output (for CI)
```bash
ash-lint --format json main.ash
```

### GitHub Actions format
```bash
ash-lint --format github main.ash
```

## Lint Rules

No target-Ash lint rules are currently active. Removed workflow-era rules are not retained in the
active lint surface.

## Configuration

Create `.ash-lint.toml` in project root:

```toml
[lints]
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
