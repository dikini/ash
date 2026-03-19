# Spec-to-Implementation Convergence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring the Ash repository to a state where the specifications are internally consistent and the Rust implementation matches those specifications across syntax, AST, lowering/IR, type checking, interpreter/runtime verification, CLI/REPL, and standard library behavior.

**Architecture:** This is a spec-first convergence plan. It first freezes canonical contracts in the spec set, then defines explicit handoff contracts between implementation layers, then executes fresh Rust follow-up tasks in dependency order. Each task is intentionally scoped to one contract boundary or one tightly coupled contract pair so it can be executed with TDD and verified independently.

**Tech Stack:** Markdown specs and plans, Rust workspace crates (`ash-core`, `ash-parser`, `ash-typeck`, `ash-interp`, `ash-engine`, `ash-cli`, `ash-repl`), property-based tests with `proptest`, workspace verification via `cargo fmt`, `cargo clippy`, `cargo test`, and project gate scripts.

---

## Planning assumptions

- Existing tasks are historical context only.
- New follow-up tasks should be created independently from the old task chain.
- Rust implementation tasks must use TDD and consult the Rust best-practices guidance.
- No downstream layer should be stabilized against an unfrozen upstream contract.

## Sequencing rules

1. No implementation task starts before its governing spec contract is frozen.
2. No interpreter/runtime task starts before parser, AST, and lowering contracts are frozen for that feature.
3. CLI/REPL tasks come after the underlying parser/type/interpreter contracts they expose.
4. Every task must list explicit non-goals to prevent scope creep.

## Verification defaults

Use the smallest focused verification first, then widen:
- Focused crate tests
- Feature-specific integration/property tests
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`
- Relevant project scripts when a cluster requires them

---

### Task 1 / TASK-156: Canonicalize workflow-form contracts in specs

**Contract:** Freeze the canonical written contract for `check`, `decide`, `receive`, and workflow effect vocabulary.

**Files:**
- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Reference: `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- Reference: `docs/audit/2026-03-19-rust-codebase-review-findings.md`

**Step 1: Write the failing audit checklist**
Create a checklist proving that these forms have one stable story for surface syntax, AST shape, effect behavior, and execution meaning.

**Step 2: Verify current failure**
Read the listed specs and confirm the checklist still fails for `check`, `decide`, or `receive` drift.

**Step 3: Write the minimal spec fixes**
Update only the sections needed to produce one canonical contract.

**Step 4: Verify**
Re-read the edited sections and confirm the checklist passes.

**Step 5: Commit**
```bash
git add docs/spec/SPEC-001-IR.md docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md CHANGELOG.md
git commit -m "docs: canonicalize workflow form contracts"
```

**Non-goals:** No Rust code changes. No policy-model redesign beyond dependencies required by these forms.

---

### Task 2 / TASK-157: Canonicalize policy contracts in specs

**Contract:** Choose and document one policy model that spans syntax, AST/IR, type checking, and runtime verification.

**Files:**
- Modify: `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- Modify: `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- Modify: `docs/spec/SPEC-008-DYNAMIC-POLICIES.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`

**Step 1: Write the failing audit checklist**
List the exact policy questions that currently have multiple answers: policy expression form, policy instance form, runtime decision form, and lowering representation.

**Step 2: Verify current failure**
Confirm at least one incompatible story exists across these files.

**Step 3: Write the minimal spec fixes**
Choose one policy model and remove or mark obsolete parallel interpretations.

**Step 4: Verify**
Confirm one continuous story exists from parsing through runtime.

**Step 5: Commit**
```bash
git add docs/spec/SPEC-006-POLICY-DEFINITIONS.md docs/spec/SPEC-007-POLICY-COMBINATORS.md docs/spec/SPEC-008-DYNAMIC-POLICIES.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md CHANGELOG.md
git commit -m "docs: canonicalize policy contracts"
```

**Non-goals:** No parser or type checker changes yet.

---

### Task 3 / TASK-158: Canonicalize streams and runtime-verification contracts in specs

**Contract:** Freeze `receive`, control-arm semantics, declaration requirements, runtime context shape, and verification outcomes.

**Files:**
- Modify: `docs/spec/SPEC-013-STREAMS.md`
- Modify: `docs/spec/SPEC-014-BEHAVIOURS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`

**Step 1: Write the failing audit checklist**
Enumerate all currently drifting stream and verification semantics.

