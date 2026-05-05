# Design Note: Structured Diagnostics and Multi-Format Rendering

## Status

Draft design note for future spec work. This document is non-normative: it records the desired architecture and open design boundaries for a future diagnostics spec/plan packet.

## 1. Question

How should Ash represent, render, and transport compiler/runtime diagnostics so that:

- the default human output approaches the quality of Rust and Elm errors;
- machine-readable formats such as JSON and YAML expose the same diagnostic content without string parsing;
- LSP diagnostics are a projection of the same structure rather than a separate error path;
- future tools, agents, CI systems, and LLMs can consume diagnostics over stable wire formats?

## 2. Core Decision

Ash diagnostics should be **structured semantic objects first**, with multiple renderers/projections layered on top.

The canonical flow should be:

```text
parser / typechecker / engine / interpreter
        ↓
Structured AshDiagnostic
        ↓
Renderer / projection
        ├── human terminal output
        ├── JSON
        ├── YAML
        ├── LSP Diagnostic
        ├── compact one-line output
        └── future LLM/tool profiles
```

`Display` strings must not be the canonical diagnostic representation. `Display` can remain a fallback/debug view, but JSON, YAML, LSP, terminal output, and future agent transports should all come from the same structured diagnostic object.

## 3. Motivation

Current Ash errors are mostly rendered by flattening internal error types through `Display` / `thiserror` strings. This creates several user-facing and tooling-facing problems:

1. Source spans exist in parser/typechecker errors but are not consistently rendered in the CLI.
2. Parse failures can leak internal parser debug output such as `ContextError { context: [], cause: None }`.
3. Type errors can leak internal representations when `Debug` formatting is used for type values.
4. Human output, JSON output, and LSP output can drift because they are formatted by separate code paths.
5. Agents and tools must parse prose if they want facts such as expected type, found type, capability name, or obligation name.

Rust and Elm demonstrate the target quality for human diagnostics: precise source snippets, labeled spans, notes, help text, suggested fixes, and no compiler-internal jargon. Ash should preserve that quality while also making the same structure available to machines.

## 4. Canonical Diagnostic Model

A future spec should define a canonical diagnostic structure roughly equivalent to:

```rust
pub struct AshDiagnostic {
    pub id: DiagnosticId,
    pub severity: Severity,
    pub phase: DiagnosticPhase,
    pub category: DiagnosticCategory,

    pub title: String,
    pub message: String,

    pub primary_span: Option<SourceSpan>,
    pub labels: Vec<DiagnosticLabel>,
    pub notes: Vec<DiagnosticNote>,
    pub help: Vec<DiagnosticHelp>,
    pub related: Vec<RelatedDiagnostic>,

    pub machine: MachineDiagnosticData,
}
```

The exact Rust representation is future spec work. The important architectural rule is that this object is the source of truth for all renderers and transports.

### 4.1 Diagnostic identity

Diagnostics should keep stable IDs such as `E140`, but a future spec should decide whether IDs need namespaced families:

```text
parse/E001
name/E220
type/E140
effect/E310
runtime/E500
```

Stable IDs serve three audiences:

- humans searching documentation;
- editors and CI systems grouping failures;
- agents/tools applying structured repair policies.

### 4.2 Severity

Severity should remain compatible with LSP-style severities:

```text
error
warning
information
hint
```

Ash may add internal subcategories, but transport projections should preserve the common severities.

### 4.3 Phase

Diagnostics should identify the compiler/runtime phase that produced them:

```text
parse
lower
name-resolution
typecheck
effect-check
capability-check
engine
runtime
tooling
```

This is especially useful for agents and CI systems because it distinguishes syntax mistakes from semantic, runtime, and tooling failures.

### 4.4 Labels and spans

A diagnostic should support one primary span and zero or more labeled spans.

Example label roles:

```text
primary
secondary
context
definition-site
use-site
expected-here
found-here
requirement-site
obligation-site
capability-site
```

The future spec should decide whether label roles are an enum, an open string namespace, or both.

### 4.5 Notes and help

Diagnostics should distinguish explanatory notes from actionable help:

- `note`: extra explanation, context, or rule citation;
- `help`: a likely user action;
- `suggestion`: a help item with an optional source edit.

For example:

```text
note: this block is currently Pure
help: move this call into an Act block or admit the required capability
```

### 4.6 Machine fields

Machine-readable formats need facts, not only prose. The structured diagnostic should carry machine fields such as:

