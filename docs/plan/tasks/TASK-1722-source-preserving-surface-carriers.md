# TASK-1722: Design the source-preserving surface syntax carrier slice

## Status: ✅ Completed

## Summary

Turn the TASK-1721 inventory into a concrete Rust carrier design for the first source-preserving
surface syntax slice. The design must be narrow enough to implement without committing to the full
future macro system.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- TASK-1721 inventory artifact

## Dependencies

- ✅ TASK-1721: Parser AST and lowering inventory

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full `ash_syntax`/`ash_syn` library | SPEC-095c §5 | Requires broader parser/tooling design | Partial | Design only the first carrier slice now | Design names exact module/API boundaries and defers full macro library explicitly |

## Files

- `docs/design/phase-168-source-preserving-surface-carriers.md`
- `crates/ash-parser/src/surface.rs`
- Candidate future module paths identified by TASK-1721

## Requirements

1. Define the first implementable carrier slice for spans, origin metadata, delimiters/grouping,
   raw operator tokens, attributes, and comments.
2. Decide whether carriers extend `crates/ash-parser/src/surface.rs` directly or introduce a new
   parser-internal module first.
3. Distinguish required carriers for Phase 168 from deferred full macro/hygiene carriers.
4. Define migration constraints for existing parser tests and downstream crates.
5. Include a small API sketch and explicit non-goals.

## Work steps

1. Read TASK-1721's inventory and `SPEC-095c` §3-§6.
2. Draft `docs/design/phase-168-source-preserving-surface-carriers.md`.
3. Include a Current → Target table for relevant Rust types.
4. Include exact recommended tests for the implementation task that follows this design.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/design/phase-168-source-preserving-surface-carriers.md").read_text(); assert "Current" in s and "Target" in s and "non-goals" in s.lower()'
checklist:
  - [x] Carrier design artifact exists.
  - [x] Module/API boundary is explicit.
  - [x] Full macro system remains explicitly deferred.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the carrier design consumed by TASK-1723.
