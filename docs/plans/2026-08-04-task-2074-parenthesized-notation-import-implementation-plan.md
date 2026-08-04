# TASK-2074 Parenthesized Notation Import Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete TASK-2074 by adding explicit parenthesized notation imports, canonical public notation-summary transport, deterministic syntax dependency validation, imported notation activation, and the remaining expanded-graph evidence.

**Architecture:** `ash-parser` parses `use module::(pattern)` into typed AST, collects public notation summaries only from the acquired canonical module graph, orders syntax providers before consumers, and injects selected full-key notation entries into shallow module expansion. The boundary remains parser-owned and non-authorizing: it retains exact syntax provenance but performs no ordinary binding, filesystem lookup, raw-text recovery, Core/CPS lowering, Engine work, or runtime admission.

**Tech Stack:** Rust 2024 on Rust 1.94, `winnow`, `ash-parser`, `ash-core::module_graph`, existing surface expansion/origin/hygiene carriers, Cargo tests, semantic-task/traceability validators, Common Changelog, and Conventional Commits.

---

## Execution protocol

- Work directly on `main`. The user explicitly approved main-branch work, so do **not** create a new worktree even though the general skill default recommends one.
- Use `@superpowers:executing-plans`, `@task-development-using-tdd`, `@superpowers:test-driven-development`, and `@rust-skills` throughout.
- Follow `AGENTS.md`: the main agent delegates each RED test batch to a Test Development sub-agent, each GREEN implementation batch to a Code Development sub-agent, then uses separate QA, specification-review, and code-review sub-agents before closeout.
- Attempt Rust language-server workspace activation before symbol inspection. If it again reports `languages_started: []`, record that separately as baseline-only and use `rg` plus Cargo as final authority.
- Preserve the existing untracked `crates/ash-parser/tests/task_2074_expanded_graph_completion.rs`; it is required Task 6 evidence, not disposable scratch work.
- Stage only files named by the current checkpoint. Preserve unrelated user changes.
- Observe every intended RED failure before implementation. An intentional RED checkpoint may require `git commit --no-verify` because project hooks correctly reject failing tests; record the exact expected failure in the commit body. Never bypass hooks for a supposedly GREEN checkpoint.
- Commit after each bounded phase. The user requires all work to be committed; do not leave finished checkpoint changes unstaged or uncommitted.
- Do not push unless the user separately authorizes it.

### Task 1: Amend the canonical contract before behavior

**Files:**
- Modify: `docs/spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- Modify: `docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`
- Modify: `CHANGELOG.md`
- Reference only: `docs/plans/2026-08-04-task-2074-parenthesized-notation-import-design.md`

**Step 1: Delegate the contract amendment**

Ask a specification sub-agent to copy the approved design into the canonical authorities without broadening it. Require these exact decisions:

```text
use crate::math::(<*>);
use crate::ranges::(_ between _ and _);

- selector = exact normalized token/hole pattern
- no trailing whole-import `as` on a notation import; `as` inside a selector is an ordinary word token
- `use m::(*);` selects exact `*` notation, while `use m::*;` remains an ordinary glob; there is no separate notation-glob form
- transport every eligible public full-key variant
- full key = normalized pattern + fixity + associativity + precedence
- target callable identity/provenance is transported but never bound or authorized
- ordinary callable imports never activate notation
- missing/private/malformed/conflicting/cyclic syntax dependencies reject atomically
```

**Step 2: Update semantic coverage before Rust changes**

In the TASK-2074 row and section, retain the current axes `partial / tested / below_spec`, name the newly selected parenthesized syntax, and add deferred trace nodes for:

```text
IMPL-MODULE-CANONICAL-NOTATION-IMPORT
IMPL-MODULE-IMPORTED-NOTATION-ACTIVATION
TEST-MOD-REAL-001-002-NOTATION-IMPORT-PARSER
TEST-MOD-REAL-001-002-CANONICAL-NOTATION-SUMMARY
TEST-MOD-REAL-001-002-NOTATION-DEPENDENCY-REJECTION
TEST-MOD-REAL-001-002-IMPORTED-NOTATION-ACTIVATION
TEST-MOD-REAL-001-002-EXPANDED-GRAPH-COMPLETION
```

Mark the implementation/test nodes deferred or untested. Do not claim implementation before source and test evidence exists.

**Step 3: Update the active task record and changelog**

- Add the same deferred evidence IDs to TASK-2074 in `docs/plan/semantic-task-records.json`.
- Update the exact next obligation to the approved parenthesized notation-import work plus the existing completion-evidence target.
- Add an `[Unreleased]` `Changed` entry describing the contract decision, with `(TASK-2074)`.
- Keep TASK-2075 `Planned` and inactive.

**Step 4: Validate the documentation amendment**

Run:

```bash
python3 tools/docs/validate_semantic_task_records.py \
  --root . --manifest docs/plan/semantic-task-records.json
