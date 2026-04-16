# PLAN-090: Spec Processor Implementation Plan

> **For Hermes:** Use `subagent-driven-development` and `ash-phase-implementation` skills to execute this plan task-by-task.

**Goal:** Build a canonical Ash workflow (the Spec Processor) that audits the Ash repository for spec drift, example conformance, PLAN-INDEX coherence, and changelog completeness.

**Architecture:** Three parallel tracks: (A) pure-string processor core, (B) stdlib substrates, (C) integration and meta-validation. All decision gates (D1–D4) are resolved; no architectural blockers remain.

**Tech Stack:** Ash workflow language, `ash-engine`/`ash-interp` runtime, Rust stdlib backends, `cargo test`, `proptest`.

---

## Phase Overview

| Track | Tasks | Deliverable |
|-------|-------|-------------|
| A | TASK-590 – TASK-594 | Pure-string processor core: file traversal, index coherence, link validation, changelog checks, report formatting |
| B | TASK-595 – TASK-598 | Stdlib substrates: `regex`, `markdown`, `json`, `process` |
| C | TASK-600 – TASK-603 | Integration: example conformance, capability boundary audit, meta-validation, CI gate |

**Deferred to future phase:** `std::diff` (TASK-599) — not required for MVP.

---

## Track A: Pure-String Processor Core

These tasks require only currently implemented Ash language features (file I/O, strings, records, pattern matching). They can begin immediately and in parallel with Track B.

---

### TASK-590: File Collector and Repository Traversal

**Objective:** Implement the repository file discovery stage: gather all spec, plan, example, and changelog files.

**Files:**
- Create: `apps/spec_processor/src/collect.ash`
- Create: `apps/spec_processor/src/types.ash`
- Test: `apps/spec_processor/tests/test_collect.ash` (or Rust integration test if Ash test runner is not ready)

**Step 1: Write failing test**

```ash
// test: given a mock directory tree, collect returns all .md and .ash files
let result = collect::scan_tree("tests/fixtures/repo_a");
assert_true(result.spec_files.length > 0);
assert_true(result.example_files.length > 0);
```

Run: `cargo test --package ash-cli --test spec_processor_collect -v` (or equivalent Rust harness)
Expected: FAIL — "module collect not found"

**Step 2: Write minimal implementation**

```ash
// apps/spec_processor/src/collect.ash

pub record FileTree {
    spec_files: List<String>,
    plan_files: List<String>,
    example_files: List<String>,
    changelog_files: List<String>,
}

pub fn scan_tree(root: String) -> FileTree {
    // Use std::io::dir::read_dir_recursive or equivalent
    let all = dir::read_dir_all(root);
    FileTree {
        spec_files: filter(all, fn p { path::file_name(p).starts_with("SPEC-") && p.ends_with(".md") }),
        plan_files: filter(all, fn p { path::file_name(p).starts_with("PLAN-") && p.ends_with(".md") }),
        example_files: filter(all, fn p { p.ends_with(".ash") }),
        changelog_files: filter(all, fn p { path::file_name(p) == "CHANGELOG.md" }),
    }
}
```

**Note:** If `dir::read_dir_all` does not exist, use `read_dir` with explicit recursion.

**Step 3: Run test to verify pass**

Run: `cargo test --package ash-cli --test spec_processor_collect -v`
Expected: PASS

**Step 4: Commit**

```bash
git add apps/spec_processor/
git commit -m "feat(spec-processor): add file collector and traversal (TASK-590)"
```

**Step 5: Codex Verification**

Spawn codex to verify the module compiles, tests pass, and file paths match the plan.

---

### TASK-591: PLAN-INDEX Parser and Coherence Checker

**Objective:** Parse `docs/plan/PLAN-INDEX.md` and detect: missing task files, orphaned tasks, and status inconsistencies.

**Files:**
- Create: `apps/spec_processor/src/plan_index.ash`
- Create: `apps/spec_processor/tests/fixtures/PLAN-INDEX-mock.md`
- Test: `apps/spec_processor/tests/test_plan_index.ash`

