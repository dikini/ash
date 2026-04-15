# SPEC-042: Ash Source Formatter

## Status: Draft

## 1. Goal

Provide a source formatter for Ash that pretty-prints any valid `ModuleFile` while preserving user comments and blank lines.

## 2. Scope

MVP formatter supports:
- All current Ash surface syntax
- Comment preservation (via SPEC-039 comment-trivia side-table)
- Configurable indent width (default 4 spaces)
- Consistent spacing around operators and punctuation

Out of scope for MVP:
- Vertical alignment of record fields
- Reflowing of long lines (soft wrap)
- Opinionated expression parenthesization beyond precedence
- Intra-expression comments (e.g., `foo(--c\n)(args)`)

## 3. Crate Boundary

The formatter lives in a **new crate**:

```
crates/ash-formatter/
  src/lib.rs   # formatting engine
  src/main.rs  # thin CLI wrapper for `ash fmt`
```

## 4. Dependency on SPEC-039

The formatter **requires** the `CommentTable` introduced in SPEC-039. It cannot be implemented before comment trivia is available. Specifically, SPEC-039 must deliver:

1. `Comment` capture during whitespace skipping (no token-stream changes).
2. `CommentTable` side-table implementation.
3. `comments: CommentTable` field on `ModuleFile`.
4. `parse_surface_file(source: &str) -> Result<ModuleFile, Vec<ParseError>>` entry point.
5. Binding spans (`Expr::Variable { name, span }`, `Pattern::Variable { name, span }`).

> **Explicit precondition:** `Expr::Variable` must carry a `span` field so that comments can be stably attached to variable references. The same requirement applies to `Pattern::Variable`.

## 5. Core Design

### 5.1 Formatter Configuration

```rust
pub struct FormatConfig {
    pub indent_width: usize,
    pub max_width: Option<usize>,
}
```

### 5.2 Formatter State

```rust
pub struct Formatter<'a> {
    output: String,
    indent_level: usize,
    config: &'a FormatConfig,
    comments: &'a CommentTable,
}

impl<'a> Formatter<'a> {
    pub fn new(comments: &'a CommentTable, config: &'a FormatConfig) -> Self { ... }
}
```

### 5.3 Entry Point

`format_module` is a **free function** taking `&ModuleFile` and `&FormatConfig`:

```rust
pub fn format_module(module: &ModuleFile, config: &FormatConfig) -> String {
    let mut fmt = Formatter::new(&module.comments, config);
    fmt.write_leading_comments(module.span);
    for decl in &module.module_decls {
        fmt.write_module_decl(decl);
        fmt.write_newline();
    }
    for def in &module.definitions {
        fmt.write_definition(def);
        fmt.write_newline();
    }
    if let Some(workflow) = &module.workflow {
        fmt.write_workflow(workflow);
        fmt.write_newline();
    }
    fmt.output
}
```

### 5.4 Formatting Algorithm

The formatter uses a **direct recursive walk** over the AST with a small formatting IR (not a full pretty-printing library). The IR consists of tokens, explicit line breaks, and indent/dedent commands. The walk generates the IR, which is then rendered to a `String`. This avoids the complexity and dependency cost of an external pretty-printing crate while still giving explicit control over comment placement and line breaks.

### 5.5 Blank-Line Preservation Rules

Blank lines are normalized numerically rather than preserved exactly:

- **Between top-level module declarations and definitions:** at most 1 blank line (collapse consecutive blank lines to 1).
- **Inside workflow bodies, block expressions, and other nested statement lists:** collapse to 0 blank lines.
- **Between the last top-level item and a workflow definition:** at most 1 blank line.
- **Start/end of file:** no leading or trailing blank lines.
- **Around comments:** if a comment is attached as a leading comment, any blank lines between the comment and its anchor are collapsed to 0.

### 5.6 Comment Insertion Rules

Before emitting any AST node with span `s`:
1. Write all `comments.leading_comments(s)`, each on its own line.
2. Emit the node.
3. Write all `comments.trailing_comments(s)` on the same line (if any).

Blank line normalization (see 5.5) is applied after comment insertion.

> **Intra-expression comments** (comments inside expressions) are **out of scope** for MVP. They are explicitly deferred to a future formatter spec.

### 5.7 Expression Formatting

Expressions are formatted with standard precedence-aware parenthesization:
- Binary operators get spaces: `a + b`
- Function calls: `foo(a, b, c)`
- Pipelines/record literals use multi-line if any field spans multiple lines
- Blocks indent their body by one level

### 5.8 Workflow Formatting

All `surface::Workflow` variants are formatted with a keyword on the same line and any nested body indented by one additional level:

