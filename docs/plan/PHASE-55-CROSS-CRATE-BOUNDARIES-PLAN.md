# Phase 55: Real Cross-Crate Boundary Enforcement

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Add source-defined crate loading and dependency syntax, then enforce real cross-crate visibility boundaries across module loading, import resolution, and type checking.

**Source:** Follow-up to TASK-329 verification findings and the deliberate single-root limitation documented in Phase 54.

**Priority:** Critical (spec compliance and security boundary)

**Estimated Duration:** 12-16 hours

---

## Overview

This phase addresses five linked gaps:

1. **User-Facing Syntax (Critical):** Ash source has no canonical way to declare crate identity or dependency crate roots.
2. **Graph Model (Critical):** `ModuleGraph` only models one crate root and cannot answer real cross-crate boundary questions.
3. **Crate Loading (Critical):** `ModuleResolver` resolves one crate at a time and uses ad hoc line scanning rather than parsed root metadata.
4. **Import Enforcement (Critical):** `ImportResolver` only understands `crate::...` and cannot enforce visibility for external crates.
5. **Parity and Confidence (High):** Type-checker and parser tests need explicit multi-crate coverage, not single-root heuristics.

### Target Surface

```ash
crate app;

dependency util from "../util/main.ash";
dependency policy from "../policy/main.ash";

use external::util::sanitize::normalize;
use crate::workflows::main;
```

### Explicitly Out of Scope

- CLI entrypoints
- Package registry/version resolution
- Build artifact caching
- New re-export chains beyond current resolver scope

---

## Task Summary

| Task | Description | Files | Est. Hours |
|------|-------------|-------|------------|
| TASK-337 | Add crate root and dependency syntax | Specs, parser AST/parser | 2-3 |
| TASK-338 | Extend module graph with crate identity | `ash-core/src/module_graph.rs` | 2 |
| TASK-339 | Implement dependency-aware multi-crate loading | `ash-parser/src/resolver.rs` | 3-4 |
| TASK-340 | Resolve external imports and enforce cross-crate visibility | `ash-parser/src/import_resolver.rs`, `parse_use.rs` tests | 2-3 |
| TASK-341 | Align type checker and add multi-crate regression coverage | `ash-typeck`, parser integration tests | 2-3 |
| TASK-342 | Final verification and closeout | All | 1 |

---

## TASK-337: Add Crate Root and Dependency Syntax

**Objective:** Introduce canonical Ash source syntax for crate identity and dependency roots, and parse it into explicit AST structures.

**Background:** Today the module system only supports single-crate `mod` discovery. Real cross-crate boundaries require source-visible crate names and declared dependency aliases.

**Files to Modify:**
- `docs/spec/SPEC-009-MODULES.md`
- `docs/spec/SPEC-012-IMPORTS.md`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/lib.rs`
- `crates/ash-parser/src/parse_crate_root.rs` (new)
- `crates/ash-parser/tests/crate_root_parser.rs` (new)

### Step 1: Write failing parser tests

Cover:
- `crate app;`
- multiple `dependency <alias> from "<path>";` lines
- missing crate name
- dependency without quoted path
- dependency metadata rejected outside crate root parsing

### Step 2: Add AST types

Introduce explicit surface structures such as:

```rust
pub struct CrateRootMetadata {
    pub crate_name: Box<str>,
    pub dependencies: Vec<DependencyDecl>,
    pub span: Span,
}

pub struct DependencyDecl {
    pub alias: Box<str>,
    pub root_path: Box<str>,
    pub span: Span,
}
```

### Step 3: Implement the root metadata parser

Expected surface:

```ash
crate app;
dependency util from "../util/main.ash";
dependency policy from "../policy/main.ash";
```

### Step 4: Update spec text and examples

- `SPEC-009`: crate root metadata and dependency declaration rules
- `SPEC-012`: `external::<alias>::...` import examples and resolution notes

### Step 5: Verify

```bash
cargo test --package ash-parser crate_root_parser --quiet
cargo test --package ash-parser parse_visibility --quiet
```

### Step 6: Commit

```bash
git add docs/spec/ crates/ash-parser/
git commit -m "feat(modules): TASK-337 - Add crate root and dependency syntax

- Parse crate root metadata and dependency declarations
- Add AST support for crate metadata
- Document external crate import syntax in specs"
```

### Step 7: Codex Verification

Spawn codex sub-agent to verify:
- parser tests cover valid and invalid crate root metadata
- specs and parser examples use the same syntax

---

## TASK-338: Extend Module Graph with Crate Identity

**Objective:** Make `ModuleGraph` represent multiple crates and declared dependency aliases explicitly.

**Background:** `ModuleGraph` currently tracks only modules, children, imports, and one root module. It cannot answer “which crate owns this module?” or “is this alias a declared dependency?”

**Files to Modify:**
- `crates/ash-core/src/module_graph.rs`

### Step 1: Write failing graph tests

Cover:
- adding crate records
- associating modules with a crate
- looking up the crate for a module
- resolving a dependency alias from crate A to crate B

### Step 2: Add crate-aware graph types

Expected structures:

```rust
pub struct CrateId(pub usize);

