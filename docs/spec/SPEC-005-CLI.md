# SPEC-005: Ash CLI Specification

## Overview

This document specifies the command-line interface for the Ash workflow language. The CLI is the primary user-facing tool for developing, checking, and executing Ash workflows.

## Design Principles

1. **Familiarity**: Follow conventions from `cargo`, `rustfmt`, `clippy`
2. **Discoverability**: Good help text, shell completions
3. **Composability**: Unix-friendly (pipeable, machine-readable output)
4. **Progressive Disclosure**: Simple by default, powerful with flags

## Command Structure

```
ash <command> [options] <file-or-path>
```

### Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message |
| `--version` | `-V` | Show version |
| `--verbose` | `-v` | Increase verbosity (repeatable) |
| `--quiet` | `-q` | Suppress output |
| `--color` | | Color output: auto, always, never |

## Commands

### `ash check` - Type Check and Lint

Validate an Ash workflow without executing it.

```bash
ash check [options] <file.ash>
```

**Options:**

| Option | Description |
|--------|-------------|
| `--all` | Check all workflows in directory |
| `--strict` | Treat warnings as errors |
| `--format <fmt>` | Output format: human, json, short |
| `--policy-check` | Enable policy conflict detection |

**Output Formats:**

Human (default):
```
error: Type mismatch in ORIENT
  --> workflow.ash:12:5
   |
12 |     orient { x + 1 }
   |     ^^^^^^^^^^^^^^^ expected String, found Int
   |
   = help: ORIENT expects an expression returning String
```

JSON:
```json
{
  "diagnostics": [
    {
      "severity": "error",
      "message": "Type mismatch in ORIENT",
      "file": "workflow.ash",
      "line": 12,
      "column": 5,
      "help": "ORIENT expects an expression returning String"
    }
  ],
  "error_count": 1,
  "warning_count": 0
}
```

**Exit Codes:**
- `0`: No errors (warnings ok unless `--strict`)
- `1`: Type errors or policy violations
- `2`: Parse errors
- `3`: I/O errors

### `ash run` - Execute Workflow

Execute an Ash workflow with the specified capability providers.

```bash
ash run [options] <file.ash> [-- <input>]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--input <file>` | JSON input file |
| `--output <file>` | Write output to file |
| `--format <fmt>` | Output format: json, yaml, toml, raw |
| `--dry-run` | Validate only, don't execute |
| `--timeout <secs>` | Maximum execution time |
| `--trace` | Enable provenance tracing |
| `--capability <name=uri>` | Bind capability to provider |

**Examples:**

```bash
# Run with JSON input
ash run workflow.ash --input data.json

# Run with inline input
ash run workflow.ash -- '{"name": "test"}'

# Run with custom capability binding
ash run workflow.ash --capability db=postgres://localhost/mydb

# Dry run (validate without executing)
ash run --dry-run workflow.ash
```

### `ash trace` - Execute with Full Provenance

Execute a workflow with complete audit trail capture.

```bash
ash trace [options] <file.ash>
```

**Options:**

| Option | Description |
|--------|-------------|
| `--output <file>` | Trace output file (default: trace.json) |
| `--format <fmt>` | Trace format: json, provn, cypher |
| `--sign` | Cryptographically sign trace |
| `--export <fmt>` | Export to: w3c-prov, dublin-core |

**Trace Output:**

```json
{
  "trace_id": "uuid",
  "workflow": "workflow.ash",
  "started_at": "2024-03-17T10:30:00Z",
  "events": [
    {
      "timestamp": "2024-03-17T10:30:00.100Z",
      "event": "observe",
      "capability": "sensor",
      "value": { "temperature": 22.5 },
      "provenance": { ... }
    },
    {
      "timestamp": "2024-03-17T10:30:00.200Z",
      "event": "decide",
      "guard": "temperature > 20",
      "result": true
    },
    {
      "timestamp": "2024-03-17T10:30:00.300Z",
      "event": "act",
      "action": "alert",
      "provenance": { ... }
    }
  ],
  "final_state": { ... }
}
```

### `ash repl` - Interactive Development

Start an interactive REPL for Ash.

```bash
ash repl [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--history <file>` | History file path |
| `--init <file>` | Startup script |
| `--capability <name=uri>` | Default capability bindings |

**REPL Commands:**

