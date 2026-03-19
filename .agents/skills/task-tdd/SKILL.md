---
name: task-development-using-tdd
description: Use when implementing any project task
---

Prerequisites:
- You must know the project guidelines from AGENTS.md
- You must use test-driven-development skill for all code development (see SKILL: test-driven-development)
- You must use rust-skills for all Rust code development (see SKILL: rust-skills)
- You must follow the Main Agent / Sub-Agent workflow for all development (see AGENTS.md)

## Step 1: Discover the Next Phase

  Check your project planning documents to identify what's next:

  1. Check the plan index
  ```bash
  cat docs/plan/PLAN-INDEX.md
  ```

  2. Or list task files
  ```bash
  ls -la docs/plan/tasks/
  ```

## Step 2: Gather Context

  Before any code is written, collect the requirements:

  1. Read the task file — docs/plan/tasks/TASK-XXX-*.md must exist (per AGENTS.md: "Agents MU
     NOT implement any task without a corresponding task file")
  2. Read relevant specs — Check docs/specs/ for the behavior/invariants you're implementing
  3. Invoke rust-skills — Since this is Rust code, load the comprehensive guidelines:
     /rust-skills

## Step 3: Follow the TDD + Sub-Agent Pattern

  Per AGENTS.md, use the Main Agent / Sub-Agent workflow:

  Test dev (sub-agent) -> Code dev (sub-agent) -> QA (sub-agent) -> Main Agent review

  Your role as Main Agent:

  • Understand full context and requirements
  • Spawn specialized sub-agents with clear, specific prompts
  • Review and verify all results
  • Integrate outputs

##Step 4: Execute TDD Cycle

  For each behavior in the task:

  1. RED — Spawn Test Dev sub-agent: "Write property tests for [specific behavior]. Expected
     ariants: X, Y, Z."
  2. Verify RED — Run tests, confirm they fail correctly
  3. GREEN — Spawn Code Dev sub-agent: "Implement to satisfy these tests: [test content]"
  4. Verify GREEN — Run tests, confirm they pass
  5. REFACTOR — Spawn Code Review sub-agent or refactor yourself

##Step 5: Completion Checklist

  Before marking done (from AGENTS.md):

  • [ ] Property tests written and passing (TDD)
  • [ ] cargo test passes
  • [ ] cargo clippy --all-targets clean
  • [ ] cargo fmt --check clean
  • [ ] CHANGELOG.md updated
  • [ ] Task status updated in PLAN-INDEX.md