python3 tools/docs/validate_semantic_traceability.py \
  --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json --format json
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

Expected: every command exits 0; deferred nodes are connected but are not reported as tested implementation.

**Step 5: Request specification review**

Require the reviewer to verify the canonical text matches the approved design, especially the
`use m::(*);` versus `use m::*;` distinction, trailing-alias rejection, no target-callable
activation, and no runtime authority.

**Step 6: Commit the contract checkpoint**

```bash
git add docs/spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md \
  docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md \
  docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md \
  docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md docs/plan/PLAN-INDEX.md \
  docs/plan/SEMANTIC-RULE-COVERAGE.md docs/plan/semantic-task-records.json \
  docs/spec/SEMANTIC-TRACEABILITY.json \
  docs/plan/audits/AUDIT-207-module-realization-seams.md \
  docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md \
  CHANGELOG.md
git commit -m "docs(parser): specify parenthesized notation imports"
```

### Task 2: Parse parenthesized notation selectors

**Files:**
- Create: `crates/ash-parser/tests/task_2074_parenthesized_notation_import_parser.rs`
- Modify: `crates/ash-parser/src/use_tree.rs`
- Modify: `crates/ash-parser/src/parse_use.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/lib.rs` only if a new public carrier is not already reachable through the existing `pub use parse_use::*` / `use_tree` surface
- Regression: `crates/ash-parser/src/import_resolver.rs`
- Regression: `crates/ash-parser/tests/task_2067_canonical_module_graph.rs`
- Regression: `crates/ash-parser/tests/task_1730_notation_declaration_parser_ast.rs`

**Step 1: Delegate parser RED tests**

Create integration tests with these exact cases:

```rust
#[test]
fn parses_symbolic_parenthesized_notation_selector() {
    let module = ash_parser::parse_surface_file("use crate::math::(<*>);")?;
    // Assert module path [crate, math], normalized Token("<*>"), no holes,
    // complete selector span, complete Use::span, inherited visibility.
}

#[test]
fn parses_mixfix_selector_with_ordered_holes_and_tokens() {
    let module = ash_parser::parse_surface_file(
        "use crate::ranges::(_ between _ and _);",
    )?;
    // Assert [Hole, Token("between"), Hole, Token("and"), Hole].
}

#[test]
fn notation_selector_rejects_trailing_whole_import_alias() {
    // Reject `use crate::math::(<*>) as ap;`.
    // Accept `use crate::logic::(_ as _);`; `as` is a selector word token there.
}

#[test]
fn parenthesized_star_is_not_an_ordinary_glob() {
    // `use crate::math::(*);` is exact `*` notation selection.
    // `use crate::math::*;` remains the existing ordinary glob.
}
```

Also cover empty `()`, missing close parenthesis, doubled/leading/trailing invalid separators, a
selector without a hole or token, comments/whitespace normalization, `_name`/`__` word tokens, and
unchanged parsing of simple/glob/nested imports. Do not assert exact diagnostic raw text.

**Step 2: Run the parser target and observe RED**

```bash
cargo test -p ash-parser --test task_2074_parenthesized_notation_import_parser
```

Expected: compile failure because `UsePath::Notation` and its typed selector carriers do not exist. Fix fixture typos until the failure is only the missing API/behavior.

**Step 3: Commit the observed RED tests**