```json
{
  "expected_type": "Int",
  "found_type": "String",
  "capability": "http::get",
  "required_effect": "Network",
  "actual_context": "Pure"
}
```

These fields should be stable enough for tools to consume, but not every diagnostic needs the same schema. A future spec should define either:

1. per-category machine payloads, or
2. an open JSON-object payload with documented keys for common categories.

## 5. Default Human Renderer

The default CLI output should be Rust/Elm-like: structured, source-centered, and helpful.

Example target shape:

```text
error[E140]: type mismatch
  --> src/main.ash:5:13
   |
 5 |     let y = x + "hello"
   |             ^   ------- found String here
   |             |
   |             expected Int because `x` has type Int
   |
help: use string concatenation or convert the integer explicitly
```

For Ash effect/capability errors:

```text
error[E310]: capability is not available in this context
  --> workflow.ash:12:9
   |
12 |         http::get(url)
   |         ^^^^^^^^^^^^^^ requires Network capability
   |
note: this block is currently Pure
help: move this call into an Act block or admit the required capability
```

The human renderer should avoid exposing internal parser, typechecker, or runtime implementation names unless those names are part of the Ash surface language.

## 6. JSON Renderer

JSON output should serialize the canonical diagnostic structure, not wrap human prose.

Illustrative shape:

```json
{
  "schema_version": "ash-diagnostic/v1",
  "diagnostics": [
    {
      "id": "E140",
      "severity": "error",
      "phase": "typecheck",
      "category": "type_mismatch",
      "title": "type mismatch",
      "message": "expected Int, found String",
      "primary_span": {
        "file": "src/main.ash",
        "start": { "line": 5, "column": 13, "byte": 84 },
        "end": { "line": 5, "column": 14, "byte": 85 }
      },
      "labels": [
        {
          "role": "primary",
          "message": "expected Int here",
          "span": {
            "file": "src/main.ash",
            "start": { "line": 5, "column": 13, "byte": 84 },
            "end": { "line": 5, "column": 14, "byte": 85 }
          }
        },
        {
          "role": "secondary",
          "message": "found String here",
          "span": {
            "file": "src/main.ash",
            "start": { "line": 5, "column": 17, "byte": 88 },
            "end": { "line": 5, "column": 24, "byte": 95 }
          }
        }
      ],
      "notes": [],
      "help": [
        {
          "message": "use string concatenation or convert the integer explicitly",
          "replacement": null
        }
      ],
      "machine": {
        "expected_type": "Int",
        "found_type": "String"
      }
    }
  ]
}
```

JSON is the primary machine/wire format and should be versioned explicitly.

## 7. YAML Renderer

YAML should expose the same structure as JSON, with only serialization differences. It is useful for users and tools that prefer readable structured output.

Illustrative shape:

```yaml
schema_version: ash-diagnostic/v1
diagnostics:
  - id: E140
    severity: error
    phase: typecheck
    category: type_mismatch
    title: type mismatch
    message: expected Int, found String
    primary_span:
      file: src/main.ash
      start: { line: 5, column: 13, byte: 84 }
      end: { line: 5, column: 14, byte: 85 }
    labels:
      - role: primary
        message: expected Int here
        span:
          file: src/main.ash
          start: { line: 5, column: 13, byte: 84 }
          end: { line: 5, column: 14, byte: 85 }
    help:
      - message: use string concatenation or convert the integer explicitly
    machine:
      expected_type: Int
      found_type: String
```

The future spec should decide whether YAML is a first-class supported format or an optional CLI feature gated by dependency policy.

## 8. LSP Projection

LSP diagnostics should be a projection of `AshDiagnostic`, not a separate representation.

Mapping sketch:

| AshDiagnostic field | LSP field |
| --- | --- |
| `primary_span` | `range` |
| `severity` | `severity` |
| `id` | `code` |
| `message` / renderer summary | `message` |
| secondary labels | `relatedInformation` |
| notes/help/machine payload | `data` |
| source | `source = "ash"` |

LSP's built-in diagnostic shape is narrower than Ash's desired structure, so the projection should preserve rich data inside `Diagnostic.data` for Ash-aware clients.

Illustrative LSP `data` payload:

```json
{
  "ash_schema_version": "ash-diagnostic/v1",
  "category": "type_mismatch",
  "phase": "typecheck",
  "labels": [...],
  "notes": [...],
  "help": [...],
  "machine": {
    "expected_type": "Int",
    "found_type": "String"
  }
}
```

