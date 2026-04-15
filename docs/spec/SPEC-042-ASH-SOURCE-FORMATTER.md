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

> **MVP limitation — literal comment loss:** Because SPEC-039 defers spans on `Expr::Literal` and `Pattern::Literal`, comments adjacent to literals will be silently dropped. This is an accepted MVP limitation and must be documented in the formatter test suite.

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
pub enum FormatCmd {
    Token(String),
    Space,
    Newline,
    Indent,
    Dedent,
}

pub struct Formatter<'a> {
    cmds: Vec<FormatCmd>,
    indent_level: usize,
    config: &'a FormatConfig,
    comments: &'a CommentTable,
}

impl<'a> Formatter<'a> {
    pub fn new(comments: &'a CommentTable, config: &'a FormatConfig) -> Self { ... }
}
```

### 5.3 Entry Points

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
        fmt.write_workflow_def(workflow);
        fmt.write_newline();
    }
    render(&fmt.cmds, config)
}
```

`format_range` formats only the AST nodes whose spans intersect the given `range`:

```rust
pub fn format_range(module: &ModuleFile, range: Span, config: &FormatConfig) -> String {
    let mut fmt = Formatter::new(&module.comments, config);
    for decl in &module.module_decls {
        if spans_intersect(decl.span, range) {
            fmt.write_module_decl(decl);
            fmt.write_newline();
        }
    }
    for def in &module.definitions {
        if spans_intersect(def_span(def), range) {
            fmt.write_definition(def);
            fmt.write_newline();
        }
    }
    if let Some(workflow) = &module.workflow {
        if spans_intersect(workflow.span, range) {
            fmt.write_workflow_def(workflow);
            fmt.write_newline();
        }
    }
    render(&fmt.cmds, config)
}
```

> **Note:** `format_range` does not attempt to preserve outer indentation; the caller (e.g., LSP) is responsible for adjusting the returned text to the starting column of the selected range.

```rust
impl<'a> Formatter<'a> {
    fn write_workflow_def(&mut self, workflow: &WorkflowDef) {
        self.write_keyword("workflow");
        self.write_space();
        self.write_ident(&workflow.name);
        if !workflow.type_params.is_empty() {
            self.write_type_params(&workflow.type_params);
        }
        self.write_param_list(&workflow.params);
        if let Some(ret) = &workflow.declared_return_type {
            self.write_space();
            self.write_punct("->");
            self.write_space();
            self.write_type(ret);
        }
        if !workflow.plays_roles.is_empty() {
            self.write_space();
            self.write_keyword("plays");
            self.write_space();
            self.write_role_refs(&workflow.plays_roles);
        }
        if !workflow.capabilities.is_empty() {
            self.write_space();
            self.write_keyword("capabilities");
            self.write_punct(":");
            self.write_space();
            self.write_capability_decls(&workflow.capabilities);
        }
        if let Some(contract) = &workflow.contract {
            self.write_space();
            self.write_contract(contract);
        }
        self.write_space();
        self.write_workflow(&workflow.body);
    }
}

pub fn render(cmds: &[FormatCmd], config: &FormatConfig) -> String {
    let mut output = String::new();
    let mut indent_level = 0;
    for cmd in cmds {
        match cmd {
            FormatCmd::Token(s) => output.push_str(s),
            FormatCmd::Space => output.push(' '),
            FormatCmd::Newline => {
                output.push('\n');
                for _ in 0..(indent_level * config.indent_width) {
                    output.push(' ');
                }
            }
            FormatCmd::Indent => indent_level += 1,
            FormatCmd::Dedent => indent_level = indent_level.saturating_sub(1),
        }
    }
    output
}
```

### 5.4 Width-Aware Layout (Two-Pass Mechanism)

The formatter uses a **direct recursive walk** over the AST with a small formatting IR. The IR consists of tokens, explicit line breaks, and indent/dedent commands. The walk generates the IR, which is then rendered to a `String` by `render`.

