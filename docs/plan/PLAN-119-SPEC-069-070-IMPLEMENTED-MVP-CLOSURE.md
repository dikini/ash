# PLAN-119: SPEC-069/SPEC-070 Implemented MVP Closure

> **For Hermes:** Use subagent-driven-development and Codex verification for each task. Keep Phase 122 closed; this is the follow-on phase that turns its documented Partial MVP limitations into Implemented MVP evidence.

**Goal:** Promote SPEC-069 and SPEC-070 from honest Partial MVP to Implemented MVP by closing every documented acceptance-row limitation with execution-grade tests, shared runtime substrate, and reconciled status surfaces.

**Architecture:** Build on Phase 122's visible tower algebra, TCIR/AMIR/bytecode carriers, and RuntimeKernel identity/admission projection. First add the missing semantic execution proof for `do:Result` operational bottom, then make run/daemon share a bytecode artifact path, then deepen RuntimeKernel admission/profile semantics, daemon start records, and child-failure observation before final status promotion.

**Tech Stack:** Rust 2024; `ash-core`, `ash-typeck`, `ash-interp`, `ash-engine`, `ash-cli`; repo serial gate `scripts/check-rust-tests.sh --workspace`; Codex CLI for independent task and phase review.

---

## Status

📝 Planned.

## Scope

This phase closes the following Phase 122 limitations:

| Gap | Source row | Closure target |
| --- | --- | --- |
| Concrete execution proof for `fail` inside `do:Result<_, E>` | SPEC-069 A69-8 | Execution test proves `fail` remains operational bottom, not implicit `Err`. |
| Bytecode-level `ash run` / `ash daemon` equivalence | SPEC-069 A69-12, SPEC-070 A70-8 | Shared artifact construction path and tests compare verified bytecode/provenance summaries across host modes. |
| Admission-profile rejection before user code | SPEC-070 A70-2 | RuntimeKernel admission profile rejects before body execution and emits admission-specific report/status. |
| Daemon start args/config/admission-profile fields | SPEC-070 A70-4 | Daemon start protocol records args/config/admission-profile and rejects invalid admission without running body. |
| Broader policy-profile admission | SPEC-070 A70-6, NI-4 | Capability/resource/action grants are evaluated at admission and enforced through Act/Proc/Workflow execution. |
| Daemon child-failure trace | SPEC-070 A70-7 | Daemon-hosted workflow child failure is observed as Proc/Workflow failure while daemon host remains healthy. |
| Status promotion | TASK-932 limitation list | SPEC-069/SPEC-070, spec index, phase plan, PLAN-INDEX, tasks, and CHANGELOG consistently say Implemented MVP only after gates/review. |

## Non-Goals

- No remote or multi-user daemon API.
- No distributed scheduling or cluster service discovery.
- No production init-system integration.
- No arbitrary algebraic effects/effect rows/user handlers.
- No full Haskell-grade inference: unrestricted type lambdas, higher-rank polymorphism, and fully free do-target inference remain outside SPEC-069 Implemented MVP.
- No JIT/native-code generation requirement.

## Task Overview

| Task | Title | Type | Est. Hours | Depends on |
| --- | --- | --- | ---: | --- |
| TASK-933 | Implemented-MVP acceptance delta and preflight audit | Docs/Planning | 6 | TASK-932 |
| TASK-934 | `do:Result` operational-bottom execution evidence | Semantic | 8 | TASK-933 |
| TASK-935 | Shared RuntimeKernel verified artifact builder | Substrate | 12 | TASK-933 |
| TASK-936 | `ash run` / daemon bytecode artifact equivalence | Semantic | 10 | TASK-935 |
| TASK-937 | One-shot admission-profile pre-body rejection | Semantic/Substrate | 12 | TASK-933, TASK-935 |
| TASK-938 | Daemon start args/config/admission-profile protocol | Semantic/Substrate | 12 | TASK-937 |
| TASK-939 | Policy-profile grant enforcement across runtime execution | Semantic/Substrate | 14 | TASK-937, TASK-938 |
| TASK-940 | Daemon child Proc failure trace semantics | Semantic | 12 | TASK-938, TASK-939 |
| TASK-941 | SPEC-069/SPEC-070 Implemented MVP closeout | Docs/Planning | 8 | TASK-934 through TASK-940 |

Estimated total: 94 hours.

## Required Execution Discipline

1. Use a dedicated worktree for implementation.
2. Do not mark any task complete from focused tests alone.
3. Each code task must use TDD: write focused RED tests, verify failure, implement, verify GREEN.
4. Each task must update `CHANGELOG.md` if it changes code, tooling, docs policy, or release-facing status.
5. Each task must request Codex verification before status promotion.
6. TASK-941 must run a final Codex phase audit after broad gates pass.
7. If independent review finds a real blocker, reopen the relevant task and rerun focused plus broad verification after remediation.

## Global Verification Gates

TASK-941 must run:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
scripts/check-rust-tests.sh --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/phase123-doc.log && ! grep -i '^warning:' /tmp/phase123-doc.log
python3 - <<'PY'
from pathlib import Path
files = [
    Path('docs/plan/PLAN-119-SPEC-069-070-IMPLEMENTED-MVP-CLOSURE.md'),
    *sorted(Path('docs/plan/tasks').glob('TASK-93[3-9]-*.md')),
    *sorted(Path('docs/plan/tasks').glob('TASK-94[0-1]-*.md')),
]
missing = []
for p in files:
    for raw in p.read_text().split('](')[1:]:
        target = raw.split(')', 1)[0].split('#', 1)[0]
        if target.endswith('.md') and not (p.parent / target).resolve().exists():
            missing.append(f'{p}: {target}')
assert not missing, missing
print(f'checked {len(files)} files')
PY
```

## Promotion Rule

SPEC-069/SPEC-070 may be promoted to Implemented MVP only after TASK-941 confirms:

- every formerly partial row has concrete evidence cited in `docs/plan/audits/TASK-931-alpha-acceptance-matrix.md` or a Phase 123 successor section;
- no current status surface still says SPEC-069/SPEC-070 are Partial MVP except historical Phase 122 notes;
- all broad gates pass on the final diff;
- Codex phase audit reports no blockers.