| Variant | Keyword(s) | Indentation Rule |
|---|---|---|
| `Observe { capability, binding, continuation, span }` | `observe` | `observe capability [as binding] [-> continuation]`; continuation body indented +1 |
| `Orient { expr, binding, continuation, span }` | `orient` | `orient expr [as binding] [-> continuation]` |
| `Propose { action, binding, continuation, span }` | `propose` | `propose action [as binding] [-> continuation]` |
| `Decide { expr, policy, then_branch, else_branch, span }` | `decide` | `decide expr [policy name] then branch [else branch]`; branches indented +1 |
| `Check { target, continuation, span }` | `check` | `check target [-> continuation]` |
| `Oblige { obligation, span }` | `oblige` | `oblige obligation` |
| `Act { action, guard, result_name, continuation, span }` | `act` | `act action [guard] [as result] [-> continuation]` |
| `Let { pattern, expr, continuation, span }` | `let` | `let pattern = expr [-> continuation]` |
| `If { condition, then_branch, else_branch, span }` | `if` / `then` / `else` | `if condition then branch [else branch]`; branches indented +1 |
| `For { pattern, collection, body, span }` | `for` | `for pattern in collection do body`; body indented +1 |
| `With { capability, body, span }` | `with` | `with capability do body`; body indented +1 |
| `Maybe { primary, fallback, span }` | `maybe` / `otherwise` | `maybe primary otherwise fallback`; each branch indented +1 if multi-line |
| `Must { body, span }` | `must` | `must body`; body indented +1 |
| `Seq { first, second, span }` | — | `first ; second` (or newline if either side is multi-line) |
| `Done { span }` | `done` | `done` |
| `Ret { expr, span }` | `return` | `return expr` |
| `Set { capability, channel, value, continuation, span }` | `set` | `set capability:channel = value [-> continuation]` |
| `Send { capability, channel, value, continuation, span }` | `send` | `send capability:channel = value [-> continuation]` |
| `Receive { mode, arms, is_control, span }` | `receive` | `receive { arms }`; each arm indented +1 |
| `Yield { role, expr, resume_var, resume_type, arms, span }` | `yield` | `yield expr to role resume resume_var: resume_type { arms }`; arms indented +1 |
| `Resume { expr, ty, span }` | `resume` | `resume expr` |

> **Note:** The surface `Workflow` enum does not currently contain a `Spawn` variant. If one is added in the future, it shall follow the same pattern: keyword on the same line, any nested body indented +1.

### 5.9 PolicyExpr and ConstraintBlock Formatting

`PolicyExpr` variants are formatted using the operators defined by the surface grammar:

| Variant | Formatting Rule |
|---|---|
| `Var(name)` | `name` |
| `And(exprs)` | `expr1 & expr2 & ...` (spaces around `&`) |
| `Or(exprs)` | `expr1 \| expr2 \| ...` (spaces around `\|`) |
| `Not(expr)` | `!expr` (no space after `!`) |
| `Implies(a, b)` | `implies(a, b)` (function-style call) |
| `Sequential(exprs)` | `expr1 >> expr2 >> ...` (spaces around `>>`) |
| `Concurrent(exprs)` | `concurrent(expr1, expr2, ...)` (function-style call) |
| `ForAll { var, items, body, .. }` | `forall(var, items, body)` |
| `Exists { var, items, body, .. }` | `exists(var, items, body)` |
| `MethodCall { receiver, method, args, .. }` | `receiver.method(args)` |
| `Call { func, args, .. }` | `func(args)` |

`ConstraintBlock` is formatted as `@ { field1: value1, field2: value2 }`. If it does not fit on one line (or contains nested objects/arrays), each field is placed on its own line indented by one additional level:

```
@ {
    field1: value1,
    field2: value2,
}
```

`ConstraintField` is formatted as `name: value`. `ConstraintValue` variants (`Bool`, `Int`, `String`, `Array`, `Object`) are formatted like their corresponding expression literals.

### 5.10 Semicolon Emission Rules

Ash statements are separated by newlines. The formatter **never emits semicolons** in the MVP because the surface syntax does not use them. If a future syntax introduces optional semicolons, the formatter will preserve them; until then, statements are terminated by `\n` only.

## 6. LSP Integration

Once the formatter exists, `textDocument/formatting` and `textDocument/rangeFormatting` become trivial handlers in `ash-lsp`:

```rust
async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>, Error> {
    let module = self.vfs.parse(uri)?;
    let formatted = ash_formatter::format_module(&module, &FormatConfig { indent_width: 4, max_width: None });
    Ok(Some(vec![TextEdit {
        range: full_document_range,
        new_text: formatted,
    }]))
}
```

`full_document_range` is defined as the range covering the entire document from `Position { line: 0, character: 0 }` to the end of the last line of the source text.

## 7. CLI Integration

Reconcile with **SPEC-005**:

```bash
ash fmt [options] <file.ash>
```

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without modifying |
| `--write` | Format files in place (default when neither `--check` nor `--stdin` is given) |
| `--stdin` | Read from stdin, write to stdout |
| `--indent <n>` | Indent width in spaces; valid range is `1..=16` (default 4) |

By default, `ash fmt` writes the formatted file in place. `--write` is the implicit default when neither `--check` nor `--stdin` is provided. `--stdin` forces output to stdout.

## 8. Error Handling

If the input cannot be parsed:
1. **CLI:** Print parse errors to stderr and exit with code 2. Do not write a formatted file.
2. **LSP:** Return an empty `TextEdit` list and let the server's diagnostic pipeline surface the parse errors.
3. **Library API (`format_module`):** Requires a pre-parsed `ModuleFile`; parsing is the caller's responsibility.

## 9. Testing Strategy

1. **Round-trip parsing:** For every example file, `parse_surface_file(format(parse_surface_file(src))) == parse_surface_file(src)` (AST equality **ignoring spans**, since formatting changes spans).
2. **Comment preservation:** Format a file with comments; verify all comment text appears in the output.
3. **Idempotency:** `format(format(source)) == format(source)`. This property depends on **comment re-attachment stability**. The formatter must use a **two-pass or span-independent algorithm** so that reformatting already-formatted code does not shift spans in a way that changes comment placement. Comments must be re-attached using a stable key (e.g., node identity or original span) that is invariant across passes.
4. **Proptest:** Generate random valid ASTs (via `ash-core` proptest helpers), format them, and assert round-trip parse equality modulo spans.

## 10. Relationship to Other Specs

- **Blocked by:** SPEC-039 (comment trivia and binding spans)
- **Blocks:** None directly; enables LSP formatting
- **Follows:** SPEC-038 LSP MVP