This keeps basic editor compatibility while allowing richer Ash editor extensions, agents, and code-action providers.

## 9. CLI Format Surface

A future CLI contract should expose the renderer explicitly:

```bash
ash check file.ash
ash check file.ash --format human
ash check file.ash --format json
ash check file.ash --format yaml
ash check file.ash --format compact
ash check file.ash --format lsp-json
```

Default output should be `human`.

Existing machine-readable output should be migrated toward the canonical diagnostic structure rather than maintained as a separate schema with only string messages.

## 10. LLM / Agent Consumption

A future LLM-oriented profile may be useful, but it should not be a separate diagnostic model.

Possible profiles:

```bash
ash check file.ash --format json --profile llm
ash check file.ash --format json --include-source-context
ash check file.ash --format json --include-repair-hints
```

Potential LLM/agent additions:

- bounded surrounding source context;
- normalized type representations;
- likely repair intent;
- source edits where safe;
- relevant imports and definitions;
- links to diagnostic documentation or Ash specs;
- redaction/minimization controls for remote tools.

The base JSON should remain useful without an LLM-specific profile. The LLM profile is an enrichment/projection decision, not a separate source of truth.

## 11. Relationship to Existing Crates

Current relevant surfaces include:

- `ash-diagnostic`: already owns `Span`, `Severity`, `DiagnosticCode`, and `AshLspError`.
- `ash-parser`: owns parse errors and source spans.
- `ash-typeck`: owns many typed semantic errors and diagnostic-code mappings.
- `ash-engine`: wraps parse/type/runtime errors and exposes check/run entry points.
- `ash-cli`: renders human and JSON output for CLI commands.
- `ash-lsp-core` / `ash-lsp`: project diagnostics into editor protocol types.
- `ash-repl`: has an isolated source-line/caret formatter that may inform the shared renderer.

Future spec work should inspect these live APIs before assigning ownership. A likely direction is to extend `ash-diagnostic` into the canonical diagnostic model and add renderer/projection modules either inside that crate or in a small companion crate.

## 12. Migration Strategy

A low-risk migration should be staged:

1. Introduce `AshDiagnostic` and conversion traits without changing user output.
2. Convert parser errors into structured diagnostics and stop leaking parser internals.
3. Convert core typechecker errors into structured diagnostics with stable machine fields.
4. Add a shared human renderer and use it from `ash check`.
5. Rebuild JSON output from `AshDiagnostic` rather than ad-hoc CLI structs.
6. Add YAML only after the JSON schema is stable enough.
7. Project LSP diagnostics from `AshDiagnostic` and preserve rich data in `Diagnostic.data`.
8. Add optional repair/source-edit fields once the core structure is stable.

This avoids a rewrite and lets each diagnostic family improve independently.

## 13. Non-Goals

This design note does not require:

- immediate i18n/localization;
- immediate replacement of every `thiserror` `Display` string;
- a full diagnostic-code taxonomy in this document;
- code-action implementation;
- guaranteed automatic repair;
- LLM-specific output as the first slice;
- choosing a rendering dependency such as `annotate-snippets`, `ariadne`, or a custom renderer.

Those are future spec/plan decisions.

## 14. Future Spec Starting Points

A future implementation-grade spec should decide:

1. the exact `AshDiagnostic` Rust types and ownership crate;
2. schema versioning and stability policy for JSON/YAML;
3. how diagnostic IDs are allocated and documented;
4. the exact source-span model, including byte offsets versus line/column ranges;
5. secondary label semantics and whether label roles are closed or extensible;
6. the machine payload policy for common categories such as type mismatch, unbound name, missing capability, unsatisfied obligation, and parse error;
7. whether YAML is first-class or optional;
8. how LSP `Diagnostic.data` should encode full Ash diagnostics;
9. whether source snippets are included in JSON by default or only via an opt-in flag;
10. how human renderer snapshots are tested;
11. how structured diagnostic tests assert code, severity, span, labels, notes, help, and machine fields rather than only message substrings.

## 15. Decision Summary

Ash should adopt a diagnostic architecture where:

- structured diagnostics are the canonical representation;
- human, JSON, YAML, compact, LSP, and future LLM/tool outputs are renderers/projections;
- `Display` is fallback/debug only;
- machine-readable outputs expose facts, spans, labels, notes, help, and category-specific machine fields;
- default human output should be source-centered and Rust/Elm-like.

This direction supports better user experience and better agent/tool interoperability without coupling the compiler's internal error types to any one output format.
