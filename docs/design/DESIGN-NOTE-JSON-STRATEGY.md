# Design Note: JSON Strategy for Ash

## Status: Draft (Decision D3)

## 1. Question

Should `std::json` be:

- **(A)** A pure-Ash JSON parser (hand-written recursive descent)?
- **(B)** A thin wrapper around a Rust JSON crate (e.g. `serde_json`)?
- **(C)** A hybrid: Rust-backed parse/generate with a pure-Ash AST and filter API?

## 2. Analysis

### 2.1 Why not (A) pure-Ash parser now

A full JSON parser in Ash would require:
- String slicing and character-level iteration (missing from stdlib)
- Recursive descent or parser-combinator infrastructure (not available in stdlib)
- Performance validation against real payloads

This is a 20–30h greenfield project. It would delay the spec processor, the dashboard backend (Case E), and any other feature needing JSON. While a pure-Ash parser is desirable for bootstrapping and proof scope, it is not MVP-critical.

### 2.2 Why not (B) thin wrapper alone

A thin Rust wrapper (e.g. `serde_json` bindings) would work immediately but would:
- Add an external dependency to the core runtime
- Prevent JSON manipulation from being used inside pure `orient` blocks unless the wrapper exposes a deep Ash-side AST
- Make Pandoc-filter compatibility harder, because the JSON representation would be Rust-controlled

### 2.3 Resolution: (C) Hybrid — Rust-backed engine with pure-Ash AST

`std::json` exposes two layers:

**Layer 1: Engine-backed parse/generate**
- `json::parse(text: String) -> JsonValue` → delegates to `serde_json` (or equivalent) in Rust
- `json::stringify(value: JsonValue) -> String` → delegates to Rust

**Layer 2: Pure-Ash AST and traversal API**
- `JsonValue` is an Ash ADT: `Null | Bool(Bool) | Number(Float) | String(String) | Array(List<JsonValue>) | Object(Map<String, JsonValue>)`
- All traversal, filtering, and manipulation happen in pure Ash
- This makes JSON a first-class citizen: you can pattern-match on `JsonValue` inside workflows

**Why this fits Ash:**
- The engine handles the messy parsing/serialisation boundary
- The language handles the semantic manipulation
- The AST shape is under Ash control, so Pandoc-filter alignment is possible
- Future replacement of the Rust backend with a pure-Ash parser is a non-breaking change

## 3. `JsonValue` AST shape

```ash
pub enum JsonValue {
    Null,
    Bool(Bool),
    Number(Float),
    String(String),
    Array(List<JsonValue>),
    Object(Map<String, JsonValue>),
}
```

This shape is intentionally close to:
- `serde_json::Value`
- Pandoc's JSON filter representation (nested `Object` / `Array` / `String`)
- CommonMark AST if we later unify JSON serialisation for `std::markdown`

## 4. Interface contract

```ash
pub fn parse(text: String) -> Result<JsonValue, JsonError>;
pub fn stringify(value: JsonValue) -> Result<String, JsonError>;
pub fn stringify_pretty(value: JsonValue) -> Result<String, JsonError>;

// Pure-Ash accessors (no engine calls)
pub fn is_null(v: JsonValue) -> Bool;
pub fn as_string(v: JsonValue) -> Option<String>;
pub fn get(v: JsonValue, key: String) -> Option<JsonValue>;
pub fn get_index(v: JsonValue, index: Int) -> Option<JsonValue>;
```

## 5. Relationship to `std::markdown` Pandoc compatibility

The `std::markdown` AST must be serialisable to JSON in a shape compatible with Pandoc filters. The `std::json` AST is the target representation. Therefore:

- `markdown::to_pandoc_json(doc: MarkdownDoc) -> JsonValue` is a pure-Ash function
- It constructs `JsonValue::Object` and `JsonValue::Array` nodes to match Pandoc's expected shape
- The engine-backed `json::stringify` handles the final JSON text emission

## 6. Future path to pure-Ash parser

When Ash has:
- String slicing (`string::slice`)
- Character iteration (`string::chars`)
- Sufficient performance for parser combinators

Then `json::parse` can be reimplemented in pure Ash without changing the `JsonValue` AST or the public API. The Rust backend becomes an optional optimisation, not a hard dependency.

## 7. Decision

**Adopt option (C).** `std::json` is a Rust-backed parse/serialise layer with a pure-Ash `JsonValue` AST. This unblocks the spec processor, preserves future bootstrapping freedom, and enables Pandoc-filter alignment for `std::markdown`.

This unblocks:
- `std::json` interface stub (Task B3)
- JSON report serialization in the spec processor
- Pandoc JSON filter compatibility in `std::markdown`
- Dashboard backend (Case E) and agent-pipeline loader (Case B)