**Step 2: Verify current failure**
Confirm `receive` and runtime verification semantics are inconsistent today.

**Step 3: Write the minimal spec fixes**
Canonicalize all stream and verification contracts.

**Step 4: Verify**
Confirm no contradictions remain across the listed files.

**Step 5: Commit**
```bash
git add docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-014-BEHAVIOURS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: canonicalize streams and runtime verification"
```

**Non-goals:** No interpreter changes yet.

---

### Task 4 / TASK-159: Canonicalize REPL and CLI contracts in specs

**Contract:** Freeze one command surface and one authority split for CLI shell vs REPL behavior.

**Files:**
- Modify: `docs/spec/SPEC-005-CLI.md`
- Modify: `docs/spec/SPEC-011-REPL.md`
- Modify: `docs/spec/SPEC-016-OUTPUT.md`

**Step 1: Write the failing audit checklist**
List command-surface, history, output, and authority questions that currently have multiple answers.

**Step 2: Verify current failure**
Confirm the current docs do not imply one authoritative REPL story.

**Step 3: Write the minimal spec fixes**
Freeze one REPL command set and one CLI/REPL boundary.

**Step 4: Verify**
Confirm no command-surface drift remains across the listed files.

**Step 5: Commit**
```bash
git add docs/spec/SPEC-005-CLI.md docs/spec/SPEC-011-REPL.md docs/spec/SPEC-016-OUTPUT.md CHANGELOG.md
git commit -m "docs: canonicalize repl and cli contracts"
```

**Non-goals:** No CLI or REPL code changes yet.

---

### Task 5 / TASK-160: Canonicalize ADT contracts in specs

**Contract:** Freeze type-definition syntax, constructor semantics, runtime variant shape, pattern checking, exhaustiveness, and stdlib helper requirements.

**Files:**
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `docs/spec/SPEC-013-STREAMS.md`
- Modify: `docs/spec/SPEC-014-BEHAVIOURS.md`

**Step 1: Write the failing audit checklist**
List all ADT questions that currently differ between spec text and code reality.

**Step 2: Verify current failure**
Confirm one or more conflicts remain in type shape, value shape, or stdlib obligations.

**Step 3: Write the minimal spec fixes**
Choose one canonical ADT story and update the listed files.

**Step 4: Verify**
Confirm the canonical ADT story is internally consistent.

**Step 5: Commit**
```bash
git add docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-020-ADT-TYPES.md docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-014-BEHAVIOURS.md CHANGELOG.md
git commit -m "docs: canonicalize adt contracts"
```

**Non-goals:** No Rust changes yet.

---

### Task 6: Write surface-syntax to parser-AST handoff reference

**Contract:** Define accepted syntax forms, produced surface AST nodes, and parser errors for the stabilized features.

**Files:**
- Create: `docs/reference/surface-to-parser-contract.md`
- Reference: `docs/spec/SPEC-002-SURFACE.md`
- Reference: `docs/spec/SPEC-013-STREAMS.md`
- Reference: `docs/spec/SPEC-020-ADT-TYPES.md`

**Step 1: Write the failing checklist**
Identify the syntax forms that lack an explicit parser handoff contract.

**Step 2: Verify current failure**
Confirm no single reference file currently covers this handoff.

**Step 3: Write the reference**
Document inputs, outputs, and legal parser failures.

**Step 4: Verify**
Check that `check`, `decide`, `receive`, policy forms, and ADT declarations are covered.

**Step 5: Commit**
```bash
git add docs/reference/surface-to-parser-contract.md CHANGELOG.md
git commit -m "docs: add surface to parser contract"
```

**Non-goals:** No code changes.

---

### Task 7: Write parser-AST to core-AST/lowering handoff reference

**Contract:** Define which surface nodes lower to which core forms and which invalid combinations are rejected.

