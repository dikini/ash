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

## 5. Core Design

### 5.1 Formatter State

```rust
pub struct Formatter<'a> {
    output: String,
    indent_level: usize,
    indent_width: usize,
    comments: &'a CommentTable,
}

impl<'a> Formatter<'a> {
    pub fn new(comments: &'a CommentTable, indent_width: usize) -> Self { ... }
    pub fn format_module(module: &ModuleFile) -> String { ... }
}
```

### 5.2 Entry Point

```rust
pub fn format_module(module: &ModuleFile, indent_width: usize) -> String {
    let mut fmt = Formatter::new(&module.comments, indent_width);
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

### 5.3 Comment Insertion Rules

Before emitting any AST node with span `s`:
1. Write all `comments.leading_comments(s)`, each on its own line.
2. Emit the node.
3. Write all `comments.trailing_comments(s)` on the same line (if any).

Blank lines are preserved by checking the line distance between the previous emitted span and the current one.

> **Intra-expression comments** (comments inside expressions) are **out of scope** for MVP. They are explicitly deferred to a future formatter spec.

### 5.4 Expression Formatting

Expressions are formatted with standard precedence-aware parenthesization:
- Binary operators get spaces: `a + b`
- Function calls: `foo(a, b, c)`
- Pipelines/record literals use multi-line if any field spans multiple lines
- Blocks indent their body by one level

### 5.5 Semicolon Emission Rules

Ash statements are separated by newlines. The formatter **never emits semicolons** in the MVP because the surface syntax does not use them. If a future syntax introduces optional semicolons, the formatter will preserve them; until then, statements are terminated by `\n` only.

## 6. LSP Integration

Once the formatter exists, `textDocument/formatting` and `textDocument/rangeFormatting` become trivial handlers in `ash-lsp`:

```rust
async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>, Error> {
    let module = self.vfs.parse(uri)?;
    let formatted = ash_formatter::format_module(&module, 4);
    Ok(Some(vec![TextEdit {
        range: full_document_range,
        new_text: formatted,
    }]))
}
```

## 7. CLI Integration

Reconcile with **SPEC-005**:

```bash
ash fmt [options] <file.ash>
```

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without modifying |
| `--write` | Format files in place (default) |
| `--stdin` | Read from stdin, write to stdout |
| `--indent <n>` | Indent width (default 4) |

## 8. Error Handling

If the input cannot be parsed:
1. **CLI:** Print parse errors to stderr and exit with code 2. Do not write a formatted file.
2. **LSP:** Return an empty `TextEdit` list and let the server's diagnostic pipeline surface the parse errors.
3. **Library API (`format_module`):** Requires a pre-parsed `ModuleFile`; parsing is the caller's responsibility.

## 9. Testing Strategy

1. **Round-trip parsing:** For every example file, `parse_surface_file(format(parse_surface_file(src))) == parse_surface_file(src)` (AST equality **ignoring spans**, since formatting changes spans).
2. **Comment preservation:** Format a file with comments; verify all comment text appears in the output.
3. **Idempotency:** `format(format(source)) == format(source)`.
4. **Proptest:** Generate random valid ASTs (via `ash-core` proptest helpers), format them, and assert round-trip parse equality modulo spans.

## 10. Relationship to Other Specs

- **Blocked by:** SPEC-039 (comment trivia and binding spans)
- **Blocks:** None directly; enables LSP formatting
- **Follows:** SPEC-038 LSP MVP
