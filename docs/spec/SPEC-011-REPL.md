# SPEC-011: REPL (Read-Eval-Print Loop)

## Status: Draft

## 1. Overview

The Ash REPL provides an interactive environment for:
- Quick experimentation with Ash syntax
- Testing workflow fragments
- Learning the language
- Debugging with `:type` and `:ast` inspection

## 2. Interface

### 2.1 Starting the REPL

```bash
$ ash repl           -- Start interactive session
$ ash-repl           -- Standalone binary
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

### 4.1 Type Inspection

```
ash> :type 42
Number

ash> :type "hello"
String

ash> :type [1, 2, 3]
List<Number>
```

### 4.2 AST Inspection

```
ash> :ast 1 + 2
Binary {
    op: Add,
    left: Literal(Number(1.0)),
    right: Literal(Number(2.0)),
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
- Default location: `~/.local/share/ash-repl/history`

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

### 6.1 Syntax Errors

```
ash> 1 + 
Error: Unexpected end of input
  |
1 | 1 + 
  |     ^ expected expression
```

### 6.2 Type Errors

```
ash> :type if true then 1 else "hello"
Error: Type mismatch
  |
1 | if true then 1 else "hello"
  |     ^^^^     ^     ^^^^^^^
  |     Number   |     String
  |              expected same type in both branches
```

### 6.3 Runtime Errors

```
ash> file:read("nonexistent.txt")
Error: File not found: nonexistent.txt
```

## 7. Configuration

### 7.1 Command Line Options

```bash
$ ash-repl --help
ash-repl [OPTIONS]

Options:
  --history          Show history file location and exit
  --no-history       Don't load or save history
  --config <PATH>    Use custom config file
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

## 9. Security Considerations

### 9.1 Capability Restrictions

REPL runs with full capabilities by default. Future versions may support restricted modes:

```bash
$ ash-repl --sandbox  -- No file system access
$ ash-repl --no-net   -- No network access
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