For nodes whose layout depends on width (e.g., parameter lists, record literals, constraint blocks, type constructor arguments), the formatter implements an **exact two-pass mechanism**:

1. **Speculative single-line pass:** Create a temporary `Formatter` instance (or a dedicated single-line accumulator). Emit the node using compact single-line rules: commas emit only `Space`, braces do not force `Newline`, and indentation commands are suppressed. Render the accumulated commands to a string using `render`.
2. **Width check:** If `max_width` is `Some(limit)` and the speculative string contains no forced newlines (e.g., from nested multi-line comments) and its length is ≤ `limit`, append the speculative command buffer to the primary formatter and return.
3. **Fallback multi-line pass:** If the speculative line exceeds `limit` or contains forced breaks, **discard** the speculative buffer and emit the node again using multi-line rules: each field/argument goes on its own line, braces open and close on dedicated lines, and the body is indented by one additional level.

This must be exposed as a helper on `Formatter`:

```rust
impl<'a> Formatter<'a> {
    /// Try to emit `f` in a single line. If the rendered result fits within
    /// `max_width`, returns `Some(cmds)` to be appended to the primary buffer.
    /// Otherwise returns `None`, and the caller must re-emit with multi-line rules.
    fn try_single_line<F>(&self, f: F) -> Option<Vec<FormatCmd>>
    where
        F: FnOnce(&mut Formatter),
    {
        let mut probe = Formatter::new(self.comments, self.config);
        f(&mut probe);
        let text = render(&probe.cmds, self.config);
        if let Some(limit) = self.config.max_width {
            if text.len() <= limit && !text.contains('\n') {
                return Some(probe.cmds);
            }
        }
        None
    }
}
```

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

> **Literal comment loss:** Because `Expr::Literal` and `Pattern::Literal` do not carry spans in the MVP AST, any comment that was originally attached to a literal node cannot be re-attached after formatting. The formatter will silently drop such comments. This is an accepted MVP limitation.

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
| `Receive { mode, arms, is_control, span }` | `receive` / `control receive` | `is_control` → prefix `control`; `NonBlocking` or `Blocking(None)` → `receive { arms }`; `Blocking(Some(d))` → `receive wait <duration> { arms }`; each arm indented +1 |
| `Yield { role, expr, resume_var, resume_type, arms, span }` | `yield` | `yield expr to role resume resume_var: resume_type { arms }`; arms indented +1 |
| `Resume { expr, ty, span }` | `resume` | `resume expr` |

> **Note:** The surface `Workflow` enum does not currently contain a `Spawn` variant. If one is added in the future, it shall follow the same pattern: keyword on the same line, any nested body indented +1.

### 5.9 YieldArm and ReceiveArm Formatting

`YieldArm` and `ReceiveArm` are formatted as comma-separated items inside brace-delimited blocks.

**`YieldArm`** is formatted as `pattern => body`, where `pattern` is formatted using standard `Pattern` rules and `body` is formatted as a nested workflow (indented +1 if multi-line). An optional trailing comma is allowed on the last arm; the formatter preserves a trailing comma only when the block is multi-line.

**`ReceiveArm`** is formatted as `pattern [if guard] => body`:
- `pattern` is a `StreamPattern`:
  - `StreamPattern::Wildcard` → `_`
  - `StreamPattern::Literal(s)` → `"s"`
  - `StreamPattern::Binding { capability, channel, pattern }` → `capability:channel as pattern`
- `guard` (if present) is prefixed by `if ` and formatted as an expression.
- `body` is formatted as a nested workflow (indented +1 if multi-line).
- Arms are separated by `, `. The formatter preserves a trailing comma only when the block is multi-line.

### 5.10 PolicyExpr and ConstraintBlock Formatting

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

### 5.11 Semicolon Emission Rules