pub struct CrateInfo {
    pub name: String,
    pub root_module: ModuleId,
    pub root_path: String,
    pub dependencies: HashMap<String, CrateId>,
}
```

Add crate ownership to `ModuleNode` or equivalent graph metadata.

### Step 3: Add minimal helpers

Helpers should answer:
- `crate_id_for_module(module_id)`
- `crate_name(crate_id)`
- `root_module_for_crate(crate_id)`
- `dependency_target(crate_id, alias)`

### Step 4: Verify

```bash
cargo test --package ash-core module_graph --quiet
cargo clippy --package ash-core -- -D warnings
```

### Step 5: Commit

```bash
git add crates/ash-core/src/module_graph.rs
git commit -m "feat(modules): TASK-338 - Add crate-aware module graph metadata

- Introduce crate identity and dependency alias records
- Associate modules with owning crates
- Add graph helpers for cross-crate boundary checks"
```

### Step 6: Codex Verification

Spawn codex sub-agent to verify:
- crate-aware helpers are covered by focused unit tests
- existing single-crate graph behavior still passes

---

## TASK-339: Implement Dependency-Aware Multi-Crate Loading

**Objective:** Extend `ModuleResolver` to parse crate root metadata and recursively load declared dependency crates into one crate-aware graph.

**Background:** `resolver.rs` currently resolves one crate root and uses ad hoc `mod` scanning. Phase 55 needs real crate metadata and dependency traversal without CLI support.

**Files to Modify:**
- `crates/ash-parser/src/resolver.rs`
- `crates/ash-parser/src/parse_crate_root.rs`
- `crates/ash-parser/tests/multi_crate_resolver.rs` (new)

### Step 1: Write failing resolver tests with `MockFs`

Cover:
- entry crate with one dependency crate
- two dependencies from one root
- duplicate dependency alias rejection
- duplicate crate name rejection
- dependency cycle detection
- missing dependency root file error

### Step 2: Parse crate root metadata before resolving dependencies

Expected control flow:

```rust
let metadata = parse_crate_root_metadata(&content, canonical_path)?;
let crate_id = graph.add_crate(metadata.crate_name, canonical_path, module_id);
```

### Step 3: Resolve declared dependency crates recursively

Each dependency declaration should:
- load the referenced root file
- register the dependency crate
- link the alias from the importing crate to the dependency crate

### Step 4: Preserve existing in-crate `mod` behavior

Do not regress:
- `foo.ash`
- `foo/mod.ash`
- circular module detection inside a crate

### Step 5: Verify

```bash
cargo test --package ash-parser resolver --quiet
cargo test --package ash-parser multi_crate_resolver --quiet
```

### Step 6: Commit

```bash
git add crates/ash-parser/
git commit -m "feat(modules): TASK-339 - Load dependency crates from root metadata

- Parse crate root metadata during resolution
- Resolve dependency crates recursively with MockFs coverage
- Detect duplicate aliases, duplicate crates, and dependency cycles"
```

### Step 7: Codex Verification

Spawn codex sub-agent to verify:
- dependency loading tests exercise multi-crate graphs
- single-crate resolver tests remain green

---

## TASK-340: Resolve External Imports and Enforce Cross-Crate Visibility

**Objective:** Teach `ImportResolver` to resolve `external::<alias>::...` imports and enforce real cross-crate visibility boundaries.

**Background:** The import resolver only supports `crate::...` today. Real crate boundaries are not enforced until external imports are both resolvable and visibility-checked.

**Files to Modify:**
- `crates/ash-parser/src/import_resolver.rs`
- `crates/ash-parser/src/parse_use.rs`

### Step 1: Write failing import resolution tests

Cover:
- `use external::util::item;` resolves when `util` is declared
- undeclared external alias fails
- external import of `pub` item succeeds
- external import of `pub(crate)` fails
- external import of `pub(super)` fails
- external glob import only includes `pub` items

### Step 2: Accept explicit external import prefixes

Expected path handling:

```rust
match first_segment {
    "crate" => self.resolve_current_crate_path(importing_module, rest)?,
    "external" => self.resolve_external_path(importing_module, rest)?,
    _ => return Err(ImportError::InvalidPrefix { prefix: first_segment.into() }),
}
```

### Step 3: Enforce cross-crate visibility

Expected rule:

```rust
if self.module_graph.crate_id_for_module(importing_module)
    != self.module_graph.crate_id_for_module(target_module)
{
    return matches!(visibility, Visibility::Public);
}
```

### Step 4: Document parser behavior with explicit tests

Add `parse_use.rs` tests for:
- `use external::util::item;`
- `use external::util::{a, b as c};`

### Step 5: Verify

```bash
cargo test --package ash-parser import_resolver --quiet
cargo test --package ash-parser parse_use --quiet
```

### Step 6: Commit

```bash
git add crates/ash-parser/
git commit -m "fix(imports): TASK-340 - Resolve external imports and enforce crate boundaries

