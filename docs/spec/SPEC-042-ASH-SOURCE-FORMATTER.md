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

## 3. Dependency on SPEC-039

The formatter **requires** the `CommentTable` introduced in SPEC-039. It cannot be implemented before comment trivia is available. Specifically, SPEC-039 must deliver:

1. `Comment` token kind in the lexer.
2. `CommentTable` side-table implementation.
3. `comments: CommentTable` field on `ModuleFile`.
4. `parse_module(source: &str) -> Result<ModuleFile, Vec<ParseError>>` entry point.
5. Binding spans (`Expr::Variable(Name, Span)`, `Pattern::Variable(Name, Span)`).

## 4. Core Design

### 4.1 Formatter State

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

### 4.2 Entry Point

```rust
pub fn format_module(module: &ModuleFile, indent_width: usize) -> String {
    let mut fmt = Formatter::new(&module.comments, indent_width);
    fmt.write_leading_comments(module.span);
    for def in &module.definitions {
        fmt.write_definition(def);
        fmt.write_newline();
    }
    fmt.output
}
```

### 4.3 Comment Insertion Rules

Before emitting any AST node with span `s`:
1. Write all `comments.leading_comments(s)`, each on its own line.
2. Emit the node.
3. Write all `comments.trailing_comments(s)` on the same line (if any).

Blank lines are preserved by checking the line distance between the previous emitted span and the current one.

### 4.4 Expression Formatting

Expressions are formatted with standard precedence-aware parenthesization:
- Binary operators get spaces: `a + b`
- Function calls: `foo(a, b, c)`
- Pipelines/record literals use multi-line if any field spans multiple lines
- Blocks indent their body by one level

## 5. LSP Integration

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

## 6. CLI Integration

Add `ash fmt` to SPEC-005:

```bash
ash fmt [options] <file.ash>
```

| Option | Description |
|--------|-------------|
| `--check` | Exit with error if file would change |
| `--indent <n>` | Indent width (default 4) |

## 7. Testing Strategy

1. **Round-trip parsing:** For every example file, `parse(format(parse(source))) == parse(source)` (AST equality).
2. **Comment preservation:** Format a file with comments; verify all comment text appears in the output.
3. **Idempotency:** `format(format(source)) == format(source)`.

## 8. Relationship to Other Specs

- **Blocked by:** SPEC-039 (comment trivia)
- **Blocks:** None directly; enables LSP formatting
- **Follows:** SPEC-038 LSP MVP