Ash statements are separated by newlines. The formatter **never emits semicolons** in the MVP because the surface syntax does not use them. If a future syntax introduces optional semicolons, the formatter will preserve them; until then, statements are terminated by `\n` only.

### 5.12 Type Formatting

All `surface::Type` variants must be formatted as follows:

| Variant | Formatting Rule |
|---|---|
| `Name(n)` | `n` |
| `List(t)` | `[t]` (no space inside brackets) |
| `Record(fields)` | `{ field1: t1, field2: t2 }` (multi-line via §5.4 if too long) |
| `Capability(n)` | `cap n` |
| `Constructor { name, args }` | `name<a1, a2>` (args comma+space separated; `<>` omitted if empty) |
| `Fn(params, ret)` | `Fn(p1, p2) -> ret` (params comma+space separated) |
| `Associated { base, name }` | `base::name` |

### 5.13 Pattern Formatting

All `surface::Pattern` variants must be formatted as follows:

| Variant | Formatting Rule |
|---|---|
| `Variable(name)` | `name` |
| `Wildcard` | `_` |
| `Tuple(pats)` | `(a, b, c)` (comma+space separated) |
| `Record(fields)` | `{ field: pat, ... }` (multi-line via §5.4 if too long) |
| `List { elements, rest }` | `[a, b, ..rest]` (comma+space separated; `..rest` only if present) |
| `Literal(lit)` | formatted via literal rules |
| `Variant { name, fields, payload }` | `name` followed by payload shape:
  - `Unit` → `name`
  - `Record(fields)` → `name { field: pat, ... }`
  - `Tuple(pats)` → `name(a, b, c)` |

### 5.14 Guard Formatting

All `surface::Guard` variants must be formatted as follows:

| Variant | Formatting Rule |
|---|---|
| `Always` | `always` |
| `Never` | `never` |
| `Pred(p)` | `name(args)` (predicate formatted like a function call) |
| `And(l, r)` | `l and r` (spaces around `and`) |
| `Or(l, r)` | `l or r` (spaces around `or`) |
| `Not(g)` | `not g` (space after `not`) |

### 5.15 Definition Subtypes Formatting

Each `Definition` subtype must be formatted as follows:

**`CapabilityDef`**
```
[visibility] capability name: effect(params) [-> ret] [where constraints] [=> provider action]
```
- Visibility omitted if `Inherited`.
- Constraints formatted comma-separated after `where`.
- Target (`=> provider action`) emitted only if both `target_provider` and `target_action` are `Some`.

**`PolicyDef`**
```
policy name [type_params] {
    field1: ty1 [= default1],
    ...
} [where expr]
```
- Fields are comma-separated, one per line if multi-line.
- Default values preceded by ` = ` when present.

**`RoleDef`**
```
role name {
    capabilities cap1, cap2,
    obligations ob1, ob2,
}
```
- Both clauses indented +1 if the block is multi-line.

**`ProxyDef`**
```
[visibility] proxy name for role {
    observes cap1, cap2,
    receives cap1:channel1, cap2,
} -> body
```
- Body formatted as a nested workflow.

**`InterfaceDef`**
```
[visibility] interface name [type_params] {
    method1(params) -> ret,
    method2(params) -> ret,
    type Assoc1;
    type Assoc2;
}
```
- Methods and associated types separated by newlines, indented +1 inside braces.

**`ImplDef`**
```
[visibility] impl [type_params] Interface<type_args> for Type [where bounds] {
    type Assoc = Type;
    method1(params) = body,
    ...
}
```
- Associated type bindings and methods each on their own line, indented +1.

**`FnDef`**
```
[visibility] fn name[type_params](params) [-> ret] [contract] { body }
```
- Body is an `Expr` block (see §5.7).

### 5.16 ModuleDecl and Import Formatting

**`ModuleDecl`** (from `crate::module`)
- File-based: `[visibility] mod name;`
- Inline: `[visibility] mod name {\n    definitions...\n}` (definitions indented +1, blank lines collapsed to 0 inside)