**Step 1: Write failing test**

```ash
let findings = plan_index::check("tests/fixtures/PLAN-INDEX-mock.md");
assert_true(findings.any(fn f { f.category == IndexIncoherence }));
```

**Step 2: Implement**

- Parse task references using simple string search (regex not required yet; use `string::contains` and `string::split`).
- For each `TASK-NNN` found, verify `docs/plan/tasks/TASK-NNN-*.md` exists.
- Detect orphaned task files (task file exists but not referenced in index).

**Step 3: Verify and commit**

Run tests, commit with conventional message.

**Step 4: Codex Verification**

Verify parsing logic handles at least 3 real sections from `docs/plan/PLAN-INDEX.md` without false positives.

---

### TASK-592: Spec Cross-Reference Validator

**Objective:** Validate internal links in `docs/spec/SPEC-*.md` files and detect broken cross-references.

**Files:**
- Create: `apps/spec_processor/src/spec_links.ash`
- Test: `apps/spec_processor/tests/test_spec_links.ash`

**Step 1: Write failing test**

```ash
let findings = spec_links::check_files(["tests/fixtures/SPEC-broken.md"]);
assert_true(findings.length > 0);
```

**Step 2: Implement**

- Extract Markdown link syntax: `[text](target.md)` and `[text](target.md#anchor)`.
- Verify that `target.md` exists in `docs/spec/` or `docs/plan/`.
- Do not verify anchor existence in MVP (optional future enhancement).

**Step 3: Verify and commit**

**Step 4: Codex Verification**

Run against all `docs/spec/*.md` files in the real repo. Report count of broken links found.

---

### TASK-593: Changelog Completeness Checker

**Objective:** Compare `PLAN-INDEX.md` tasks marked “✅ Complete” against `CHANGELOG.md` entries and flag missing changelog coverage.

**Files:**
- Create: `apps/spec_processor/src/changelog.ash`
- Test: `apps/spec_processor/tests/test_changelog.ash`

**Step 1: Write failing test**

```ash
let mock_index = "..."; // contains TASK-001 as Complete
let mock_changelog = "..."; // missing TASK-001
let findings = changelog::check(mock_index, mock_changelog);
assert_true(findings.any(fn f { f.description.contains("TASK-001") }));
```

**Step 2: Implement**

- Extract completed task IDs from `PLAN-INDEX.md` (lines containing ✅ and a `TASK-NNN` reference).
- Verify each completed task ID appears at least once in `CHANGELOG.md`.
- Emit `ChangelogMissing` findings.

**Step 3: Verify and commit**

**Step 4: Codex Verification**

Run against the real repo. The result should match known gaps (if any). Zero false positives required.

---

### TASK-594: Report Formatter

**Objective:** Aggregate findings into human-readable and JSON output formats, with a non-zero exit code when blocked.

**Files:**
- Create: `apps/spec_processor/src/report.ash`
- Test: `apps/spec_processor/tests/test_report.ash`

**Step 1: Write failing test**

```ash
let report = report::format_json([finding_t2]);
assert_true(report.contains("\"tier\": 2"));
assert_true(report.blocked);
```

**Step 2: Implement**

```ash
pub record Report {
    findings: List<SpecFinding>,
    blocked: Bool,
    tier_0_count: Int,
    tier_1_count: Int,
    tier_2_count: Int,
}

pub fn format_human(r: Report) -> String;
pub fn format_json(r: Report) -> String;
```

For MVP, `format_json` can use ad-hoc string construction (pending TASK-597 `std::json`). Once TASK-597 is complete, refactor to use `json::stringify`.

**Step 3: Verify and commit**

**Step 4: Codex Verification**

Verify JSON output is syntactically valid and `blocked` is true iff at least one Tier 2 finding exists.

---

## Track B: Stdlib Substrates

These tasks create the missing stdlib modules. They run in parallel with Track A and with each other.

---

