# Implementation-Grade Review: SPEC-041, SPEC-042, SPEC-043

**Review Date:** 2025-04-15  
**Specs Reviewed:** SPEC-041 (Ash Lint Library), SPEC-042 (Ash Source Formatter), SPEC-043 (Incremental Analysis Engine)  
**Tasks Reviewed:** TASK-574, TASK-575, TASK-576  
**Plans Reviewed:** PLAN-033, PLAN-034, PLAN-035  
**Context:** These phases follow the LSP MVP (SPEC-038, Phase 87).

---

## Executive Summary

| Spec | Overall Feasibility | Severity | Verdict |
|------|---------------------|----------|---------|
| SPEC-041 | Feasible with adjustments | MEDIUM | Acceptable after spec updates |
| SPEC-042 | Feasible but blocked | MEDIUM | Acceptable, dependencies must land first |
| SPEC-043 | High risk / underspecified | HIGH | Requires significant architectural spiking before commitment |

All three specs share a **critical hidden dependency:** the Ash parser currently lacks a `parse_module(source) -> ModuleFile` top-level API. SPEC-041 and SPEC-042 both assume this function exists, and SPEC-043 assumes `ash-typeck` can type-check a `ModuleFile`—neither of which is true today. These gaps must be resolved in SPEC-039 (Parser Tooling) before any of the downstream specs can be implemented.

---

## 1. SPEC-041: Ash Lint Library Extraction

### 1.1 Lint API Appropriateness for LSP Integration

**Assessment:** The `lint_module(module: &ModuleFile, config: &LintConfig) -> Vec<LintDiagnostic>` API is the *right shape* for LSP use.

**Strengths:**
- Accepting a pre-parsed `&ModuleFile` avoids double-parsing inside the LSP diagnostic loop.
- `LintDiagnostic` carries `span: Span`, which maps cleanly to LSP `Range`.
- Severity enum maps 1:1 to LSP `DiagnosticSeverity`.

**Issues:**
1. **String-based `code`:** Using `code: String` is brittle for LSP consumers. Recommend `code: LintCode` (a thin newtype around `&'static str` or enum) so that `ash-lsp-core` can match on it for code actions / quick fixes later.
2. **No structured fix/suggestion field:** The current CLI diagnostic has `suggestion: Option<String>`. LSP code actions need `Vec<TextEdit>` or at least a replacement span. Recommend adding `pub fixes: Vec<LintFix>` to `LintDiagnostic` now, even if MVP fixes are simple string replacements.
3. **Missing `related_information`:** Some lints (e.g., cross-definition policy conflicts) will eventually need related spans. This can be deferred, but the struct should reserve room for it.

**Severity:** MEDIUM  
**Recommendation:** Update `LintDiagnostic` to include a `LintFix` sub-type before implementation begins, and define `LintCode` as a non-string type.

### 1.2 Current State vs. Target State

The spec accurately describes `crates/ash-lint` as a CLI-only binary with trivial string-matching lints. Converting it to a dual crate is mechanically straightforward.

**Risk:** The existing lints are so primitive that the refactor is effectively a rewrite. The 12-hour estimate is reasonable for 4 simple AST visitors *if* the parser surface API is stable.

### 1.3 Hidden Blocker: `parse_module` Does Not Exist

SPEC-041 assumes:
```rust
let module = match ash_parser::parse_module(source) { ... };
```

**Fact:** The Ash parser has **no** public `parse_module` or `parse_file` function returning `ModuleFile`. There is a `surface::ModuleFile` struct, but no top-level parser entry point that produces it.

**Severity:** HIGH  
**Recommendation:** Add "Implement `ash_parser::parse_module(source: &str) -> Result<ModuleFile, Vec<ParseError>>`" as an explicit deliverable of SPEC-039 before SPEC-041 begins.

---

## 2. SPEC-042: Ash Source Formatter

### 2.1 Dependency on SPEC-039

The formatter explicitly requires SPEC-039's `CommentTable`. This dependency is correctly identified in the spec.

