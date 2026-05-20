# TASK-932: Alpha Closeout Review Remediation

## Status: ✅ Complete

## Description

Close PLAN-118 only after broad gates, docs/status reconciliation, changelog updates, and independent review findings are patched and re-reviewed.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-931 completion

## Requirements

### Functional Requirements

1. Reconcile statuses across SPEC-069, SPEC-070, PLAN-118, PLAN-INDEX, TASK-919 through TASK-932, docs/spec/README.md, and CHANGELOG.md.
2. Run fresh broad gates on the final diff and record exact evidence in the closeout artifact.
3. Run independent review focused on spec correctness, runtime-regime consistency, task/status drift, stale authority claims, and verification evidence.
4. Patch every blocking review finding and rerun affected focused/broad checks before changing status to complete.
5. Mark SPEC-069/SPEC-070 Implemented MVP only if the evidence supports every non-deferred acceptance row; otherwise preserve honest Draft/Partial status and explicit deferrals.

### Property Requirements

Property tests are required for Rust semantic tasks when TASK-920 identifies a stable strategy. Documentation-only or audit tasks must instead provide corpus consistency evidence.

## TDD Steps

### Step 1: Re-run broad final gates

Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `scripts/check-rust-tests.sh --workspace`, `cargo doc --workspace --no-deps`, scoped docs-link checks, and `git diff --check` on the final diff.

### Step 2: Reconcile status surfaces

Update SPEC-069, SPEC-070, PLAN-118, PLAN-INDEX, TASK-919 through TASK-932, docs/spec/README.md, and CHANGELOG.md so every status and acceptance claim matches the evidence.

### Step 3: Run independent review

Request an independent review of the final packet and implementation, specifically checking stale authority handoffs, runtime-regime consistency, acceptance evidence, and task/status drift.

### Step 4: Patch review findings and re-review

Patch all blocking findings, rerun affected focused/broad checks, and request focused re-review before declaring PLAN-118 closed.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - cargo fmt --check
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - scripts/check-rust-tests.sh --workspace
  - cargo doc --workspace --no-deps
  - |
    python3 - <<'PY'
    from pathlib import Path
    import re
    files = [
        Path("docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md"),
        Path("docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md"),
        Path("docs/plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md"),
        Path("docs/plan/PLAN-INDEX.md"),
        *sorted(Path("docs/plan/tasks").glob("TASK-91[9]-*.md")),
        *sorted(Path("docs/plan/tasks").glob("TASK-92[0-9]-*.md")),
        *sorted(Path("docs/plan/tasks").glob("TASK-93[0-2]-*.md")),
        Path("docs/spec/README.md"),
        Path("CHANGELOG.md"),
    ]
    pat = re.compile(r"\[[^\]]+\]\(([^)]+\.md(?:#[^)]+)?)\)")
    bad = []
    for p in files:
        for m in pat.finditer(p.read_text()):
            target = m.group(1).split("#", 1)[0]
            if not (p.parent / target).resolve().exists():
                bad.append(f"{p}:{m.group(1)}")
    assert not bad, "broken local markdown links: " + repr(bad)
    print(f"checked {len(files)} files")
    PY
  - git diff --check
checklist:
  - [x] Broad Rust gates pass on final diff
  - [x] Scoped docs links pass
  - [x] Independent review findings patched
  - [x] Specs/plan/tasks/index/changelog statuses reconciled
```

## Dependencies for Next Task

This task outputs:
- Closeout review artifact and status reconciliation.

## Notes

- File targets to inspect or modify: `docs/spec/README.md`, `docs/plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md`, `docs/plan/PLAN-INDEX.md`, `CHANGELOG.md`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.

## Closeout Review Progress

- Independent closeout remediation completed. Focused remediation covers the stdlib corpus baseline, module-loader import-continuation regression guard, RuntimeKernel admission fail-closed invoke fallback behavior, admitted-binding projected fallback dispatch with action-surface narrowing, explicit hidden workflow `ActEnv` admission projection, capability implementation dependency alias dispatch, transported effectful closure admission preservation, standard pilot explicit admission projection, stable SHA-256 RuntimeKernel source/summary identities, daemon reload parse/check validation, and HKT Monad evidence fixture stabilization.
- Final closeout evidence (2026-05-20T17:26:37Z, branch `phase122-alpha-runtime`):
  - `cargo fmt --check` passed.
  - `cargo check --workspace` passed.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
  - `scripts/check-rust-tests.sh --workspace` passed with exit code 0.
  - `cargo doc --workspace --no-deps` passed.
  - Scoped local Markdown link check over SPEC-069, SPEC-070, PLAN-118, PLAN-INDEX, TASK-919 through TASK-932, docs/spec/README.md, and CHANGELOG.md passed (`checked 20 files`).
  - `git diff --check` passed.
- Independent Codex-style review after the broad gates found the in-progress surfaces were internally consistent and recommended either keeping SPEC-069/SPEC-070 honest as Draft/Partial or documenting exact Implemented MVP boundaries before promotion. TASK-932 therefore closes Phase 122 as an honest Partial MVP rather than over-promoting deferred A69/A70 rows.
- Post-closeout review remediation (2026-05-20): Codex review found three blocking gaps; TASK-932 patched invoke fallback to execute through projected admitted binding contexts rather than ambient `ActEnv`, changed one-shot/daemon RuntimeKernel identity helpers from `DefaultHasher` to SHA-256 provenance hashes, and made daemon reload/indexing reject parse/check-invalid workflow sources while preserving the previous admitted index.
- Remaining documented limitations are retained in `docs/plan/audits/TASK-931-alpha-acceptance-matrix.md`, especially A69-8 execution coverage, A69-12/A70-8 bytecode-level cross-host artifact equivalence, A70-2 full admission-profile rejection before user body execution, A70-4 daemon args/config/admission-profile fields, A70-6 policy-profile admission breadth, and A70-7 daemon child-failure execution traces.
