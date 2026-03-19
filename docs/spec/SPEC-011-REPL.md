# SPEC-011: REPL (Read-Eval-Print Loop)

## Status: Draft

## 1. Overview

The Ash REPL provides an interactive environment for:
- Quick experimentation with Ash syntax
- Testing workflow fragments
- Learning the language
- Debugging with `:type` and `:ast` inspection

This document specifies the session behavior of the REPL started through `ash repl`.
There is one normative REPL authority: the CLI entrypoint defined in `SPEC-005`.
Any standalone `ash-repl` binary is non-normative compatibility surface, not a second contract.
The observable REPL command output, display shape, and error visibility are owned by
[SPEC-021: Runtime Observable Behavior](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md).

## 2. Interface

### 2.1 Starting the REPL

```bash
$ ash repl                    -- Start interactive session
$ ash repl --history /tmp/h   -- Override history location
$ ash repl --no-history       -- Disable persistent history
$ ash repl --config repl.toml -- Override REPL config
```

### 2.2 Prompt

```
ash>               -- Normal prompt
...                -- Continuation prompt (multiline input)
```

## 3. Input Handling

### 3.1 Expression Evaluation

Simple expressions are evaluated immediately:

```
ash> 1 + 2
3

ash> "hello"
"hello"

ash> [1, 2, 3]
[1, 2, 3]
```

### 3.2 Workflow Definitions

Workflows can be defined and executed:

```
ash> workflow test { action a { effect: operational; body: || -> 42; } }
ash> test
42
```

### 3.3 Multiline Input

Incomplete expressions continue to next line:

```
ash> workflow test {
...     action a {
...         effect: operational;
...         body: || -> 42;
...     }
... }
ash> test
42
```

## 4. REPL Commands

Commands start with `:`:

| Command | Alias | Description |
|---------|-------|-------------|
| `:help` | `:h` | Show help message |
| `:quit` | `:q` | Exit the REPL |
| `:type` | `:t` | Show type of expression |
| `:ast` | | Show AST representation |
| `:clear` | | Clear screen |

No other REPL commands are normative in this specification. Interactive effect inspection,
DOT generation, workflow loading, and trace toggling are outside the REPL contract unless
they are added here and in `SPEC-005`.

### 4.1 Type Inspection

```
ash> :type 42
Int

ash> :type "hello"
String

ash> :type [1, 2, 3]
List<Int>
```

### 4.2 AST Inspection

```
ash> :ast 1 + 2
Binary {
    op: Add,
    left: Literal(Int(1)),
    right: Literal(Int(2)),
}
```

## 5. Readline Features

### 5.1 Line Editing

Standard readline editing:
- Arrow keys for navigation
- Home/End for line start/end
- Ctrl+A/Ctrl+E for line start/end
- Ctrl+K to kill to end of line
- Ctrl+U to kill whole line

### 5.2 History

- Up/Down arrows navigate history
- History persists between sessions
- Default location: `~/.local/share/ash/repl/history`
- `ash repl --history <file>` overrides the path for one session
- `ash repl --no-history` disables both loading and saving history for one session

### 5.3 Tab Completion

Tab completion for:
- Keywords (`workflow`, `action`, `capability`, etc.)
- Built-in functions
- Previously defined names

```
ash> cap<TAB>
ash> capability

ash> work<TAB>
ash> workflow
```

## 6. Error Display

Observable REPL error rendering, including parse, type, verification, policy, and runtime
failures, is owned by [SPEC-021: Runtime Observable Behavior](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md).
This section only records that the REPL must surface those failures through the canonical observable
categories instead of inventing a separate REPL-specific error model.

## 7. Configuration

### 7.1 Command Line Options

```bash
$ ash repl --help
ash repl [OPTIONS]

Options:
  --history <PATH>   Override history file location
  --no-history       Disable history load/save
  --config <PATH>    Use custom config file
  --init <PATH>      Run startup commands before the first prompt
  --capability <name=uri>
                     Provide default capability bindings for the session
  -h, --help         Print help
```

### 7.2 Configuration File

Optional config at `~/.config/ash/repl.toml`:

```toml
[repl]
history_limit = 1000
prompt = "ash> "
colors = true

[completion]
enabled = true
max_suggestions = 20
```

## 8. Implementation

### 8.1 Architecture

```
┌─────────────────┐
│   REPL Loop     │
│  (rustyline)    │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌───────┐ ┌─────────┐
│Command│ │Evaluate │
│Handler│ │ (Engine)│
└───────┘ └─────────┘
```

### 8.2 Multiline Detection

Multiline input is detected by parse errors:
- `Incomplete` → continue reading
- `Error` → report error immediately

## 8.3 Observable Output Contract

The canonical REPL observable behavior contract lives in
[SPEC-021: Runtime Observable Behavior](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md).

## 9. Security Considerations

### 9.1 Capability Restrictions

REPL runs with full capabilities by default. Future versions may support restricted modes:

```bash
$ ash repl --sandbox  -- No file system access
$ ash repl --no-net   -- No network access
```

### 9.2 History Privacy

- History file contains all inputs
- May contain sensitive data
- Stored with user permissions only

## 10. Future Extensions

- Syntax highlighting
- Auto-formatting on enter
- Integration with language server
- Save/load session state
- Built-in tutorials (`:tutorial`)