### TASK-595: std::regex Interface and Rust Backend

**Objective:** Add `std::regex` with `match`, `find`, and `replace` functions, backed by the `regex` Rust crate.

**Files:**
- Create: `std/src/regex.ash`
- Modify: `std/src/lib.ash` (add `pub use regex::*`)
- Modify: `crates/ash-engine/src/providers/mod.rs` or new provider module
- Test: `crates/ash-engine/tests/regex_capability.rs`

**Step 1: Define Ash interface**

```ash
pub fn find(pattern: String, text: String) -> Option<String>;
pub fn matches(pattern: String, text: String) -> Bool;
pub fn replace(pattern: String, replacement: String, text: String) -> String;
```

**Step 2: Write failing test**

Rust integration test that calls the capability through the engine.

**Step 3: Implement Rust backend**

- Register a built-in `regex` capability provider in the engine.
- Map `find`/`matches`/`replace` to `regex::Regex` operations.
- Return `RuntimeError` on invalid pattern.

**Step 4: Verify and commit**

**Step 5: Codex Verification**

Verify real non-test callsites exist (the spec processor uses `regex` in TASK-591/TASK-592).

---

### TASK-596: std::markdown CommonMark AST MVP

**Objective:** Implement a CommonMark-compliant Markdown AST with Pandoc JSON filter compatibility, backed by `pulldown-cmark` (or equivalent) in Rust.

**Files:**
- Create: `std/src/markdown.ash`
- Modify: `std/src/lib.ash`
- Create: `crates/ash-engine/src/providers/markdown.rs`
- Test: `crates/ash-engine/tests/markdown_capability.rs`

**Step 1: Define Ash AST**

```ash
pub enum Block {
    Paragraph(List<Inline>),
    Heading(Int, List<Inline>),
    CodeBlock(Option<String>, String),
    BlockQuote(List<Block>),
    List(Bool, List<ListItem>),
    Extension(ExtensionBlock),
}

pub enum Inline {
    Text(String),
    Code(String),
    Emphasis(List<Inline>),
    Strong(List<Inline>),
    Link(String, Option<String>, List<Inline>),
    Image(String, Option<String>, List<Inline>),
    Extension(ExtensionInline),
}

pub record MarkdownDoc {
    blocks: List<Block>,
}

pub enum ExtensionBlock { /* architecture-only */ }
pub enum ExtensionInline { /* architecture-only */ }
```

**Step 2: Implement Rust-backed parser**

- `pub fn parse(text: String) -> Result<MarkdownDoc, MarkdownError>`
- Uses `pulldown-cmark` to parse, then constructs the Ash AST via the engine's value system.

**Step 3: Implement Pandoc JSON filter helper**

```ash
pub fn to_pandoc_json(doc: MarkdownDoc) -> json::JsonValue;
```

This function is pure Ash, constructing `JsonValue::Object`/`Array` nodes. It is implemented after TASK-597 (`std::json`) is available, or stubbed with string construction until then.

**Step 4: Verify and commit**

**Step 5: Codex Verification**

Verify that parsing a real `SPEC-*.md` file produces a non-empty `MarkdownDoc` and that `to_pandoc_json` round-trips through `json::stringify`.

---

### TASK-597: std::json Hybrid Interface

**Objective:** Implement `std::json` with Rust-backed `parse`/`stringify` and a pure-Ash `JsonValue` AST.

**Files:**
- Create: `std/src/json.ash`
- Modify: `std/src/lib.ash`
- Create: `crates/ash-engine/src/providers/json.rs`
- Test: `crates/ash-engine/tests/json_capability.rs`

**Step 1: Define Ash interface**

