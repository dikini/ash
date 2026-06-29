# PLAN-169: Surface Expansion and Notation Elaboration

## Status: 📝 Planned; task packet created

## Overview

Phase 169 turns the Phase 168 surface substrate into the first usable expansion and notation
elaboration pass. Phase 168 proved that parser-only forms can be preserved and rejected fail-closed;
Phase 169 should make that boundary useful for real syntax by adding reusable traversal, notation
item parsing, raw operator preservation, active notation tables, binary operator-section elaboration,
and a high-level lowering gate that accepts only expanded surface modules.

The phase is deliberately narrower than "finish macros" or "finish surface-to-Core lowering". It
implements enough notation infrastructure for built-in and declared binary infix notation to become
ordinary callable syntax before Core, while keeping macro hygiene, binder-introducing mixfix forms,
general `do`/comprehension lowering, and full origin sidecar threading as later packets.

## Source specs and prior artifacts

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/audit/phase-168-surface-to-core-lowering-inventory.md`
- `docs/design/phase-168-source-preserving-surface-carriers.md`
- `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`

## Goals

- [ ] Replace the Phase 168 ad-hoc expansion scan with reusable traversal APIs for expression-bearing
      module, workflow, contract, policy, law, and proof surfaces.
- [ ] Parse and preserve minimal notation declarations as source items without implementing full macro
      expansion or binder-introducing mixfix forms.
- [ ] Preserve raw operator token spelling for built-in infix expressions and section nodes so
      diagnostics and future formatting do not depend only on semantic `BinaryOp` variants.
- [ ] Build a minimal active notation table with local declarations, duplicate/conflict diagnostics,
      and explicit deferral of import/export propagation if the live module-summary substrate cannot
      carry it honestly yet.
- [ ] Elaborate binary operator sections `(+), (x +), (+ x)` into ordinary callable surface forms when
      the operator target is resolved.
- [ ] Add a high-level expanded-surface-to-Core lowering gate so unresolved parsed-surface notation
      cannot bypass expansion accidentally.
- [ ] Record origin-sidecar decisions for notation and operator-section expansion without claiming full
      Core origin threading.
- [ ] Close out with focused parser/lowering/typeck/engine gates, docs gates, and independent review.

## Non-goals

- No full macro expander or hygiene-complete macro system.
- No typed macros.
- No binder-introducing mixfix notation.
- No generalized partial application for arbitrary mixfix patterns such as `(_ between _ and _)`.
- No full import/export notation propagation unless a bounded task proves the live summary carriers
  can support it safely.
- No general `do`, comprehension, handler, contract, trace, or full `SPEC-098c` lowering completion.
- No broad Core origin sidecar threading beyond the metadata needed to represent notation expansion
  products honestly at the surface boundary.

## Phase structure

### Phase 1: Make expansion traversal reusable

Tasks:

- TASK-1728: Create the Phase 169 plan and task packet. ✅
- TASK-1729: Add reusable surface traversal for expansion diagnostics. 📝

### Phase 2: Preserve and declare notation

Tasks:

- TASK-1730: Parse and preserve minimal notation declarations. 📝
- TASK-1731: Preserve raw built-in infix operator tokens. 📝
- TASK-1732: Build minimal local notation-table resolution diagnostics. 📝

### Phase 3: Elaborate notation before Core

Tasks:

- TASK-1733: Elaborate binary operator sections to callable surface forms. 📝
- TASK-1734: Add expanded-surface-to-Core lowering gate. 📝
- TASK-1735: Close out Phase 169 with verification and review. 📝

## Dependency graph

```text
TASK-1728
  -> TASK-1729
      -> TASK-1730
          -> TASK-1731
              -> TASK-1732
                  -> TASK-1733
                      -> TASK-1734
                          -> TASK-1735
```

## Implementation constraints

- Start from live code, especially `crates/ash-parser/src/surface.rs`,
  `crates/ash-parser/src/parse_expr.rs`, parser module/item dispatch, and `crates/ash-parser/src/lower.rs`.
- Use the Phase 168 `ParsedSurfaceModule`, `ExpandedSurfaceModule`, `SurfaceOrigin`,
  `RawOperatorToken`, and `OperatorSection` carriers instead of inventing parallel structures.
- Keep existing accepted syntax stable. New parser lookahead must not steal valid fallback parses such
  as underscore-leading identifiers.
- Any unresolved surface-only node must fail before Core lowering with a structured diagnostic or an
  existing explicit unsupported-feature error.
- Notation resolution must preserve authority: rows, capability requirements, failures, and contracts
  come from the resolved callable, not from the notation token.
- If a task cannot implement an imported/exported notation table honestly, it must record a narrow
  deferral and keep local-only resolution explicit in diagnostics and docs.

## Verification policy

Each implementation task must run focused parser or lowering tests plus formatting and relevant crate
checks. Any task that adds an `Expr` variant, definition variant, or public surface carrier must run at
least `cargo check --workspace` to catch downstream exhaustive matches.

Baseline closeout commands:

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
```

If a task touches a shared parser enum or public crate API, add `cargo check --workspace` before
status updates. If a task changes docs/spec indexes, run the docs gate in the same slice.

## Acceptance criteria

- [ ] `expand_surface_module` uses a reusable traversal API rather than one-off expression scans.
- [ ] Notation declarations are parsed into durable surface carriers with spans and raw token/pattern
      information.
- [ ] Built-in infix expressions retain raw operator spelling where it is needed for later diagnostics
      and formatting.
- [ ] Local notation declarations produce deterministic duplicate/conflict diagnostics before type
      inference.
- [ ] Binary operator sections elaborate to ordinary callable surface forms after local notation/built-in
      operator resolution.
- [ ] High-level lowering paths can require `ExpandedSurfaceModule` or an equivalent validated carrier
      before Core lowering.
- [ ] Deferred macro, imported-notation, generalized mixfix, and broad `SPEC-098c` lowering work is
      documented without overclaiming.
- [ ] `PLAN-INDEX.md`, this plan, task files, and `CHANGELOG.md` agree on Phase 169 status.