**However**, the current `ModuleFile` struct in `crates/ash-parser/src/surface.rs` does **not** have a `comments` field. SPEC-039 proposes adding:
```rust
pub struct ModuleFile {
    pub definitions: Vec<Definition>,
    pub module_decls: Vec<ModuleDecl>,
    pub workflow: Option<WorkflowDef>,
    pub span: Span,
    // comments field is MISSING
}
```

SPEC-039 must land first, including:
1. `Comment` token kind in the lexer.
2. `CommentTable` side-table implementation.
3. `comments: CommentTable` field on `ModuleFile`.
4. Binding spans (`Expr::Variable(Name, Span)`, `Pattern::Variable(Name, Span)`).

**Severity:** MEDIUM  
**Recommendation:** Update SPEC-042's "Blocked by" list to explicitly call out the `ModuleFile.comments` field and the `parse_module` entry point.

### 2.2 Does the Formatter Design Depend on Anything Beyond SPEC-039?

No. The formatter design in SPEC-042 is self-contained given SPEC-039. It does not require ash-typeck, ash-lint, or the LSP core. The `Formatter` struct only needs `ModuleFile` + `CommentTable`.

**Minor concern:** The `format_module` function in the spec accesses `module.comments` directly:
```rust
let mut fmt = Formatter::new(&module.comments, indent_width);
```
This is fine, but it means `ModuleFile` must own the `CommentTable` (not borrow it). SPEC-039 should make this clear.

### 2.3 Implementation Realism

- **40-hour estimate:** Reasonable for a basic pretty-printer, but comment placement is notoriously finicky. Expect 10–15 hours of edge-case triage (comments between record fields, end-of-line trailing comments, blank-line heuristics).
- **Round-trip testing:** The spec proposes `parse(format(parse(source))) == parse(source)`. This is the right invariant, but it requires parser equality to be stable. Any non-determinism in `HashMap` iteration order inside `ModuleFile` will break this. Ensure `surface.rs` types derive `PartialEq` in a deterministic way.

---

## 3. SPEC-043: Incremental Analysis Engine (Salsa)

### 3.1 Salsa Integration: Well-Scoped or Over-Ambitious?

**Assessment:** The *goal* is correct, but the spec is **underspecified** and **assumes APIs that do not exist**.

### 3.2 Missing APIs in `ash-typeck`

SPEC-043 defines this tracked query:
```rust
#[salsa::tracked]
pub fn type_check_file(db: &dyn AshDb, path: FilePath) -> (TypeCheckResult, Vec<ConstructorError>) {
    let module = parse_file(db, path).0;
    let graph = module_graph(db, workspace_root(path));
    // run ash-typeck
}
```

**Problem:** `ash-typeck` currently has **no function that takes a `ModuleFile`**.

The existing public type-check entry points in `crates/ash-typeck/src/lib.rs` are:
- `type_check_program(program: &Program) -> Result<TypeCheckResult, TypeCheckError>`
- `type_check_workflow_def(workflow: &WorkflowDef) -> Result<TypeCheckResult, TypeCheckError>`
- `type_check_workflow(workflow: &Workflow, param_bindings) -> Result<TypeCheckResult, TypeCheckError>`

There is no `type_check_module_file`, no `check_module`, and no `type_check_file`.

**Severity:** HIGH  
**Recommendation:** Before SPEC-043 can be implemented, `ash-typeck` must expose a `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph) -> (TypeCheckResult, Vec<ConstructorError>)` API. This is a non-trivial piece of engineering that should be its own task (or part of SPEC-038 Phase 2).

### 3.3 Salsa Trait Requirements vs. Current `ash-typeck` Architecture

Salsa 0.26 requires all tracked function arguments and return types to be:
- `'static`
- `Clone`
- `Eq`
- `Hash`
- `Debug`

**Current types that likely FAIL these requirements:**