```ash
pub enum JsonValue {
    Null,
    Bool(Bool),
    Number(Float),
    String(String),
    Array(List<JsonValue>),
    Object(Map<String, JsonValue>),
}

pub fn parse(text: String) -> Result<JsonValue, JsonError>;
pub fn stringify(value: JsonValue) -> Result<String, JsonError>;
pub fn stringify_pretty(value: JsonValue) -> Result<String, JsonError>;

pub fn is_null(v: JsonValue) -> Bool;
pub fn as_string(v: JsonValue) -> Option<String>;
pub fn get(v: JsonValue, key: String) -> Option<JsonValue>;
pub fn get_index(v: JsonValue, index: Int) -> Option<JsonValue>;
```

**Step 2: Implement Rust backend**

- Use `serde_json` for parse/stringify.
- Map `serde_json::Value` to/from Ash `JsonValue` via the engine's value conversion layer.

**Step 3: Verify round-trip**

Property test: `parse(stringify(v)) == v` for a generated `JsonValue`.

**Step 4: Codex Verification**

Verify `JsonValue` shape matches the design note and that `stringify` produces valid JSON.

---

### TASK-598: std::process Interface and Rust Backend

**Objective:** Implement `std::process` as a built-in, auto-registered capability, per `DESIGN-NOTE-PROCESS-EFFECT.md`.

**Files:**
- Create: `std/src/process.ash`
- Modify: `std/src/lib.ash`
- Create: `crates/ash-engine/src/providers/process.rs`
- Modify: `crates/ash-engine/src/lib.rs` (add `with_process_capabilities()` to `EngineBuilder` default path)
- Test: `crates/ash-engine/tests/process_capability.rs`

**Step 1: Define Ash interface**

```ash
pub record ProcessOutput {
    status: Int,
    stdout: String,
    stderr: String,
}

pub record ProcessOptions {
    timeout_ms: Option<Int>,
    cwd: Option<String>,
}

pub fn run(cmd: String, args: List<String>) -> Result<ProcessOutput, ProcessError>;
pub fn run_with_options(cmd: String, args: List<String>, options: ProcessOptions) -> Result<ProcessOutput, ProcessError>;
```

**Step 2: Implement Rust backend**

- Register `Process` capability provider with `Operational` effect.
- Use `tokio::process::Command` for async subprocess execution.
- Enforce timeout (default 30s, override via `ProcessOptions`).
- Capture stdout/stderr.

**Step 3: Auto-register in default engine**

```rust
// In EngineBuilder::build() or a default builder helper
builder.with_process_capabilities();
```

**Step 4: Verify and commit**

**Step 5: Codex Verification**

Verify that an Ash workflow can call `process::run("echo", ["hello"])` and receive `"hello\n"` in stdout.

---

### [PLACEHOLDER] TASK-599: std::diff Line-Diff Utility

**Status:** Deferred. Not required for spec processor MVP.

---

## Track C: Integration and Meta-Validation

These tasks run after Tracks A and B are complete. They wire the processor together and enforce the CI gate.

---

### TASK-600: Example Syntax Conformance (ash check Integration)

**Objective:** Integrate `std::process` to run `ash check` on every `.ash` example file and aggregate parse/type errors.

**Files:**
- Create: `apps/spec_processor/src/examples.ash`
- Modify: `apps/spec_processor/src/main.ash`
- Test: `apps/spec_processor/tests/test_examples.ash`

**Step 1: Implement**

```ash
pub fn check_examples(paths: List<String>) -> List<SpecFinding> {
    for path in paths {
        let output = process::run("ash", ["check", path]);
        if output.status != 0 {
            findings.push(SpecFinding {
                category: ExampleFailure,
                description: output.stderr,
                // ...
            })
        }
    }
}
```

**Step 2: Verify against real repo**

Run the processor against `examples/`. Every `.ash` file should either pass or produce a valid `ExampleFailure` finding.

**Step 3: Codex Verification**

Verify that the subprocess integration does not deadlock on large stdout/stderr and that timeout is handled gracefully.

---

### TASK-601: Capability Boundary Audit

**Objective:** Implement the `capability_boundary.ash` mechanism and the audit rule that flips flags when substrates are verified.