**Files:**
- Create: `docs/reference/parser-to-core-lowering-contract.md`
- Reference: `docs/spec/SPEC-001-IR.md`
- Reference: `docs/spec/SPEC-002-SURFACE.md`
- Reference: `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- Reference: `docs/spec/SPEC-013-STREAMS.md`
- Reference: `docs/spec/SPEC-020-ADT-TYPES.md`

**Step 1: Write the failing checklist**
List the currently ambiguous lowering cases.

**Step 2: Verify current failure**
Confirm no explicit lowering contract exists for the drifted features.

**Step 3: Write the reference**
Map accepted surface nodes to canonical core forms.

**Step 4: Verify**
Check that policy and `receive` lowering are explicitly covered.

**Step 5: Commit**
```bash
git add docs/reference/parser-to-core-lowering-contract.md CHANGELOG.md
git commit -m "docs: add parser to core lowering contract"
```

**Non-goals:** No lowering implementation changes.

---

### Task 8: Write type/runtime handoff references

**Contract:** Define type-checking outputs relied on by interpreter and runtime verification, and the required runtime behavior exposed to CLI/REPL and stdlib.

**Files:**
- Create: `docs/reference/type-to-runtime-contract.md`
- Create: `docs/reference/runtime-observable-behavior-contract.md`
- Reference: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`
- Reference: `docs/spec/SPEC-005-CLI.md`
- Reference: `docs/spec/SPEC-011-REPL.md`
- Reference: `docs/spec/SPEC-016-OUTPUT.md`

**Step 1: Write the failing checklist**
List the missing guarantees between type checking, interpreter behavior, runtime verification, and observables.

**Step 2: Verify current failure**
Confirm the current docs do not define these handoffs explicitly.

**Step 3: Write the references**
Document required outputs, rejected states, and observable behaviors.

**Step 4: Verify**
Check that runtime verification, REPL output, and stdlib-visible ADT behavior are covered.

**Step 5: Commit**
```bash
git add docs/reference/type-to-runtime-contract.md docs/reference/runtime-observable-behavior-contract.md CHANGELOG.md
git commit -m "docs: add type and runtime handoff contracts"
```

**Non-goals:** No Rust changes.

---

### Task 9: Route `receive` through the main parser

**Files:**
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/parse_receive.rs`
- Test: `crates/ash-parser/tests/receive_parser.rs`

**Step 1: Write the failing test**
Add parser tests proving that canonical `receive` syntax is accepted by the main workflow parser entrypoint.

**Step 2: Run test to verify it fails**
Run: `cargo test -p ash-parser receive_parser -- --nocapture`
Expected: fail because the main parser does not dispatch correctly.

**Step 3: Write minimal implementation**
Wire `receive` into the main parser entrypoint without changing downstream semantics yet.

**Step 4: Run focused verification**
Run: `cargo test -p ash-parser receive_parser -- --nocapture`
Expected: pass.

**Step 5: Run broader verification**
Run: `cargo test -p ash-parser`
Expected: pass.

**Step 6: Commit**
```bash
git add crates/ash-parser/src/parse_workflow.rs crates/ash-parser/src/parse_receive.rs crates/ash-parser/tests/receive_parser.rs CHANGELOG.md
git commit -m "fix: route receive through main parser"
```

**Non-goals:** No lowering or interpreter support.

---

### Task 10: Align `check` and `decide` AST contracts

**Files:**
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Test: `crates/ash-core/src/ast.rs`
- Test: `crates/ash-parser/tests/workflow_contracts.rs`

**Step 1: Write the failing tests**
Add tests proving `check` and `decide` surface forms lower into canonical AST shapes.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-parser workflow_contracts -- --nocapture`
Expected: fail due to current contract mismatch.

**Step 3: Write minimal implementation**
Update core and surface AST definitions and parser handling so both forms match the canonical spec.

**Step 4: Run focused verification**
Run: `cargo test -p ash-parser workflow_contracts -- --nocapture`
Run: `cargo test -p ash-core`
Expected: pass.

**Step 5: Commit**
```bash
git add crates/ash-core/src/ast.rs crates/ash-parser/src/surface.rs crates/ash-parser/src/parse_workflow.rs crates/ash-parser/tests/workflow_contracts.rs CHANGELOG.md
git commit -m "fix: align check and decide ast contracts"
```

**Non-goals:** No policy lowering or runtime behavior changes.

---

### Task 11: Replace placeholder policy lowering

**Files:**
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-core/src/ast.rs`
- Test: `crates/ash-parser/tests/policy_lowering.rs`
- Test: `crates/ash-core/src/ast.rs`

**Step 1: Write the failing tests**
Add tests proving canonical policy forms lower into a meaningful core representation rather than debug-string placeholders or dummy obligations.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-parser policy_lowering -- --nocapture`
Expected: fail against current placeholder lowering.

**Step 3: Write minimal implementation**
Introduce the canonical core policy representation and update lowering accordingly.

**Step 4: Run focused verification**
Run: `cargo test -p ash-parser policy_lowering -- --nocapture`
Run: `cargo test -p ash-core`
Expected: pass.

