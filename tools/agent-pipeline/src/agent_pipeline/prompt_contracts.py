"""Reusable prompt-contract fragments for the agent pipeline."""

from __future__ import annotations

from pathlib import Path

STRICT_VERDICTS = "Use strict verdict vocabulary: VERIFIED, BLOCKED, NEEDS_REVISION."


def execution_roots_contract(task_dir: Path, repo_root: Path) -> str:
    """Return shared wording that separates the repository worktree from task-bundle artifacts."""
    return f"""
Execution roots:
- Repository workspace/worktree root: {repo_root}
- Task bundle artifact directory: {task_dir}
- Use the repository workspace/worktree root for code edits and repo-relative paths.
- Use the task bundle artifact directory only for generated stage artifacts, reviews, logs, manifests, and prompt files.
- Keep repository code changes in the worktree and keep generated task artifacts under {task_dir}.
""".strip()


def design_contract(task_dir: Path) -> str:
    """Return the shared design-stage quality contract."""
    return f"""
Contract-first design requirements:
- Produce a design that is reviewable from the outside, not a self-attested completion note.
- Include these sections exactly: Problem Statement, Analysis Approach, Success Criteria, Risk Assessment, Non-goals, Assumptions, Traceability.
- In Traceability, map each task requirement from {task_dir}/task.md to the design sections or decisions that address it.
- {STRICT_VERDICTS}
- If you discover missing information, record it under Assumptions and identify the impact.
""".strip()


def document_quality_contract(task_dir: Path) -> str:
    """Return shared spec/plan document quality lenses."""
    return f"""
Document quality lenses:
- Technical Critic: verify technical correctness, constraints, dependencies, and measurable acceptance criteria.
- Pedagogical Critic: make the document teachable, navigable, and easy for a fresh implementer to follow.
- Style Critic: remove ambiguity, weak language, and undefined terms.
- Traceability Matrix: map task/design/spec requirements to concrete sections, tasks, and verification points.
- Use exact file paths whenever you refer to repository files or generated artifacts.
- Include verification commands for every meaningful implementation or validation claim.
- Maintain requirement-to-task traceability so downstream stages can audit coverage.
- Reference the task bundle paths directly under {task_dir}.
""".strip()


def implementation_contract(task_dir: Path) -> str:
    """Return implementation-stage evidence requirements."""
    return f"""
Implementation evidence contract:
- Evidence before claims: run or inspect verification evidence first, then make completion claims.
- You MUST create all of these artifacts:
  - {task_dir}/impl.complete
  - {task_dir}/impl.summary.md
  - {task_dir}/impl.verification.md
- impl.summary.md must summarize code changes, exact file paths touched, and remaining risks.
- impl.verification.md must record the verification commands run, their outputs/results, and any gaps.
""".strip()


def qa_contract(task_dir: Path) -> str:
    """Return QA-stage pass/fail artifact requirements."""
    return f"""
QA review contract:
- You MUST create {task_dir}/qa.md plus exactly one verdict artifact:
  - pass: {task_dir}/qa.verified
  - fail: {task_dir}/qa.review
- In qa.md include these sections exactly: Spec compliance, Code quality, Verification evidence.
- Use fail-closed review language and explain any blocking findings with concrete evidence.
- {STRICT_VERDICTS}
""".strip()


def validate_contract(task_dir: Path) -> str:
    """Return validate-stage pass/fail artifact requirements."""
    return f"""
Final validation contract:
- You MUST create exactly one verdict artifact:
  - pass: {task_dir}/validated
  - fail: {task_dir}/validate.review
- Validate the full task bundle against produced evidence, not agent intent.
- {STRICT_VERDICTS}
- A BLOCKED or NEEDS_REVISION outcome must explain the missing artifact, failed verification, or unmet requirement.
""".strip()
