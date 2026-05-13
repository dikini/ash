# TASK-870: Phase 115 review remediation

## Status: ✅ Complete

## Description

Reserve a mandatory independent review remediation slice for Phase 115 after closeout, with all findings addressed before final completion.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-869 completion

## Files / Ownership

- Modify: files identified by independent review
- Expected review surfaces include: `docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`, `docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`, `docs/plan/PLAN-INDEX.md`, `docs/spec/README.md`, `CHANGELOG.md`, `docs/plan/tasks/TASK-857-*.md` through `TASK-870-*.md`, `docs/plan/audits/TASK-858-associated-family-computation-audit.md`, and `docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md`, plus all Phase 115 code/test files changed during implementation.

## Requirements

### Functional Requirements

1. Run independent review across SPEC-063, PLAN-111, TASK-857 through TASK-869, changed code, acceptance artifact, and verification evidence.
2. Treat blocking, important, minor, and non-blocking findings as work to address unless explicitly documented as out-of-scope by spec authority.
3. Patch docs/code/tests and rerun focused plus broad gates after the final change.
4. Update status surfaces and changelog with actual remediation evidence.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Run review

- Delegate independent review with axes: spec conformance, task/order drift, live-code feasibility, non-inversion, summary opacity, diagnostics, verification honesty.

### Step 2: Address findings

- Patch every finding, including non-blocking clarity items, unless impossible or out of scope; document any scoped exception.

### Step 3: Reverify

- Rerun broad closeout gates after the final remediation patch and record evidence.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Independent TASK-870 review found six remediations; all were addressed in this slice:
  - Finding 1: public/source typechecking paths now accept explicit associated-family projections in public type positions. Regression coverage: `crates/ash-typeck/tests/task_870_associated_family_public_lowering.rs::task_870_explicit_family_projection_is_accepted_in_public_fn_signature_positions` and `crates/ash-engine/tests/task_870_associated_family_public_lowering.rs::task_870_module_loader_accepts_explicit_family_projection_in_public_type_alias`.
  - Finding 2: SPEC-035 compatibility spelling and explicit `<Interface<Args>>::Assoc` spelling now canonicalize equivalently for abstract `List<X>` arguments and reduce without output inversion. Regression coverage: `task_870_compat_and_explicit_iterator_list_x_item_are_canonical_equivalent_and_reduce`.
  - Finding 3: associated-family summary dependency-closure conversion now fails closed instead of silently shortening unrepresentable projection argument spines. Regression coverage: `task_867_import_rejects_lossy_associated_projection_argument_spine`, `task_867_import_rejects_lossy_domain_constructor_argument_spine`, `task_867_export_rejects_unrepresentable_nested_projection_argument`, and `task_867_export_preserves_representable_projection_argument_spine`.
  - Finding 4: TASK-868 acceptance matrix row 5 now cites the TASK-870 compatibility-vs-explicit equivalence test for abstract arguments.
  - Finding 5: TASK-869 records retained broad-gate evidence for `cargo fmt --check`, `git diff --check`, workspace check/clippy/test/doc, and the scoped Markdown check, including exit status and available counts/summaries without overclaiming an unavailable full test aggregate.
  - Finding 6: the empty associated-family scheme diagnostic now only reports the actual empty-equation condition, and the focused TASK-861 regression asserts that the confusing `exactly one equation` clause is absent.
- Focused remediation verification passed:
  - `cargo test -p ash-typeck --test task_861_associated_family_registration -- --nocapture`: exit 0; 8 passed, 0 failed.
  - `cargo test -p ash-typeck --test task_867_associated_family_import -- --nocapture`: exit 0; 9 passed, 0 failed.
  - `cargo test -p ash-typeck --test task_870_associated_family_public_lowering -- --nocapture`: exit 0; 3 passed, 0 failed.
  - `cargo test -p ash-engine --test task_870_associated_family_public_lowering -- --nocapture`: exit 0; 1 passed, 0 failed.
  - `cargo test -p ash-cli --test cli_input_workflow_test -- --nocapture`: exit 0; 1 passed, 1 ignored, 0 failed.
  - `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`: exit 0.
- Broad verification after the final remediation patch passed:
  - `cargo fmt --check`: exit 0; no formatting diffs reported.
  - `git diff --check`: exit 0; no whitespace errors reported.
  - `cargo check --workspace`: exit 0; workspace check completed successfully.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0; no warnings emitted under `-D warnings`.
  - `cargo doc --workspace --no-deps` plus warning grep: exit 0; workspace API docs generated and no `warning:` lines were present.
  - `cargo test --workspace`: exit 0; workspace tests completed successfully after the final remediation patch.
  - Scoped Markdown trailing-whitespace and relative-link check: exit 0 over the Phase 115 documentation packet including the TASK-868 acceptance audit artifact.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - |
    cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase115-doc.log
  - |
    ! grep -i '^warning:' /tmp/ash-phase115-doc.log
  - |
    python3 - <<'PY'
    import re, sys
    from pathlib import Path
    files=[Path('docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),Path('docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),Path('docs/plan/PLAN-INDEX.md'),Path('docs/spec/README.md'),Path('CHANGELOG.md'),Path('docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md')]
    files += sorted(Path('docs/plan/tasks').glob('TASK-85[7-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-86[0-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-870-*.md'))
    link=re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
    bad=[]
    for p in files:
        txt=p.read_text(); fence=False
        for n,line in enumerate(txt.splitlines(),1):
            if line.strip().startswith('```'):
                fence=not fence; continue
            if line.rstrip()!=line: bad.append(f'{p}:{n}: trailing whitespace')
            if fence: continue
            for m in link.finditer(line):
                target=m.group(1).split('#',1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target): continue
                if not (p.parent/target).exists(): bad.append(f'{p}:{n}: {target}')
    if bad:
        print('\n'.join(bad)); sys.exit(1)
    PY
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Post-review remediated Phase 115 packet and implementation ready for final status.