**`Use` (Import)**
- `Use` statement: `[visibility] use path [as alias];`
- `UsePath::Simple(path)` → segments joined by `::`
- `UsePath::Glob(path)` → `path::*`
- `UsePath::Nested(path, items)` → `path::{item1, item2 as alias2}` (items comma+space separated; nested multi-line via §5.4 if too long)

**`DependencyDecl`**
- `dependency alias from "path";`

### 5.17 MatchArm and BlockStmt Formatting

**`MatchArm`**
- Formatted as `pattern => expr`
- Arms are comma-separated inside the match braces.
- Multi-line match blocks place each arm on its own line indented +1.

**`BlockStmt`**
- `Let { pattern, expr, .. }` → `let pattern = expr;` (terminated by newline, no semicolon emitted per §5.11)

### 5.18 Visibility Formatting

`Visibility` is formatted as a prefix with a trailing space when non-inherited:

| Variant | Output |
|---|---|
| `Inherited` | *(nothing)* |
| `Public` | `pub ` |
| `Crate` | `pub(crate) ` |
| `Super { levels }` | `pub(super) ` if `levels == 1`, otherwise `pub(super::super) ` repeated |
| `Self_` | `pub(self) ` |
| `Restricted { path }` | `pub(in path) ` |

### 5.19 Constraint and Predicate Formatting

**`Predicate`**
- Formatted as a function-style call: `name(arg1, arg2, ...)` (comma+space separated).

**`Constraint`**
- Wraps a single `Predicate`; formatted identically to the predicate: `name(arg1, arg2, ...)`.
- Inside a `CapabilityDef`, multiple constraints appear after `where ` separated by commas.

## 6. LSP Integration

Once the formatter exists, `textDocument/formatting` and `textDocument/rangeFormatting` become handlers in `ash-lsp`:

```rust
async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>, Error> {
    let module = self.vfs.parse(uri)?;
    let formatted = ash_formatter::format_module(&module, &FormatConfig { indent_width: 4, max_width: None });
    Ok(Some(vec![TextEdit {
        range: full_document_range,
        new_text: formatted,
    }]))
}

async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> Result<Option<Vec<TextEdit>>, Error> {
    let module = self.vfs.parse(uri)?;
    let span = lsp_range_to_span(params.range);
    let formatted = ash_formatter::format_range(&module, span, &FormatConfig { indent_width: 4, max_width: None });
    Ok(Some(vec![TextEdit {
        range: params.range,
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

1. **Round-trip parsing:** For every example file, `parse_surface_file(format(parse_surface_file(src)))` must produce an AST that is structurally equal to `parse_surface_file(src)` (equality **ignoring spans**, since formatting changes spans).
2. **Comment preservation:** Format a file with comments; verify all comment text appears in the output. Exclude literal-adjacent comments per the accepted MVP limitation (§4, §5.6).
3. **Round-trip stability (idempotency replacement):**
   - Let `m0 = parse_surface_file(src)`.
   - Let `m1 = parse_surface_file(format_module(m0))`.
   - Assert `m0` and `m1` are structurally identical ASTs (ignoring spans).
   - Let `text1 = format_module(m0)` and `text2 = format_module(m1)`.
   - Assert `text1 == text2`.
   This property depends on **comment re-attachment stability**. The formatter must use a **span-independent or two-pass algorithm** so that reformatting already-formatted code does not shift spans in a way that changes comment placement. Comments must be re-attached using a stable key (e.g., node identity or original span) that is invariant across passes.
4. **Proptest:** Generate random valid ASTs (via `ash-core` proptest helpers), format them, and assert round-trip parse equality modulo spans.

## 10. Relationship to Other Specs

- **Blocked by:** SPEC-039 (comment trivia and binding spans)
- **Blocks:** None directly; enables LSP formatting
- **Follows:** SPEC-038 LSP MVP