**Step 5: Commit**
```bash
git add crates/ash-parser/src/lower.rs crates/ash-core/src/ast.rs crates/ash-parser/tests/policy_lowering.rs CHANGELOG.md
git commit -m "fix: replace placeholder policy lowering"
```

**Non-goals:** No runtime policy evaluator rewrite yet.

---

### Task 12: Lower `receive` into canonical core form

**Files:**
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Test: `crates/ash-parser/tests/receive_lowering.rs`
- Test: `crates/ash-core/src/ast.rs`

**Step 1: Write the failing tests**
Add tests proving surface `receive` lowers into a real canonical core representation.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-parser receive_lowering -- --nocapture`
Expected: fail because lowering currently collapses `receive`.

**Step 3: Write minimal implementation**
Add the canonical core `receive` form and lower into it.

**Step 4: Run focused verification**
Run: `cargo test -p ash-parser receive_lowering -- --nocapture`
Run: `cargo test -p ash-core`
Expected: pass.

**Step 5: Commit**
```bash
git add crates/ash-core/src/ast.rs crates/ash-parser/src/lower.rs crates/ash-parser/tests/receive_lowering.rs CHANGELOG.md
git commit -m "fix: lower receive into canonical core form"
```

**Non-goals:** No runtime receive execution yet.

---

### Task 13: Align type checking for policies and `receive`

**Files:**
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/policy_check.rs`
- Modify: `crates/ash-typeck/src/capability_check.rs`
- Test: `crates/ash-typeck/tests/policy_contracts.rs`
- Test: `crates/ash-typeck/tests/receive_contracts.rs`

**Step 1: Write the failing tests**
Add tests proving canonical policy forms and `receive` declarations are type-checked consistently.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-typeck policy_contracts receive_contracts -- --nocapture`
Expected: fail due to current contract mismatch.

**Step 3: Write minimal implementation**
Update type checking and declaration checking to match the canonical contracts.

**Step 4: Run focused verification**
Run: `cargo test -p ash-typeck policy_contracts receive_contracts -- --nocapture`
Expected: pass.

**Step 5: Run broader verification**
Run: `cargo test -p ash-typeck`
Expected: pass.

**Step 6: Commit**
```bash
git add crates/ash-typeck/src/check_expr.rs crates/ash-typeck/src/policy_check.rs crates/ash-typeck/src/capability_check.rs crates/ash-typeck/tests/policy_contracts.rs crates/ash-typeck/tests/receive_contracts.rs CHANGELOG.md
git commit -m "fix: align type checking for policies and receive"
```

**Non-goals:** No interpreter execution changes yet.

---

### Task 14: Unify runtime verification context and obligation enforcement

**Files:**
- Modify: `crates/ash-typeck/src/runtime_verification.rs`
- Modify: `crates/ash-typeck/src/capability_typecheck.rs`
- Modify: `crates/ash-typeck/tests/runtime_verification_contracts.rs`

**Step 1: Write the failing tests**
Add tests proving canonical runtime context shape and aggregate obligation enforcement behavior.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-typeck runtime_verification_contracts -- --nocapture`
Expected: fail because the aggregate path does not fully enforce the intended contract.

**Step 3: Write minimal implementation**
Unify the runtime context contract and restore the required aggregate checks.

**Step 4: Run focused verification**
Run: `cargo test -p ash-typeck runtime_verification_contracts -- --nocapture`
Expected: pass.

**Step 5: Run broader verification**
Run: `cargo test -p ash-typeck`
Expected: pass.

**Step 6: Commit**
```bash
git add crates/ash-typeck/src/runtime_verification.rs crates/ash-typeck/src/capability_typecheck.rs crates/ash-typeck/tests/runtime_verification_contracts.rs CHANGELOG.md
git commit -m "fix: unify runtime verification context"
```

**Non-goals:** No CLI/REPL changes.

---

### Task 15: Implement end-to-end `receive` execution

**Files:**
- Modify: `crates/ash-interp/src/execute_stream.rs`
- Modify: `crates/ash-interp/src/eval.rs`
- Modify: `crates/ash-interp/src/stream.rs`
- Test: `crates/ash-interp/tests/receive_execution.rs`

