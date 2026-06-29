# PLAN-170: Expanded Surface Integration and Notation Scoping

## Status: 🟢 In Progress; notation non-propagation complete

## Overview

Phase 170 closes the highest-value deferrals left by Phase 169 around the expanded-surface boundary and notation scoping. Phase 169 made local notation declarations, binary operator-section elaboration, raw operator-token preservation, reusable traversal, and expanded-surface gates real. Phase 170 should turn that substrate into a stronger architectural seam by auditing and routing high-level lowering/module-loader paths through expansion, deciding the conservative notation summary/export contract, implementing bounded notation propagation only if the carriers can do it honestly, and specifying source-origin sidecars for expansion products.

The phase deliberately avoids full macro hygiene, typed macros, generalized binder-introducing mixfix notation, and broad `SPEC-098c` lowering completion. Its goal is to make the current boundary harder to bypass and make notation scope/export behavior explicit enough for later macro work.

## Source specs and prior artifacts

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- `docs/audit/phase-168-surface-to-core-lowering-inventory.md`

## Goals

- [x] Audit all public parser/lowering/module-loader paths that can still accept parsed-surface structures directly.
- [x] Route high-level module/file lowering paths through expanded-surface validation where safe.
- [ ] Keep low-level parser/test helpers available but explicitly fail-closed for unresolved surface-only nodes.
- [x] Specify notation declaration summary/export semantics, including visibility and import behavior.
- [x] Implement bounded imported/exported notation propagation only if module-summary carriers support it cleanly; otherwise record and test explicit non-propagation.
- [ ] Specify source-origin sidecar threading for notation and operator-section expansion products without claiming full Core provenance if it is not wired.
- [ ] Close out with focused parser/lowering/typeck/engine gates, docs gates, and independent review.

## Non-goals

- No full macro expander or hygiene-complete macro system.
- No typed macro system.
- No binder-introducing/generalized mixfix partial application.
- No broad `SPEC-098c` lowering completion for every surface form.
- No semantic authority attached to notation declarations; authority remains on resolved callables.
- No public imported-notation claim unless implemented with positive import and negative leakage tests.

## Decision gates

| Gate | Question | Tier | Blocks | Default |
|---|---|---|---|---|
| D1 | Which public lowering/module-loader APIs must require `ExpandedSurfaceModule` now? | T1 | TASK-1738 | Route only high-level boundaries; leave low-level test helpers documented. |
| D2 | Can module summaries carry notation declarations without overclaiming import/export behavior? | T1/T2 if summary schema changes broadly | TASK-1740 | Local-only plus explicit non-propagation tests if carriers are not ready. |
| D3 | Is source-origin sidecar metadata surface-only or Core-visible in this phase? | T2 if Core API changes | TASK-1741 | Design only unless a narrow surface-side carrier can be added safely. |

## Phase structure

### Phase 1: Register and audit the boundary

Tasks:

- TASK-1736: Create the Phase 170 plan and task packet. ✅
- TASK-1737: Audit expanded-surface boundary and direct-lowering call sites. ✅

### Phase 2: Enforce safe high-level expansion

Tasks:

- TASK-1738: Route high-level module/file lowering through expanded-surface validation. ✅

### Phase 3: Scope notation across modules

Tasks:

- TASK-1739: Specify notation summary/export and visibility semantics. ✅
- TASK-1740: Implement bounded notation import/export propagation or explicit non-propagation. ✅

### Phase 4: Preserve origin and close out

Tasks:

- TASK-1741: Specify and implement the narrow source-origin sidecar boundary for expansion products. 📝
- TASK-1742: Close out Phase 170 with verification, changelog, index reconciliation, and review. 📝

## Dependency graph

```text
TASK-1736
  -> TASK-1737
      -> TASK-1738
      -> TASK-1739
          -> TASK-1740
      -> TASK-1741
          -> TASK-1742
```

TASK-1738 and TASK-1739/1740 can proceed after TASK-1737 if D1 and D2 are resolved independently. TASK-1742 depends on all implementation and design tasks.

## Implementation constraints

- Start from live code in `crates/ash-parser/src/lower.rs`, `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`, module loading call sites in `crates/ash-engine`, and downstream typeck/lsp consumers.
- Preserve accepted syntax and existing low-level tests unless the task explicitly marks a parser-only bypass as invalid.
- Every unresolved parsed-surface-only node must fail before Core lowering with a structured or existing explicit unsupported diagnostic.
- Notation scope must be conservative. Inline-module, parent-module, import, and export behavior need both positive visibility tests and negative leakage tests.
- Notation never grants authority. Rows, failures, contracts, capabilities, and admission remain properties of the resolved callable after ordinary type/lower/runtime checks.
- Do not claim Core origin/provenance threading unless the task actually updates the relevant Core APIs and tests them.

## Verification policy

Each task must run its focused tests plus formatting and relevant crate checks. Any task changing public surface carriers, module summaries, lowering APIs, or downstream consumers must run `cargo check --workspace`.

Baseline closeout commands:

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
cargo check --workspace
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

## Expected follow-on after Phase 170

If Phase 170 closes cleanly, the next plausible packet is full macro/notation hygiene or generalized mixfix sections. If D2 shows summary carriers are not ready, the follow-on should first harden module summaries before macro work.
