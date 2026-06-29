# TASK-1721: Audit current parser AST and lowering seams against Phase 167 specs

## Status: 📝 Planned

## Summary

Audit the live parser AST, parser modules, and lowering consumers against the Phase 167 surface and
lowering specs. This task creates the implementation map that downstream Phase 168 tasks consume.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- SPEC-095a: `docs/spec/SPEC-095a-CURRENT-GRAMMAR.md`
- SPEC-095b: `docs/spec/SPEC-095b-TARGET-GRAMMAR.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- SPEC-098c: `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`

## Dependencies

- ✅ Phase 167 target surface and semantics gap closure (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Parser/macro/lowering implementation | PLAN-167 closeout | Specs were being hardened first | Yes | Audit live implementation before coding substrate | Audit artifact exists and names exact code seams |

## Files

- `docs/audit/phase-168-surface-ast-lowering-inventory.md`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_*.rs`
- `crates/ash-parser/src/lower*.rs` or current lowering modules, if present
- `crates/ash-engine/src/*.rs` lowering/adapter consumers, if present

## Requirements

1. Map current parsed surface carriers against the `SPEC-095c` layers: token/concrete, parsed
   surface AST, expanded surface AST, and Core boundary.
2. Identify every current consumer that assumes parser AST is already semantically normalized.
3. Identify whether operator-like tokens, grouping, attributes, comments, and origin metadata are
   preserved, partially preserved, or lost.
4. Identify current lowering entry points and the closest existing analog to `SPEC-098c`'s
   expanded-surface-AST input.
5. Produce a gap table with columns: spec requirement, live code seam, current behavior, downstream
   risk, proposed owning task.

## Work steps

1. Inspect `crates/ash-parser/src/surface.rs` and parser module boundaries.
2. Search for lowering functions and engine adapters that consume parser nodes.
3. Write `docs/audit/phase-168-surface-ast-lowering-inventory.md`.
4. Add follow-on references to TASK-1722 through TASK-1726 where each gap is owned.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; p=Path("docs/audit/phase-168-surface-ast-lowering-inventory.md"); s=p.read_text(); assert "SPEC-095c" in s and "SPEC-098c" in s and "gap table" in s.lower()'
checklist:
  - [ ] Inventory artifact exists.
  - [ ] Parser AST seams are mapped.
  - [ ] Lowering consumers are mapped.
  - [ ] Every major gap has an owning follow-on task.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the gap table and code-seam map consumed by TASK-1722.