1. **`TypeCheckResult`** (in `ash-typeck/src/lib.rs`):
   ```rust
   pub struct TypeCheckResult {
       pub substitution: Substitution,
       pub errors: Vec<TypeError>,
       pub inferred_types: HashMap<String, Type>,
       pub effect: ash_core::Effect,
       pub obligation_status: ObligationCheckResult,
   }
   ```
   `Substitution`, `TypeError`, `Type`, and `ObligationCheckResult` must all implement `Eq + Hash`. Today they derive `Debug, Clone` but not necessarily `Eq` or `Hash`.

2. **`ModuleGraph`** (from `ash_core::module_graph`):
   Contains `HashMap<ModuleId, ModuleNode>` and likely paths / file system data. Needs audit for `Eq + Hash`.

3. **`ConstructorError` / `TypeEnvError` / `ExhaustivenessError`**:
   `TypeEnvError` and `ExhaustivenessError` currently do **not** implement `Hash`. Some variants carry `Type`, which itself may not implement `Hash`.

**Severity:** HIGH  
**Recommendation:** Add a **spike task** (8–12 hours) to:
- Audit `ash-typeck` and `ash_core` types for Salsa compatibility.
- Add missing `Eq + Hash` derives (or wrap types in salsa-compatible newtypes).
- Verify that `TypeCheckResult` can be returned from a `#[salsa::tracked]` function without orphan-rule issues.

### 3.4 `ash-lsp-core` Does Not Exist Yet

SPEC-043 wants to replace the cache in `ash-lsp-core`. As of today, there is **no `crates/ash-lsp-core`** directory in the workspace. It is defined in SPEC-038 (LSP MVP) as a new crate to be created in Phase 87.

**Severity:** MEDIUM  
**Recommendation:** SPEC-043 should not start until SPEC-038 Phase 2 (Diagnostics & Symbols) is complete and the `ash-lsp-core` API has stabilized. The spec already says "Blocked by: SPEC-038 LSP MVP"; this should be elevated to a **hard blocker**.

### 3.5 VFS / Salsa Integration Sketch

The `on_did_change` snippet in SPEC-043 has a conceptual bug:
```rust
let file_path = FilePath::new(&mut self.db, path);
let source_file = SourceFile::new(&mut self.db, new_text);
self.db.set_source_file(file_path, source_file);
```

Salsa inputs should be created once and then mutated via setters. Creating a *new* `FilePath` input on every `didChange` changes the identity of the input, which invalidates **all** queries keyed by that path. The correct pattern is:
1. Store `FilePath` salsa inputs in a `HashMap<Url, FilePath>` when the file is first opened.
2. On `didChange`, call `source_file.set_text(&mut self.db, new_text)`.

**Severity:** MEDIUM  
**Recommendation:** Update the VFS integration snippet to show input mutation, not input re-creation.

### 3.6 Estimate Realism

32 hours for:
- Adding Salsa dependency
- Defining database + 4 tracked queries
- VFS integration
- Migration strategy (side-by-side testing)
- Performance validation

**Verdict:** Optimistic. For a codebase with no existing Salsa footprint and no clean query boundaries, 32h is more like a **best-case** estimate. A realistic estimate is **40–48 hours**, especially given the type-system compatibility work described above.

---

## 4. Timeline and Dependency Chain Realism

### 4.1 Phase Ordering

| Phase | Spec / Task | Status |
|-------|-------------|--------|
| 86 | SPEC-041 / TASK-574 (Lint Library) | Planned |
| 87 | SPEC-038 (LSP MVP) | Draft, not yet implemented |
| 88 | SPEC-042 / TASK-575 (Formatter) | Planned |
| 89 | SPEC-043 / TASK-576 (Salsa) | Planned |

### 4.2 Hidden Dependencies

```
SPEC-039 (Parser Tooling)
  ├─ Must provide: parse_module() -> ModuleFile
  ├─ Must provide: ModuleFile.comments: CommentTable
  └─ Must provide: Expr::Variable(Name, Span), Pattern::Variable(Name, Span)
       │
       ├─ Blocks SPEC-041 (lint_module needs parse_module)
       ├─ Blocks SPEC-042 (formatter needs CommentTable)
       └─ Blocks SPEC-043 (Salsa parse_file query needs parse_module)

SPEC-038 (LSP MVP)
  ├─ Must create: crates/ash-lsp-core
  ├─ Must expose: check_file(module, graph) API
  └─ Must stabilize: Diagnostic aggregator / VFS
       │
       └─ Blocks SPEC-043 (Salsa needs something to replace)

ash-typeck internal refactor
  ├─ Must expose: type_check_module_file() API
  └─ Must implement: Eq + Hash on TypeCheckResult, errors, types
       │
       └─ Blocks SPEC-043
```