**Step 1: Write the failing tests**
Add integration tests proving canonical `receive` behavior from parsed form through execution.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-interp receive_execution -- --nocapture`
Expected: fail because `receive` is not fully implemented end-to-end.

**Step 3: Write minimal implementation**
Implement canonical runtime behavior for `receive` using the frozen core and verification contracts.

**Step 4: Run focused verification**
Run: `cargo test -p ash-interp receive_execution -- --nocapture`
Expected: pass.

**Step 5: Run broader verification**
Run: `cargo test -p ash-interp`
Expected: pass.

**Step 6: Commit**
```bash
git add crates/ash-interp/src/execute_stream.rs crates/ash-interp/src/eval.rs crates/ash-interp/src/stream.rs crates/ash-interp/tests/receive_execution.rs CHANGELOG.md
git commit -m "feat: implement end-to-end receive execution"
```

**Non-goals:** No REPL exposure yet.

---

### Task 16: Align runtime policy outcomes with the canonical contract

**Files:**
- Modify: `crates/ash-typeck/src/runtime_verification.rs`
- Modify: `crates/ash-interp/src/execute_observe.rs`
- Modify: `crates/ash-interp/src/execute_set.rs`
- Modify: `crates/ash-interp/src/exec_send.rs`
- Test: `crates/ash-typeck/tests/policy_runtime_outcomes.rs`
- Test: `crates/ash-interp/tests/policy_runtime_outcomes.rs`

**Step 1: Write the failing tests**
Add tests for deny, warning, and transform behavior at the runtime boundary.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-typeck policy_runtime_outcomes -- --nocapture`
Run: `cargo test -p ash-interp policy_runtime_outcomes -- --nocapture`
Expected: fail for current mismatches.

**Step 3: Write minimal implementation**
Update runtime verification and interpreter integration to match the canonical policy-outcome contract.

**Step 4: Run focused verification**
Run the same commands again.
Expected: pass.

**Step 5: Commit**
```bash
git add crates/ash-typeck/src/runtime_verification.rs crates/ash-interp/src/execute_observe.rs crates/ash-interp/src/execute_set.rs crates/ash-interp/src/exec_send.rs crates/ash-typeck/tests/policy_runtime_outcomes.rs crates/ash-interp/tests/policy_runtime_outcomes.rs CHANGELOG.md
git commit -m "fix: align runtime policy outcomes"
```

**Non-goals:** No provider-effect redesign.

---

### Task 17: Unify REPL implementation behind one authority

**Files:**
- Modify: `crates/ash-repl/src/lib.rs`
- Modify: `crates/ash-repl/src/main.rs`
- Modify: `crates/ash-cli/src/commands/repl.rs`
- Test: `crates/ash-repl/tests/repl_commands.rs`
- Test: `crates/ash-cli/tests/repl_command.rs`

**Step 1: Write the failing tests**
Add tests proving both entrypoints expose the same canonical command surface and command handling behavior.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-repl repl_commands -- --nocapture`
Run: `cargo test -p ash-cli repl_command -- --nocapture`
Expected: fail because the implementations differ today.

**Step 3: Write minimal implementation**
Choose one shared REPL authority and make both entrypoints delegate to it.

**Step 4: Run focused verification**
Run the same commands again.
Expected: pass.

**Step 5: Commit**
```bash
git add crates/ash-repl/src/lib.rs crates/ash-repl/src/main.rs crates/ash-cli/src/commands/repl.rs crates/ash-repl/tests/repl_commands.rs crates/ash-cli/tests/repl_command.rs CHANGELOG.md
git commit -m "refactor: unify repl implementation"
```

**Non-goals:** No new REPL features beyond the canonical spec.

---

### Task 18: Replace placeholder REPL type reporting

**Files:**
- Modify: `crates/ash-repl/src/lib.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/parse.rs`
- Test: `crates/ash-repl/tests/repl_type_reporting.rs`

**Step 1: Write the failing tests**
Add tests proving `:type` reports the canonical inferred type output instead of a placeholder.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-repl repl_type_reporting -- --nocapture`
Expected: fail because type reporting is placeholder-level today.

**Step 3: Write minimal implementation**
Route type reporting through the canonical parse/type-check pipeline.

**Step 4: Run focused verification**
Run the same command again.
Expected: pass.

**Step 5: Commit**
```bash
git add crates/ash-repl/src/lib.rs crates/ash-engine/src/lib.rs crates/ash-engine/src/parse.rs crates/ash-repl/tests/repl_type_reporting.rs CHANGELOG.md
git commit -m "fix: implement repl type reporting"
```