```bash
git add crates/ash-parser/tests/task_2074_parenthesized_notation_import_parser.rs
git commit --no-verify -m "test(parser): specify parenthesized notation imports"
```

**Step 4: Add typed AST carriers**

Replace declaration-only token extraction and selector-only parts with one span-preserving parsed
part carrier shared by notation declarations and imports:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NotationPatternPart {
    Hole { span: Span },
    Token { spelling: Box<str>, span: Span },
}

pub struct NotationPattern {
    pub raw: Box<str>, // retained diagnostic/backward-compatible spelling
    pub tokens: Vec<RawOperatorToken>, // retain existing diagnostic/backward-compatible consumers
    pub parts: Box<[NotationPatternPart]>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotationImportSelector {
    pub raw: Option<Box<str>>, // optional diagnostic spelling only
    pub parts: Box<[NotationPatternPart]>,
    pub span: Span,
}

pub enum UsePath {
    Simple(SimplePath),
    Glob(SimplePath),
    Nested(SimplePath, Vec<UseItem>),
    Notation {
        module: SimplePath,
        selector: NotationImportSelector,
    },
}
```

`NotationPattern.raw` and its existing symbolic `tokens` remain for diagnostic and backward
compatibility consumers; this task adds `parts` rather than replacing those fields. The import
selector may omit `raw` or retain it as optional diagnostic metadata. Tests and semantic matching
use normalized `parts`; neither declarations nor imports may reparse or compare raw spelling as a
semantic input.

**Step 5: Parse `::(...)` before ordinary path continuation**

- In `parse_path_segments`, stop before `::(` just as it already stops before `::*` and `::{...}`.
- In `parse_use_path`, recognize `::(` after the module path and call a dedicated `parse_notation_selector`.
- Tokenize `_` as a hole only when it is the complete selector atom; retain symbolic and identifier-like notation tokens in source order.
- Normalize insignificant whitespace without losing part spans.
- Reuse the same `NotationPatternPart` construction for `NotationDecl.pattern` and
  `NotationImportSelector`. Update `parse_module.rs` to populate declaration `parts` during the
  authoritative parse while preserving its existing `raw` and `tokens`; do not derive structured
  parts by scanning `raw` later.
- Preserve `task_1730_notation_declaration_parser_ast.rs` and every existing
  `canonical_syntax_dependencies.rs` consumer of `NotationPattern.raw`/`tokens` for diagnostic or
  compatibility behavior. Their semantic key construction must migrate to `parts`, without
  requiring raw equality.
- In `parse_use`, reject only a trailing whole-import alias when the path is `UsePath::Notation`.
  The word `as` within `(_ as _)` remains a valid `NotationPatternPart::Token`.
- Parse `::(*)` as exact `*` notation selection before ordinary path continuation. Preserve
  `::*` as the existing ordinary glob; do not add a separate notation-glob form.
- Update every exhaustive `UsePath` match in `import_resolver.rs` and tests. Ordinary binding must treat this variant as syntax-only and must not create an import binding.

**Step 6: Run parser and existing import regressions**

```bash
cargo test -p ash-parser --test task_2074_parenthesized_notation_import_parser
cargo test -p ash-parser parse_use::tests
cargo test -p ash-parser --test task_1730_notation_declaration_parser_ast
cargo test -p ash-parser --test task_2067_canonical_module_graph
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
cargo fmt --check
```

Expected: PASS with no warnings. The notation variant exists only as typed syntax.

**Step 7: Request code review and commit GREEN**

```bash
git add crates/ash-parser/src/use_tree.rs crates/ash-parser/src/parse_use.rs \
  crates/ash-parser/src/parse_module.rs crates/ash-parser/src/surface.rs \
  crates/ash-parser/src/import_resolver.rs crates/ash-parser/src/lib.rs \
  crates/ash-parser/tests/task_2074_parenthesized_notation_import_parser.rs
git commit -m "feat(parser): parse parenthesized notation imports"
```

### Task 3: Add canonical public notation-summary carriers

**Files:**
- Create: `crates/ash-parser/tests/task_2074_canonical_notation_import.rs`
- Modify: `crates/ash-parser/src/canonical_syntax_dependencies.rs`
- Modify: `crates/ash-parser/src/canonical_expanded_module_graph.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Reference: `crates/ash-parser/tests/task_1730_notation_declaration_parser_ast.rs`
- Reference: `crates/ash-parser/tests/task_1732_local_notation_table_resolution.rs`

**Step 1: Delegate carrier RED tests**

Use real `CanonicalModuleGraphResolver` fixtures. Add tests proving one import retains:

- canonical provider `ModuleKey`;
- normalized shared `NotationPatternPart` values including holes, with no raw-spelling equality requirement;
- every matching public full-key variant;
- `NotationFixity`, associativity, and precedence;
- callable target path, distinct from the notation identity;
- declaration visibility;
- provider declaration span and artifact/source provenance;
- exact consumer `Use::span`;
- read-only exposure through `CanonicalExpandedModuleRef::notation_imports()`.

Add a provider with two compatible public declarations sharing one normalized selector and assert both full keys are transported in deterministic key order.

**Step 2: Run the focused carrier test and observe RED**

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import canonical_public_notation_summary -- --exact
```

Expected: compile failure because canonical notation key/summary/import APIs are absent.

**Step 3: Commit the RED carrier tests**

```bash
git add crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit --no-verify -m "test(parser): specify canonical notation summaries"
```

**Step 4: Add canonical typed carriers**

Keep notation separate from macro aliases:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalNotationPatternPart {
    Hole,
    Token(Box<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalNotationKey {
    pattern: Box<[CanonicalNotationPatternPart]>,
    fixity: CanonicalNotationFixityKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalNotationSummary {
    key: CanonicalNotationKey,
    target: CallablePath,
    visibility: Visibility,
    declaration_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalNotationImport {
    provider_key: ModuleKey,
    summary: CanonicalNotationSummary,
    provider_source_path: Option<Box<str>>,
    provider_artifact_origin: ModuleArtifactOrigin,
    use_span: Span,
}
```

`CanonicalNotationPatternPart` is intentionally distinct from the shared parsed
`NotationPatternPart`: canonical keys must not contain source spans. Convert each parsed hole/token
part to this span-free form in order, preserving token spelling and hole order; never construct the
canonical form from `NotationPattern.raw` or rendered text.

Represent associativity and precedence explicitly in `CanonicalNotationFixityKey` or through an orderable equivalent of `NotationFixity`; do not key only by rendered pattern text.

**Step 5: Collect summaries from authoritative AST only**

- Add `collect_public_notation_summaries(&ModuleBody)` in `canonical_syntax_dependencies.rs` or a narrowly reusable parser-owned helper in `surface.rs`.
- Derive normalized token/hole parts from the shared parsed `NotationDecl.pattern.parts`, never
  from source text or optional diagnostic `raw` metadata.
- Reject a public declaration whose parsed pattern cannot produce a complete normalized key.
- Retain target `CallablePath`; do not resolve or bind it here.
- Add `notation_imports: Box<[CanonicalNotationImport]>` to each private expanded module record and a read-only accessor on `CanonicalExpandedModuleRef`.

**Step 6: Run carrier tests and regressions**

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import canonical_public_notation_summary -- --exact
cargo test -p ash-parser --test task_1730_notation_declaration_parser_ast
cargo test -p ash-parser --test task_1732_local_notation_table_resolution
cargo test -p ash-parser --lib
```

Expected: the carrier tests pass; imported notation is not activated yet.

**Step 7: Request review and commit the carrier**

```bash
git add crates/ash-parser/src/canonical_syntax_dependencies.rs \
  crates/ash-parser/src/canonical_expanded_module_graph.rs \
  crates/ash-parser/src/surface.rs crates/ash-parser/src/lib.rs \
  crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit -m "feat(parser): add canonical notation summaries"
```

### Task 4: Resolve notation dependencies and reject invalid graphs

**Files:**
- Modify: `crates/ash-parser/tests/task_2074_canonical_notation_import.rs`
- Modify: `crates/ash-parser/src/canonical_syntax_dependencies.rs`
- Modify: `crates/ash-parser/src/canonical_expanded_module_graph.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Regression: `crates/ash-parser/tests/task_2074_canonical_syntax_prepass.rs`

**Step 1: Delegate dependency RED tests**

Add separate tests for:

```text
private notation declaration
private structural provider path
missing selector summary
malformed selector/summary
local-versus-imported full-key overlap
imported-versus-imported full-key overlap
same pattern with incompatible precedence or associativity
two-module macro/notation cycle
three-module notation cycle
valid sibling plus invalid notation edge returns only Err
```

Every failure must assert consumer/provider keys, exact use span, all applicable provider declaration spans, artifact/source context, stable failure kind, and stable cycle edge order.

**Step 2: Observe RED**

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import notation_dependency -- --nocapture
```

Expected: behavioral failures because notation imports are not yet included in dependency requests, conflict detection, or cycle edges.

**Step 3: Commit the RED dependency tests**

```bash
git add crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit --no-verify -m "test(parser): specify notation dependency rejection"
```

**Step 4: Add dedicated anchored notation failures**

Do not overload macro-only names such as `PrivateMacro` or `DuplicateLocalName`. Add a typed failure surface such as:

```rust
pub enum CanonicalNotationImportFailureKind {
    PrivateModulePath,
    PrivateNotation,
    MissingSummary,
    MalformedPattern,
    ConflictingActiveKey,
    UnsupportedPath,
}

pub struct CanonicalNotationImportFailure {
    kind: CanonicalNotationImportFailureKind,
    consumer_key: ModuleKey,
    provider_key: Option<ModuleKey>,
    use_span: Span,
    declaration_spans: Box<[Span]>,
    // source paths and artifact origins parallel existing macro failure facts
}
```

Expose it through a dedicated `CanonicalModuleExpansionError::InvalidNotationImport` accessor and source chain.

**Step 5: Unify macro and notation ordering without merging identities**

- Extend `CanonicalSyntaxPrepass` with notation requests keyed by consumer.
- Resolve only `UsePath::Notation` against canonical keys.
- Select all public summaries whose normalized pattern exactly equals the selector.
- Add importer-to-provider edges to the same dependency graph used for macros.
- Deduplicate identical module edges while retaining every triggering use span needed for diagnostics.
- Stable-sort by provider key, use span, and full notation key.
- Run the existing provider-first DFS/topological ordering over the combined syntax graph.
- Return no prepass value on any failure.

**Step 6: Apply the existing notation-overlap rules before publication**

Construct the consumer's prospective active full-key set from local declarations plus imported summaries. Reject collisions deterministically; never choose by import order or target spelling. Keep complete namespace collection and ordinary import ambiguity in TASK-2075/TASK-2072.

**Step 7: Run focused and macro regression tests**

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import
cargo test -p ash-parser --test task_2074_canonical_syntax_prepass
cargo test -p ash-parser --lib
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
cargo fmt --check
```

Expected: notation dependency tests pass and the existing macro prepass remains 17/17 or higher.

**Step 8: Request specification/code review and commit**

```bash
git add crates/ash-parser/src/canonical_syntax_dependencies.rs \
  crates/ash-parser/src/canonical_expanded_module_graph.rs crates/ash-parser/src/lib.rs \
  crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit -m "feat(parser): validate notation dependencies"
```

### Task 5: Activate imported notation during shallow expansion

**Files:**
- Modify: `crates/ash-parser/tests/task_2074_canonical_notation_import.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/canonical_expanded_module_graph.rs`
- Modify: `crates/ash-parser/src/canonical_syntax_dependencies.rs` only if active-entry conversion belongs beside summaries
- Regression: `crates/ash-parser/tests/task_1723_notation_token_preservation.rs`
- Regression: `crates/ash-parser/tests/task_1724_operator_section_boundary.rs`
- Regression: `crates/ash-parser/tests/task_1733_operator_section_elaboration.rs`
- Regression: `crates/ash-parser/tests/task_1745_expansion_origin_chain.rs`

**Step 1: Delegate activation RED tests**

Add positive tests proving:

- imported `<*>` expands only in the importing consumer;
- provider, parent, and sibling modules do not receive the imported active entry;
- existing supported operator-section contexts select only compatible full-key variants;
- all variants matching the selected normalized pattern are transported before contextual selection;
- ambiguous overlapping compatible variants reject deterministically;
- `_ between _ and _` reaches the existing syntax-phase table with its three holes, two separator tokens, target, full key, and argument order preserved left-to-right;
- imported operator-section origin sidecars retain notation declaration and supported use-site provenance, while mixfix summaries retain declaration/import provenance in the handoff;
- `use crate::provider::between;` alone does not activate `_ between _ and _`;
- importing notation does not create a callable binding.

Do **not** add a generalized mixfix expression parser or elaborator merely to make `x between lo and hi` execute. TASK-2074 owns canonical summary transport and activation into the existing syntax-phase table. The existing local notation-resolution seam is `surface.rs`'s `LocalNotationTable` from TASK-1732 and operator-section elaboration from TASK-1733. Generalized mixfix use-site parsing/elaboration is absent and must remain an explicit follow-on ownership gap until a separate task is activated.

**Step 2: Parse-smoke every intended-valid expression fixture**

Run a dedicated parse-only filter before the semantic target:

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import \
  intended_notation_activation_fixtures_parse -- --exact
```

Parse-smoke the parenthesized selector and all existing supported operator-section fixtures. Do not introduce a generalized mixfix expression fixture that current parser ownership does not support.

**Step 3: Observe activation RED**

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import imported_notation -- --nocapture
```

Expected: imported entries remain inactive because `expand_module_body_shallow` does not yet receive the prepass-approved notation rows.

**Step 4: Commit the activation RED tests**

```bash
git add crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit --no-verify -m "test(parser): specify imported notation activation"
```

**Step 5: Accept imported entries in the local notation table**

Refactor the private table construction seam:

```rust
fn build_local_notation_table_for_definitions(
    definitions: &[Definition],
    imported: &[CanonicalImportedNotationEntry],
) -> Result<LocalNotationTable, ExpansionError>
```

- Insert local and imported rows using the same full-key overlap validator.
- Preserve canonical provider identity and import/declaration spans on imported rows.
- Extend `expand_module_body_shallow` to accept imported notation entries alongside imported macros.
- Pass only the current module's prepass-approved entries from `CanonicalExpandedModuleGraph::try_expand`.
- Add a crate-private table test proving `_ between _ and _` is active as one ordered full-key entry with hole-to-argument positions `[0, 1, 2]`, even though generalized mixfix use-site execution remains outside TASK-2074.

**Step 6: Resolve supported uses by exact pattern and use-site context**

- Match normalized pattern parts, not target callable names and not raw rendered text.
- Filter full-key variants using the parsed context and existing overlap rules.
- Reject zero or multiple compatible variants with anchored typed diagnostics.
- Elaborate only operator-section forms already supported by TASK-1733 into an ordinary `Expr::Call` to the retained `CallablePath`.
- Preserve mixfix hole order and target metadata in the active table handoff; do not claim generalized mixfix execution evidence.
- Retain `SurfaceOrigin::NotationExpansion`, operator/mixfix use origin, parent origin, and hygiene metadata without granting callable authority.

**Step 7: Run activation and historical notation regressions**

```bash
cargo test -p ash-parser --test task_2074_canonical_notation_import
cargo test -p ash-parser --test task_1723_notation_token_preservation
cargo test -p ash-parser --test task_1724_operator_section_boundary
cargo test -p ash-parser --test task_1732_local_notation_table_resolution
cargo test -p ash-parser --test task_1733_operator_section_elaboration
cargo test -p ash-parser --test task_1745_expansion_origin_chain
cargo test -p ash-parser --test task_1749_cross_boundary_hygiene_validation
cargo test -p ash-parser --lib
```

If `task_1749_cross_boundary_hygiene_validation` belongs to another package, run its exact package-qualified command reported by Cargo rather than deleting the regression.

Expected: all pass; the older callable-only nonactivation invariant remains true.

**Step 8: Record the compositional ownership boundary**

In the task evidence and semantic coverage prepared for closeout, report independently:

```text
TASK-2074 canonical notation transport: implemented/tested
TASK-2074 activation into existing LocalNotationTable: implemented/tested
generalized mixfix use-site parsing/elaboration: not implemented/none/below_spec
historical local resolver owner: TASK-1732/TASK-1733
next implementation owner: separately activated follow-on task, not inferred here
```

This prevents a successful transport test from being misreported as end-to-end mixfix execution.

**Step 9: Request code/spec review and commit**

```bash
git add crates/ash-parser/src/surface.rs \
  crates/ash-parser/src/canonical_expanded_module_graph.rs \
  crates/ash-parser/src/canonical_syntax_dependencies.rs \
  crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit -m "feat(parser): activate imported notation summaries"
```

### Task 6: Land file/inline, mutation, atomicity, and authority evidence

**Files:**
- Add the existing untracked file: `crates/ash-parser/tests/task_2074_expanded_graph_completion.rs`
- Modify: `crates/ash-parser/tests/task_2074_expanded_graph_completion.rs` only for API adjustments caused by the notation implementation
- Modify: `crates/ash-parser/tests/task_2074_canonical_notation_import.rs`
- Regression: `crates/ash-parser/tests/task_2074_canonical_expanded_module_graph.rs`
- Regression: `crates/ash-parser/tests/task_2074_canonical_syntax_prepass.rs`

**Step 1: Rebase the existing completion test onto the final APIs**

Preserve its reviewed evidence:

```text
typed span-free file/inline expanded projections
deterministic exhaustive 64-case loop
depth 1/2, source form, declaration order, alias, template, function-count dimensions
observable File versus Inline artifact origins before normalization
alias/provider-definition mutation detection
graph-wide prepass atomic rejection with exact anchors
callable import does not activate notation
direct orchestration source fence
behavioral expansion after acquired files are overwritten/deleted
ash-parser manifest has no ash-engine dependency
```

Extend its typed projection with `notation_imports()` and any newly supported operator-section expansion payloads. Exclude only filenames/artifact source-form tags, byte spans, and diagnostic display paths permitted by SPEC-103 §4. Do not add generalized mixfix expression variants in this task.

**Step 2: Add notation-specific mutations**

In `task_2074_canonical_notation_import.rs`, mutate one fact at a time and assert changed output or anchored rejection:

- token/hole pattern;
- fixity;
- precedence;
- associativity;
- notation visibility;
- structural provider visibility;
- provider key;
- dependency edge;
- provider declaration order;
- use order.

**Step 3: Run the completion and notation targets**

```bash
cargo test -p ash-parser --test task_2074_expanded_graph_completion
cargo test -p ash-parser --test task_2074_canonical_notation_import
cargo test -p ash-parser --test task_2074_canonical_expanded_module_graph
cargo test -p ash-parser --test task_2074_canonical_syntax_prepass
```

Expected: all pass. Record exact test counts; do not infer a count from filtered output.

**Step 4: Run the exact historical regression matrix**

```bash
cargo test -p ash-parser --test task_1725_expanded_surface_boundary
cargo test -p ash-parser --test task_1732_local_notation_table_resolution
cargo test -p ash-parser --test task_1755_macro_registry_scope
cargo test -p ash-parser --test task_1756_expression_macro_expansion
cargo test -p ash-parser --test task_1757_macro_origin_hygiene
cargo test -p ash-parser --test task_1769_hygienic_binder_macros
cargo test -p ash-parser --test task_2059_file_inline_module_unit_parity
cargo test -p ash-parser --test task_2067_canonical_module_graph
cargo test -p ash-parser --test task_1763_macro_summary_carriers
cargo test -p ash-parser --test task_1786_macro_identity
```

Expected: all pass. Report each target's actual count and aggregate only after summing observed results.

**Step 5: Run parser quality checks**

```bash
cargo test -p ash-parser --lib
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

Expected: PASS with no warnings.

**Step 6: Delegate QA and two reviews**

- QA sub-agent reruns the focused targets and the exact regression matrix.
- Code-review sub-agent checks Rust API design, error/source chains, deterministic ordering, no panic on user input, and no accidental authority.
- Specification-review sub-agent maps every approved-design clause to a source path and an exact test.
- Address every blocking finding test-first and rerun affected commands.

**Step 7: Commit completion evidence**

```bash
git add crates/ash-parser/tests/task_2074_expanded_graph_completion.rs \
  crates/ash-parser/tests/task_2074_canonical_notation_import.rs
git commit -m "test(parser): cover complete expanded graph invariants"
```

### Task 7: Reconcile evidence and close TASK-2074

**Files:**
- Modify: `docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`
- Modify: `CHANGELOG.md`

**Step 1: Perform a requirement-by-requirement completion audit**

Build a table for every TASK-2074 requirement and approved-design clause. For each row, cite:

```text
canonical spec anchor
production symbol/file
positive test
negative test
mutation test
parity/architecture evidence
exact fresh command/result
```

Treat missing or indirect evidence as incomplete. Do not close TASK-2074 merely because focused tests pass.

**Step 2: Promote only observed traceability nodes**

- Recompute SHA-256 fingerprints for final source and test files.
- Change deferred nodes to `implemented` / `tested` only when their exact anchors exist and fresh commands pass.
- Add `implemented_by` and `tested_by` edges for both MOD-REAL-001 and MOD-REAL-002 where applicable.
- Update `semantic-task-records.json` evidence arrays and exact command counts.
- Keep proof as `none`; tests are not proofs.
- Keep Type/runtime/client parity outside this parser task.

**Step 3: Decide TASK-2074 status from evidence**

- Mark TASK-2074 `Complete` only if the complete atomic expanded graph handoff, parenthesized notation transport/activation, normalized file/inline evidence, mutations, and authority fences are all proven.
- Otherwise retain `In progress`, `partial / tested / below_spec`, and state the exact remaining target-spec clause.
- Do not activate TASK-2075 unless TASK-2074's complete handoff is genuinely available.

**Step 4: Update changelog and orientation documents**

Add final `[Unreleased]` entries for the parser syntax, canonical notation-summary transport, imported activation, diagnostics, and completion evidence. Ensure every statement distinguishes parser-stage syntax activation from callable/runtime authority.

**Step 5: Run the final full verification gate**

```bash
cargo test -p ash-parser --test task_2074_parenthesized_notation_import_parser
cargo test -p ash-parser --test task_2074_canonical_notation_import
cargo test -p ash-parser --test task_2074_expanded_graph_completion
cargo test -p ash-parser --test task_2074_canonical_expanded_module_graph
cargo test -p ash-parser --test task_2074_canonical_syntax_prepass
cargo test -p ash-parser
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
cargo fmt --check
python3 tools/docs/validate_semantic_task_records.py \
  --root . --manifest docs/plan/semantic-task-records.json
python3 tools/docs/validate_semantic_traceability.py \
  --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json --format json
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-semantic-task-gate.sh --all
bash scripts/check-docs-gate.sh
git diff --check
```

Expected: every command exits 0. If the full semantic gate exposes a known unrelated workspace failure, record it precisely, run every TASK-2074-owned focused gate independently, and do not represent the unrelated failure as a TASK-2074 pass.

**Step 6: Request final independent review**

Require final QA, code review, and specification review against the requirement table. Fix all blocking findings and rerun the complete affected gate.

**Step 7: Commit closeout metadata**

```bash
git add docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md \
  docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md docs/plan/PLAN-INDEX.md \
  docs/plan/SEMANTIC-RULE-COVERAGE.md docs/plan/semantic-task-records.json \
  docs/spec/SEMANTIC-TRACEABILITY.json \
  docs/plan/audits/AUDIT-207-module-realization-seams.md \
  docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md \
  CHANGELOG.md
git commit -m "docs(parser): close TASK-2074 expanded graph"
```

**Step 8: Verify all work is committed**

```bash
git status --short
git log --oneline -8
```

Expected: no TASK-2074 files remain modified or untracked, and every planned checkpoint appears in history. Do not push without separate authorization.