**Files:**
- Create: `apps/spec_processor/src/capability_boundary.ash`
- Create: `apps/spec_processor/capability_boundary.ash`
- Test: `apps/spec_processor/tests/test_boundary.ash`

**Step 1: Implement**

```ash
// apps/spec_processor/capability_boundary.ash
let expected_capabilities = {
    file_io: true,
    process_spawn: true,  // now available after TASK-598
    regex_matching: true, // now available after TASK-595
    markdown_parsing: true,
    json_parsing: true,
    first_class_functions: true,
    generic_interfaces: false, // still pending Phase 83
};
```

The boundary module reads this record and:
- Skips validations where the capability is `false`.
- Emits a `ToolingGap` finding if a `true` capability fails at runtime.

**Step 2: Codex Verification**

Verify that changing a flag from `true` to `false` causes the processor to skip the corresponding validation.

---

### TASK-602: Meta-Validation (Processor Audits Its Own Rules)

**Objective:** The processor validates that its own source files conform to the same rules it applies to the rest of the repo.

**Files:**
- Modify: `apps/spec_processor/src/main.ash`

**Step 1: Implement**

Add `apps/spec_processor/` to the scanned file tree. The processor should:
- Find its own `.ash` files and ensure they parse.
- Validate that `capability_boundary.ash` is well-formed.
- Verify that its own spec cross-references (e.g. to `DESIGN-SPEC-PROCESSOR.md`) are valid.

**Step 2: Codex Verification**

Introduce a deliberate broken link in the processor's own docs. Verify the processor flags it as a `SpecDrift` finding.

---

### TASK-603: End-to-End CI Gate Integration

**Objective:** Wire the processor into `cargo test` and the manual CI workflow so that a failing processor blocks the pipeline.

**Files:**
- Create: `tests/spec_processor_integration.rs`
- Modify: `.github/workflows/manual-ci.yml` (if present) or document in `docs/CI-PLAN.md`

**Step 1: Implement Rust integration test**

```rust
#[test]
fn spec_processor_passes() {
    let output = run_ash_workflow("apps/spec_processor/src/main.ash");
    assert!(output.status.success(), "Spec processor failed: {}", output.stderr);
}
```

**Step 2: Verify against real repo**

Run the integration test. It should produce the actual processor report for the current repo state. Review the report for unexpected findings.

**Step 3: Codex Phase Audit**

Spawn codex sub-agent for full phase audit:

```
goal: "Audit Phase 90 (Spec Processor) completion"
context: |
  Tasks: TASK-590 through TASK-603
  Verify:
  1. cargo test --workspace passes
  2. Processor runs end-to-end without panic
  3. Report output is valid JSON and human-readable
  4. No compiler warnings in new code
  5. CHANGELOG.md updated for Phase 90
```

---

## Phase Completion Criteria

Before marking Phase 90 complete, verify:

- [ ] All Track A tasks implemented and tested
- [ ] All Track B tasks implemented and tested (except deferred TASK-599)
- [ ] All Track C tasks implemented and passing
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `cargo doc --no-deps` clean
- [ ] `CHANGELOG.md` has an `[Unreleased]` entry for Phase 90
- [ ] `PLAN-INDEX.md` Phase 90 status updated to ✅ Complete
- [ ] Codex phase audit returns VERIFIED

---

## Gating Notes

- **Actionable now:** TASK-590, TASK-591, TASK-592, TASK-593, TASK-594 (Track A)
- **Blocked on Rust backend capacity:** TASK-595, TASK-596, TASK-597, TASK-598 (Track B)
- **Blocked on A + B completion:** TASK-600, TASK-601, TASK-602, TASK-603 (Track C)
- **Deferred indefinitely:** TASK-599 (`std::diff`)

## Downstream Impact

Phase 90 unblocks:
- Phase 83+ verification (processor can audit generic impl syntax once implemented)
- Case B (agent-pipeline loader) via `std::json` and `std::regex`
- Case E (dashboard backend) via `std::json` and `std::process`
- Case C (REPL kernel) via `std::markdown` and `std::json`