**Non-goals:** No output-format redesign outside the canonical spec.

---

### Task 19: Align ADT type, value, and pattern contracts

**Files:**
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-core/src/value.rs`
- Modify: `crates/ash-parser/src/parse_type_def.rs`
- Modify: `crates/ash-typeck/src/check_pattern.rs`
- Modify: `crates/ash-typeck/src/exhaustiveness.rs`
- Modify: `crates/ash-interp/src/pattern.rs`
- Test: `crates/ash-typeck/tests/adt_contracts.rs`
- Test: `crates/ash-interp/tests/adt_contracts.rs`

**Step 1: Write the failing tests**
Add tests proving the canonical ADT contract is shared by type definitions, runtime values, pattern typing, and pattern execution.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-typeck adt_contracts -- --nocapture`
Run: `cargo test -p ash-interp adt_contracts -- --nocapture`
Expected: fail due to current mismatches.

**Step 3: Write minimal implementation**
Align all ADT layers to the canonical contract.

**Step 4: Run focused verification**
Run the same commands again.
Expected: pass.

**Step 5: Run broader verification**
Run: `cargo test -p ash-typeck`
Run: `cargo test -p ash-interp`
Expected: pass.

**Step 6: Commit**
```bash
git add crates/ash-core/src/ast.rs crates/ash-core/src/value.rs crates/ash-parser/src/parse_type_def.rs crates/ash-typeck/src/check_pattern.rs crates/ash-typeck/src/exhaustiveness.rs crates/ash-interp/src/pattern.rs crates/ash-typeck/tests/adt_contracts.rs crates/ash-interp/tests/adt_contracts.rs CHANGELOG.md
git commit -m "fix: align adt type value and pattern contracts"
```

**Non-goals:** No stdlib helper expansion yet.

---

### Task 20: Align ADT stdlib and example surface

**Files:**
- Modify: `std/src/option.ash`
- Modify: `std/src/result.ash`
- Modify: `std/src/prelude.ash`
- Modify: `examples/README.md`
- Test: `crates/ash-parser/tests/stdlib_surface.rs`
- Test: `tests/std/`

**Step 1: Write the failing tests**
Add tests proving the standard library and examples expose the canonical Option/Result helper surface.

**Step 2: Run tests to verify they fail**
Run: `cargo test -p ash-parser stdlib_surface -- --nocapture`
Run: `cargo test --test '*'` 
Expected: at least one failure for missing helper surface.

**Step 3: Write minimal implementation**
Add only the canonical helper surface and example updates required by the spec.

**Step 4: Run focused verification**
Run: `cargo test -p ash-parser stdlib_surface -- --nocapture`
Expected: pass.

**Step 5: Run broader verification**
Run: `cargo test --workspace`
Expected: pass.

**Step 6: Commit**
```bash
git add std/src/option.ash std/src/result.ash std/src/prelude.ash examples/README.md tests/std CHANGELOG.md
git commit -m "feat: align adt stdlib surface"
```

**Non-goals:** No new ADT families beyond the canonical spec.

---

### Task 21: Final convergence audit

**Files:**
- Modify: `docs/audit/`
- Modify: `CHANGELOG.md`
- Reference: all specs, reference contracts, and affected crates

**Step 1: Write the failing final checklist**
Use the original drift classes as the acceptance checklist.

**Step 2: Verify current status**
Run the full consistency review against specs and code.

**Step 3: Write the final audit report**
Record which drift classes are now closed and which, if any, remain.

**Step 4: Run full verification**
Run:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`
- relevant repository gate scripts for changed clusters

Expected: pass.

**Step 5: Commit**
```bash
git add docs/audit CHANGELOG.md
git commit -m "docs: publish final convergence audit"
```

**Non-goals:** No feature expansion beyond convergence work.

---

## Execution notes

- Prefer one branch or worktree per task or tightly related mini-sequence.
- Do not merge downstream tasks early because the plan depends on upstream contract stability.
- For Rust tasks, use unit tests, integration tests, and `proptest` where the boundary has invariants rather than single examples.
- Update `CHANGELOG.md` for every completed task.
- After each completed task, confirm that the next task still has stable upstream assumptions.

## Completion criteria

This plan is complete only when:
- the canonical specs are internally consistent,
- the new handoff references exist and are accurate,
- every implementation task has been completed or consciously superseded,
- the final code audit reports no unresolved drift in workflow forms, policies, runtime verification, REPL/CLI, or ADTs.
