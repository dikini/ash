# Phase 53: Post-Review Remediation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Address remaining contract gaps, clippy warnings, and spec inconsistencies identified in the post-Phase 52 review.

**Source:** Review findings from Honcho memory - outdated specs, clippy warnings, syntax inconsistencies

**Priority:** High (compliance and code quality)

**Estimated Duration:** 8-12 hours

---

## Overview

This phase addresses four categories of issues:

1. **Code Quality (High):** Clippy warnings at pedantic level that indicate potential issues
2. **Syntax Consistency (High):** Example and workflow fixture files still use deprecated `authority:` syntax
3. **Spec Compliance (High):** Verify SPEC-009 visibility enforcement across both type checking and import resolution
4. **CLI/Docs Consistency (Medium):** Verify user-facing help and crate docs match the post-Phase-52 contract

---

## Task Summary

| Task | Description | Files | Est. Hours |
|------|-------------|-------|------------|
| TASK-327 | Fix clippy pedantic warnings | Test files | 2 |
| TASK-328 | Update example/workflow files to capabilities syntax | `.ash` files, parser validation | 3 |
| TASK-329 | SPEC-009 visibility compliance verification | `ash-typeck/src/visibility.rs`, `ash-parser/src/import_resolver.rs` | 2 |
| TASK-330 | Documentation and CLI help consistency audit | Specs, CHANGELOG, `ash-cli` docs/help | 2 |
| TASK-331 | Final verification and closeout | All | 1-3 |

---

## TASK-327: Fix Clippy Pedantic Warnings

**Objective:** Resolve all clippy warnings at pedantic level in test code

**Current State:**
```
warning: casting `usize` to `i64` may wrap around the value
  --> crates/ash-engine/tests/e2e_capability_provider_tests.rs:67:28

warning: variables can be used directly in the `format!` string
  --> crates/ash-engine/tests/e2e_capability_provider_tests.rs:478:34
```

**Files:**
- Modify: `crates/ash-engine/tests/e2e_capability_provider_tests.rs`

### Step 1: Write test to verify current warnings exist

Run: `cargo clippy --package ash-engine --tests -- -W clippy::pedantic 2>&1 | grep -E "warning:|error:" | head -20`

Expected: Shows 9 warnings (cast_possible_wrap ×2, uninlined_format_args ×7)

### Step 2: Fix cast_possible_wrap warnings

**Location:** Lines 67, 84

**Current:**
```rust
Value::Int(self.get_count() as i64),
```

**Fix:** Use `try_into()` or add explicit allow with justification:
```rust
// Test code - count won't exceed i64 range in practice
#[allow(clippy::cast_possible_wrap)]
Value::Int(self.get_count() as i64),
```

### Step 3: Fix uninlined_format_args warnings

**Location:** Lines 478, 499, 646, 647, 833, 834, 851

**Current:**
```rust
format!("workflow main {{ ret {}; }}", i)
```

**Fix:** Use inline format args:
```rust
format!("workflow main {{ ret {i}; }}")
```

### Step 4: Run clippy to verify clean

Run: `cargo clippy --package ash-engine --tests -- -W clippy::pedantic --quiet`
Expected: No warnings

### Step 5: Run tests to verify no regressions

Run: `cargo test --package ash-engine --quiet`
Expected: All tests pass

### Step 6: Commit

```bash
git add crates/ash-engine/tests/e2e_capability_provider_tests.rs
git commit -m "fix(clippy): TASK-327 - Fix pedantic warnings in e2e tests

- Fix cast_possible_wrap warnings with justification
- Fix uninlined_format_args warnings using inline format syntax"
```

### Step 7: Codex Verification

Spawn codex sub-agent to verify:
- `cargo clippy --package ash-engine --tests -- -W clippy::pedantic` is clean
- `cargo test --package ash-engine` passes

---

## TASK-328: Update Example Files to Capabilities Syntax

**Objective:** Update all affected example and workflow `.ash` files from deprecated `authority:` syntax to new `capabilities:` syntax

**Background:** TASK-322 implemented SPEC-024 compliant `capabilities:` syntax, but several example and workflow fixture files still use old `authority:` syntax.

**Files to Modify:**
- `examples/code_review.ash`
- `examples/multi_agent_research.ash`
- `examples/03-policies/01-role-based.ash`
- `examples/03-policies/02-time-based.ash`
- `examples/04-real-world/customer-support.ash`
- `examples/04-real-world/code-review.ash`
- `examples/workflows/40_tdd_workflow.ash`
- `examples/workflows/40a_tdd_concrete_example.ash`
- `tests/workflows/code_review.ash`
- `tests/workflows/multi_agent_research.ash`

### Step 1: Find all files using authority syntax

Run: `grep -r "authority:" --include="*.ash" .`

Expected: List of 10 files

### Step 2: Update examples/code_review.ash

