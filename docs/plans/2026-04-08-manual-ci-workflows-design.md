# Manual CI Workflows Design

**Goal:** Disable automatic GitHub Actions execution while keeping the existing workflows manually runnable.

**Design:** Replace the current `push`, `pull_request`, and `schedule` triggers in every repository workflow with `workflow_dispatch` only. This preserves the current job definitions and keeps workflow execution available through the GitHub Actions UI or API without creating automatic CI load.

**Scope:** `.github/workflows/ci-fast.yml`, `.github/workflows/differential-testing.yml`, `.github/workflows/lean-reference.yml`, plus the required changelog and planning updates for repository policy compliance.