- Add external crate path resolution
- Reject non-public cross-crate imports
- Extend import parser coverage for external prefixes"
```

### Step 7: Codex Verification

Spawn codex sub-agent to verify:
- undeclared external aliases fail deterministically
- only `pub` crosses crate boundaries

---

## TASK-341: Align Type Checker and Add Multi-Crate Regression Coverage

**Objective:** Make type-checker visibility semantics explicit for external crate paths and add parser/type-checker regression tests for multi-crate scenarios.

**Background:** `ash-typeck/src/visibility.rs` still uses string-based heuristics. Phase 55 needs explicit external-path coverage so import resolver and type checker stay aligned.

**Files to Modify:**
- `crates/ash-typeck/src/visibility.rs`
- `crates/ash-typeck/tests/visibility_test.rs`
- `crates/ash-parser/tests/multi_crate_visibility.rs` (new)

### Step 1: Write failing visibility tests

Cover:
- `pub(crate)` denied from `external::<alias>::...`
- `pub` allowed from `external::<alias>::...`
- `pub(super)` denied externally
- `pub(in crate::internal)` denied externally
- explicit parsing/handling of `external::<alias>::module`

### Step 2: Replace loose external heuristics with explicit path semantics

Expected direction:

```rust
enum CrateRef {
    Current,
    External(String),
}
```

Whether implemented as a new enum or equivalent parsed representation, the checker should stop depending on “starts with external” as a loose shortcut.

### Step 3: Add integration-style regression tests

Use realistic multi-crate source fixtures or in-memory test inputs to prove:
- dependency crate imports type-check only when exported publicly
- same source shape used by the loader is covered in tests

### Step 4: Verify

```bash
cargo test --package ash-typeck visibility --quiet
cargo test --package ash-parser multi_crate_visibility --quiet
```

### Step 5: Commit

```bash
git add crates/ash-typeck/ crates/ash-parser/tests/
git commit -m "test(visibility): TASK-341 - Align type checker with cross-crate visibility

- Add explicit external crate visibility coverage
- Remove loose external-path heuristics
- Add multi-crate parser/type-checker regressions"
```

### Step 6: Codex Verification

Spawn codex sub-agent to verify:
- import resolver and type checker agree on cross-crate visibility expectations
- regression tests cover both success and rejection paths

---

## TASK-342: Phase 55 Closeout

**Objective:** Final verification and documentation of Phase 55 completion.

**Prerequisites:**
- TASK-337 complete
- TASK-338 complete
- TASK-339 complete
- TASK-340 complete
- TASK-341 complete

### Step 1: Run focused verification

```bash
cargo test --package ash-core module_graph --quiet
cargo test --package ash-parser resolver --quiet
cargo test --package ash-parser import_resolver --quiet
cargo test --package ash-typeck visibility --quiet
```

### Step 2: Run workspace quality gates

```bash
cargo test --workspace --quiet
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
```

### Step 3: Update plan tracking

- Add Phase 55 status to `docs/plan/PLAN-INDEX.md`
- Update follow-up references from Phase 54 if needed

### Step 4: Commit

```bash
git add docs/plan/ docs/spec/ crates/
git commit -m "docs(plan): TASK-342 - Close out Phase 55 cross-crate boundary work

- Cross-crate loading and visibility enforcement verified
- Specs, parser, resolver, and type checker aligned
- Workspace quality gates pass"
```

### Step 5: Codex Verification

Spawn codex sub-agent to verify:
- focused cross-crate checks pass
- workspace gates pass
- plan index reflects Phase 55 accurately

---

## Success Criteria

Phase 55 is complete when:

- [ ] Ash crate roots can declare `crate <name>;`
- [ ] Ash crate roots can declare `dependency <alias> from "<path>";`
- [ ] `ModuleResolver` loads declared dependency crates recursively
- [ ] `ModuleGraph` can answer crate-ownership and dependency-alias questions
- [ ] `ImportResolver` resolves `external::<alias>::...`
- [ ] Only `pub` items are importable across crates
- [ ] Type checker and import resolver agree on cross-crate visibility
- [ ] Workspace tests, clippy, fmt, and docs are clean

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Crate-aware graph changes break single-crate code paths | Keep single-crate tests green in every task |
| Resolver metadata parsing drifts from spec examples | Add spec and parser tests in the same task |
| External imports reintroduce relative-path ambiguity | Use explicit `external::<alias>::...` syntax |
| Type checker and import resolver diverge | Add explicit parity tests in TASK-341 |