**Current:**
```ash
role drafter {
  authority: [read_code, create_pr, respond_to_comments],
  obligations: [ensure_tests_pass]
}
```

**New:**
```ash
role drafter {
  capabilities: [read_code, create_pr, respond_to_comments]
  obligations: [ensure_tests_pass]
}
```

Note: Remove trailing commas inside brackets (Ash syntax)

### Step 3: Update examples/multi_agent_research.ash

**Current:**
```ash
role analyst {
  authority: [search_literature, extract_findings],
  obligations: [cite_sources]
}
```

**New:**
```ash
role analyst {
  capabilities: [search_literature, extract_findings]
  obligations: [cite_sources]
}
```

### Step 4: Update examples/04-real-world/customer-support.ash

**Current:**
```ash
role supervisor {
    authority: [view_all, assign, escalate, resolve, refund],
    obligations: []
}
```

**New:**
```ash
role supervisor {
    capabilities: [view_all, assign, escalate, resolve, refund]
    obligations: []
}
```

### Step 5: Update examples/04-real-world/code-review.ash

**Current:**
```ash
role maintainer {
    authority: [merge, approve, request_changes, bypass_checks],
    obligations: []
}
```

**New:**
```ash
role maintainer {
    capabilities: [merge, approve, request_changes, bypass_checks]
    obligations: []
}
```

### Step 6: Update tests/workflows/code_review.ash

Same pattern as examples/code_review.ash

### Step 7: Update tests/workflows/multi_agent_research.ash

Same pattern as examples/multi_agent_research.ash

### Step 8: Verify syntax with the full module/program parser

Run: `cargo test --package ash-parser --quiet`
Expected: All tests pass

Note: Do **not** rely solely on `ash check` for verification here. The current engine CLI path parses a bare workflow surface and may reject module-shaped examples that contain top-level `role` or `capability` declarations. Validation for this task must exercise the parser entrypoint that supports the full source shape used by the examples.

If the existing parser test suite does not already load these files directly, add a regression test that parses each migrated file through the module/program parser.

### Step 9: Commit

```bash
git add examples/ tests/workflows/
git commit -m "fix(syntax): TASK-328 - Update examples to capabilities: syntax

- Replace deprecated authority: with capabilities:
- Remove trailing commas inside capability lists
- Align all affected example/workflow files with SPEC-024"
```

### Step 10: Codex Verification

Spawn codex sub-agent to verify:
- No `authority:` remains in any `.ash` file
- `cargo test --package ash-parser` passes
- Migrated files are covered by parser validation

---

## TASK-329: SPEC-009 Visibility Compliance Verification

**Objective:** Verify SPEC-009 visibility implementation is complete and compliant across both type checking and import resolution

**Background:** Review identified potential compliance gaps in restricted visibility handling, including the import-resolution path.

**Spec Reference:** SPEC-009 Section 3.2, 7.1

### Step 1: Review current implementation

Read: `crates/ash-typeck/src/visibility.rs`

Check for:
- `Visibility::Crate` implementation
- `Visibility::Super { levels }` implementation
- `is_visible_path()` logic for each variant

Read: `crates/ash-parser/src/import_resolver.rs`

Check for:
- Import visibility enforcement for `Visibility::Crate`
- `Visibility::Super { .. }` handling
- `Visibility::Restricted { .. }` handling

### Step 2: Review test coverage

Read: `crates/ash-typeck/tests/visibility_test.rs`

Check for tests covering:
- `pub(crate)` visibility
- `pub(super)` visibility with levels
- Edge cases (root module, nested modules)

Read: `crates/ash-parser/src/import_resolver.rs` tests

Check for tests covering:
- Private item rejection
- `pub(super)` import allow/deny cases
- Restricted-path import allow/deny cases

### Step 3: Write missing tests (if any)

If gaps found, add property tests:
```rust
#[test]
fn test_pub_crate_visible_same_crate() {
    // Item in crate::a should be visible from crate::b
}

#[test]
fn test_pub_crate_not_visible_other_crate() {
    // Would need multi-crate test setup
}
```

### Step 4: Run visibility tests

Run: `cargo test --package ash-typeck visibility --quiet`
Expected: All tests pass

Run: `cargo test --package ash-parser import_resolver --quiet`
Expected: All tests pass

### Step 5: Commit (if changes made)

```bash
git add crates/ash-typeck/
git commit -m "test(visibility): TASK-329 - Add SPEC-009 compliance tests

- Verify pub(crate) visibility behavior
- Add edge case coverage for visibility rules"
```

### Step 6: Codex Verification

Spawn codex sub-agent to verify SPEC-009 compliance:
- All visibility variants implemented per spec
- Tests cover all visibility rules from Section 7.1
- Import resolution enforces the same restrictions as the type checker
- No TODO/FIXME in visibility code

---

## TASK-330: Documentation Consistency Audit

**Objective:** Ensure all specs, crate docs, and live CLI help reflect current implementation state