### 4.3 Estimate Roll-Up

| Work Package | Spec Estimate | Realistic Estimate | Risk |
|--------------|---------------|--------------------|------|
| Lint Library | 12h | 12–16h | Low-Medium |
| Source Formatter | 40h | 40–56h | Medium |
| Salsa Integration | 32h | 40–56h | High |
| **Parser API gaps** | *Not budgeted* | **8–12h** | High |
| **Type-checker API gaps** | *Not budgeted* | **12–20h** | High |
| **Salsa compatibility spike** | *Not budgeted* | **8–12h** | High |

**Bottom line:** The combined timeline for Phases 86–89 is realistically **6–7 weeks**, not the 4.5 weeks implied by the current estimates.

---

## 5. Actionable Recommendations

### Immediate (Before any implementation begins)

1. **SPEC-039:** Add explicit deliverables for:
   - `pub fn parse_module(source: &str) -> Result<ModuleFile, Vec<ParseError>>`
   - `pub comments: CommentTable` field on `ModuleFile`
   - `#[derive(Hash, Eq)]` on `Span`, `Name`, and all `surface.rs` types that will cross Salsa boundaries.

2. **SPEC-041:** Update `LintDiagnostic` to:
   ```rust
   pub code: LintCode,  // not String
   pub fixes: Vec<LintFix>,
   ```
   Define `LintFix { span: Span, replacement: String, description: String }`.

3. **SPEC-038:** Add an explicit task to expose `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph)` in `ash-typeck` before the Salsa phase.

### Before SPEC-043 is approved

4. **Spike TASK-SALSA-COMPAT (8–12h):**
   - Branch off main.
   - Add `salsa = "0.26"` to a scratch crate.
   - Try to define `#[salsa::tracked] fn type_check_file(...) -> TypeCheckResult`.
   - Record every missing `Eq`/`Hash`/`Clone` derive.
   - Report findings. Use this to revise SPEC-043 and TASK-576 estimate.

5. **Fix SPEC-043 VFS snippet:** Show input reuse + `set_text`, not `FilePath::new` on every change.

### Medium-term

6. **Combine SPEC-043 with SPEC-038 Phase 3:** Consider making Salsa the *default* cache for `ash-lsp-core` from the start, rather than building an LRU cache in SPEC-038 and then throwing it away in SPEC-043. This saves ~1 week of throwaway work. If Salsa proves too risky, the spike in recommendation #4 will surface that early.

---

## 6. Severity Ratings Summary

| Issue | Location | Severity | Action Owner |
|-------|----------|----------|--------------|
| `parse_module()` API missing | SPEC-039 / SPEC-041 / SPEC-042 / SPEC-043 | HIGH | Update SPEC-039 |
| `type_check_module_file()` API missing | SPEC-043, ash-typeck | HIGH | Add to SPEC-038 or new task |
| Salsa type-compatibility unverified | SPEC-043, ash-typeck | HIGH | Spike before commit |
| `LintDiagnostic` uses `String` for code | SPEC-041 | MEDIUM | Update spec |
| `LintDiagnostic` lacks fix/suggestion struct | SPEC-041 | MEDIUM | Update spec |
| VFS snippet recreates salsa inputs | SPEC-043 | MEDIUM | Update spec |
| `ash-lsp-core` does not exist yet | SPEC-043 | MEDIUM | Wait for SPEC-038 |
| Formatter estimate too optimistic | TASK-575 | MEDIUM | Increase to 48–56h |
| Salsa estimate too optimistic | TASK-576 | HIGH | Increase to 40–56h |

---

*Review completed. No code changes made; this is a specification review only.*