| Command | Description |
|---------|-------------|
| `:help` | Show help |
| `:quit` | Exit REPL |
| `:type <expr>` | Show type of expression |
| `:effect <expr>` | Show effect level |
| `:parse <code>` | Show AST |
| `:dot <code>` | Generate DOT visualization |
| `:load <file>` | Load workflow file |
| `:trace` | Toggle tracing |

**Example Session:**

```
$ ash repl
Ash 0.1.0 REPL
Type :help for help, :quit to exit

ash> observe capability "sensor" as data
Effect: Epistemic
Type: Value

ash> :type data
Value

ash> :effect data
Epistemic

ash> :dot decide { data.temperature > 20 }
digraph Workflow {
  node_0 [label="DECIDE", shape=diamond, fillcolor="lightsalmon"];
  node_1 [label="expr", shape=ellipse, fillcolor="white"];
  node_0 -> node_1 [label="guard"];
}

ash> :quit
```

### `ash dot` - Generate AST Visualization

Generate Graphviz DOT output for workflow visualization.

```bash
ash dot [options] <file.ash>
```

**Options:**

| Option | Description |
|--------|-------------|
| `--output <file>` | Output file (default: stdout) |
| `--format <fmt>` | Output: dot, svg, png (requires graphviz) |
| `--effect-colors` | Color nodes by effect level (default: true) |
| `--simplify` | Simplify nested structures |

**Examples:**

```bash
# Generate DOT output
ash dot workflow.ash > workflow.dot

# Generate SVG (requires graphviz)
ash dot --format svg workflow.ash > workflow.svg

# View in browser
ash dot workflow.ash | dot -Tsvg > /tmp/wf.svg && firefox /tmp/wf.svg
```

### `ash fmt` - Format Workflow Files

Format Ash workflow source files.

```bash
ash fmt [options] <file-or-path>
```

**Options:**

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without modifying |
| `--write` | Format files in place (default) |
| `--stdin` | Read from stdin, write to stdout |

**Exit Codes:**
- `0`: Files are formatted (or `--check` and formatted)
- `1`: Formatting needed (with `--check`)

### `ash lsp` - Language Server

Start LSP server for editor integration.

```bash
ash lsp [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--stdio` | Use stdio for communication (default) |
| `--port <n>` | Use TCP port |

**LSP Features:**
- Syntax highlighting
- Error squiggles
- Hover type information
- Go to definition
- Completion
- Formatting
- Code actions (quick fixes)

## Configuration

### Configuration File

Location: `.ash.toml` or `ash.toml` in project root

```toml
[cli]
default_format = "json"
color = "auto"

[check]
strict = false
policy_check = true

[run]
timeout = 30
trace = false

[capabilities]
db = "postgres://localhost/mydb"
sensor = "mqtt://broker.local/sensors/#"

[provenance]
enabled = true
sign_keys = ["~/.ash/signing.key"]
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ASH_LOG` | Log level (error, warn, info, debug, trace) |
| `ASH_CONFIG` | Path to config file |
| `ASH_CACHE_DIR` | Cache directory |
| `ASH_NO_COLOR` | Disable color output |

## Error Handling

All commands use consistent error formatting:

```
error: <brief description>
  --> <file>:<line>:<col>
   |
<line> | <code>
   | <pointer>
   |
   = <category>: <detailed explanation>
   = help: <suggestion>
   = note: <additional context>
```

## Shell Completions

Generate shell completion scripts:

```bash
ash completions bash > /usr/share/bash-completion/completions/ash
ash completions zsh > /usr/share/zsh/site-functions/_ash
ash completions fish > ~/.config/fish/completions/ash.fish
```

## Future Commands

| Command | Description | Priority |
|---------|-------------|----------|
| `ash test` | Run workflow tests | High |
| `ash doc` | Generate documentation | Medium |
| `ash package` | Package workflow for distribution | Low |
| `ash publish` | Publish to registry | Low |
| `ash serve` | HTTP API server | Medium |

## Version Compatibility

- CLI version follows workspace version
- Breaking changes bump major version
- Config file has version field for migration
- Deprecated commands show warnings

## Implementation Notes

- Use `clap` for argument parsing
- Use `tracing` for logging
- Use `color-eyre` for error reporting
- Use `completest` for shell completions
- Use `tower-lsp` for LSP implementation