### Step 1: Audit SPEC-005 against CLI implementation

Check: `docs/spec/SPEC-005-CLI.md`

Verify:
- `--capability` flag is documented as removed
- `--input` flag is documented as removed
- Examples don't use removed flags
- `ash trace` does not expose removed input binding by accident

### Step 2: Audit SPEC-010 against engine implementation

Check: `docs/spec/SPEC-010-EMBEDDING.md`

Verify:
- HTTP section documents unimplemented status
- `with_http_capabilities()` documents error behavior

### Step 3: Verify CHANGELOG completeness

Check: `CHANGELOG.md`

Verify:
- Phase 52 tasks are documented
- Breaking changes noted
- Superseded tasks mentioned

### Step 3b: Audit CLI help and crate docs against implementation

Check: `crates/ash-cli/src/lib.rs`

Verify:
- Crate-level examples do not use removed flags
- Help text exposed by `ash trace --help` matches the documented contract

### Step 4: Update any inconsistencies found

Fix any documentation that doesn't match implementation.

### Step 5: Commit

```bash
git add docs/ CHANGELOG.md
git commit -m "docs(spec): TASK-330 - Documentation consistency audit

- Verify SPEC-005 matches CLI implementation
- Verify SPEC-010 matches engine implementation
- Update CHANGELOG with complete Phase 52 notes"
```

### Step 6: Codex Verification

Spawn codex sub-agent to verify:
- No removed flags in documentation examples
- All specs match implementation
- Live CLI help does not drift from documented flags
- CHANGELOG is complete

---

## TASK-331: Final Verification and Phase Closeout

**Objective:** Comprehensive verification of all Phase 53 work

### Step 1: Full test suite

Run: `cargo test --workspace --quiet`
Expected: All tests pass (including any new ones)

### Step 2: Full clippy check

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: No warnings

### Step 3: Format check

Run: `cargo fmt --check`
Expected: Clean

### Step 4: Documentation build

Run: `cargo doc --workspace --no-deps`
Expected: No warnings

### Step 5: Example file syntax check

Run: `cargo test --package ash-parser --quiet`
Expected: Parser coverage passes for migrated example/workflow files

Run: `cargo run --package ash-cli --bin ash -- trace --help`
Expected: Help output does not advertise removed input binding unless intentionally supported and documented

### Step 6: Update PLAN-INDEX.md

Add Phase 53 section documenting all tasks as complete.

### Step 7: Final commit

```bash
git add docs/plan/PLAN-INDEX.md
git commit -m "docs(plan): TASK-331 - Phase 53 closeout

- All remediation tasks complete
- Tests passing, clippy clean
- Documentation consistent with implementation"
```

### Step 8: Codex Phase Audit

Spawn codex sub-agent for comprehensive phase audit:

```
delegate_task to codex:
  goal: "Audit Phase 53 completion"
  context: |
    Phase: 53 Post-Review Remediation
    Tasks: TASK-327 through TASK-331
    
    Audit checklist:
    1. Clippy warnings resolved (TASK-327)
    2. Example files use capabilities syntax (TASK-328)
    3. SPEC-009 compliance verified (TASK-329)
    4. Documentation consistent (TASK-330)
    5. All tests pass
    6. PLAN-INDEX updated
    
    Run:
    - cargo test --workspace
    - cargo clippy --workspace --all-targets --all-features -- -D warnings
    - cargo fmt --check
    - Verify no authority: in .ash files
    - Verify parser coverage for migrated example/workflow files
    - Verify live CLI help matches the documented flag contract
    - Verify specs match implementation
```

---

## Success Criteria

Phase 53 is complete when:

- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` produces no output
- [ ] `cargo test --workspace` passes with no failures
- [ ] No `.ash` files contain `authority:` syntax
- [ ] Migrated example/workflow files are covered by parser validation
- [ ] Live CLI help matches the documented flag contract
- [ ] All specs match implementation
- [ ] PLAN-INDEX.md documents Phase 53 as complete
- [ ] Codex phase audit passes

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Example files have complex authority patterns | Test each file after conversion using parser coverage that supports full modules, not just `ash check` |
| Visibility tests reveal compliance gaps | Review both `ash-typeck` and `ash-parser` visibility enforcement before closing the task |
| CLI help or crate docs still expose removed flags | Audit live `ash --help` / `ash trace --help` output alongside spec text |
| Clippy warnings in dependencies | Only fix project code, not deps |

---

## Dependencies

None - Phase 53 is self-contained remediation work.

---

## Notes

- TASK-327 and TASK-328 can be done in parallel
- TASK-329 may reveal gaps requiring new tasks (don't try to fix everything)
- TASK-330 depends on all other tasks
- TASK-331 is the final gate
- If example validation still fails after TASK-328 syntax migration, treat that as a blocking parser/CLI contract gap and create follow-up remediation work instead of weakening verification
