# PLAN-172: Parser-First Macro Execution MVP

## Status: ✅ Complete

## Overview

Phase 172 is the first intentionally executable macro slice after Phase 171's fail-closed macro invocation boundary. It remains conservative: the MVP executes only local, parser-first, expression-position macros whose arguments and template body parse as ordinary Ash surface expressions. It does not add token-tree rewriting, typed macros, binder-introducing macros, imported/exported macro activation, or Core/runtime macro constructs.

The goal is to replace a narrow class of `MacroInvocation` carriers with expanded surface expressions before notation resolution and before Core lowering, while preserving Phase 171 hygiene invariants: stable expansion identity, parent-origin chains, generated/source separation, and fail-closed high-level validation for every unsupported macro shape.

## Source specs and prior artifacts

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/audit/phase-171-hygiene-origin-scope-audit.md`

## Goals

- [x] Create this Phase 172 plan and task packet.
- [x] Audit the live `MacroInvocation`, expansion, parsing, module-loader, and typechecker seams before implementing execution.
- [x] Amend specs with a parser-first expression-macro MVP that is honest about syntax, scope, and non-goals.
- [x] Add macro declaration surface carriers without executing macros.
- [x] Parse macro invocation arguments as structured expression lists only for supported delimiters/shapes.
- [x] Build a local-only macro registry with fail-closed duplicate, missing, imported, and re-export behavior.
- [x] Execute only whitelisted expression-template macros with arity checking, source-origin preservation, and explicit unsupported-template diagnostics.
- [x] Validate parser, engine/module-loader, typechecker, and lowering boundaries with positive execution tests and negative leakage tests.
- [x] Close out with broad verification, docs/status reconciliation, changelog, and focused review.

## Non-goals

- No full macro expander.
- No typed macros or macro type inference.
- No arbitrary token-tree rewriting.
- No macro-by-example/rules system.
- No procedural macros or host-language callbacks.
- No binder-introducing macros, macro-generated definitions, or macro-generated modules.
- No imported/exported macro activation or macro summary carriers.
- No Core/runtime macro representation.
- No authority, row, contract, failure, proof, or evidence effects introduced by macro metadata.
- No generalized mixfix/binder notation.

## MVP syntax model

The planned MVP uses a deliberately small declaration form, subject to TASK-1752/TASK-1753 audit/spec confirmation:

```ash
macro inc(x) => add(x, 1);
macro pair(a, b) => Tuple::new(a, b);

fn example(n: Int) -> Int {
  inc!(n)
}
```

MVP properties:

1. `macro` is a module-level parsed surface declaration.
2. Macro names are unqualified ordinary names in the declaring module.
3. Invocations are expression-position `name!(...)`, with parenthesized expression arguments only for the executable MVP.
4. Bracketed/braced invocations remain parsed for diagnostics but fail closed unless a task explicitly implements them.
5. Template bodies are parsed Ash expressions and expanded by substituting parameter-name variable occurrences with invocation argument expressions; free template variables are rejected rather than resolved at the call site.
6. Template bodies that introduce binders or definitions fail closed in the MVP unless a task proves hygienic binder handling.
7. Expansion output re-enters the existing notation/operator-section expansion pass and Core never sees macro syntax.

## Decision gates

| Gate | Question | Tier | Blocks | Default |
|---|---|---|---|---|
| D1 | Is `macro name(args) => expr;` acceptable as the MVP declaration syntax, or should the phase only implement a carrier/audit? | T1 | TASK-1753+ | Accept only after spec patch and parser tests. |
| D2 | Which template expression variants are safe without binder hygiene? | T1 | TASK-1756 | Whitelist literals, variables, calls, field/index, unary/binary, constructor/list if traversal proves no binders. Reject binder/block/control forms. |
| D3 | Should bracket/brace invocations execute? | T1 | TASK-1754/TASK-1756 | No; keep bracket/brace fail-closed until real token-tree parsing exists. |
| D4 | Can macro declarations be imported or re-exported? | T2 | TASK-1755/TASK-1758 | No; local module only, with negative import/export tests. |
| D5 | How should macro expansion origins compose with notation expansion origins? | T1 | TASK-1756/TASK-1758 | Use Phase 171 origin sidecars: macro expansion parent origin then notation/operator-section child origins. |

## Phase structure

### Phase 1: Planning and audit

- TASK-1751: Create the Phase 172 parser-first macro execution MVP plan packet. ✅
- TASK-1752: Audit macro execution seams and define the safe MVP subset. ✅

### Phase 2: Spec and parsed carriers

- TASK-1753: Amend macro specs for parser-first expression macro MVP. ✅
- TASK-1754: Add parsed macro declaration and structured invocation-argument carriers. ✅

### Phase 3: Local registry and execution

- TASK-1755: Add local macro registry and scope-boundary validation. ✅
- TASK-1756: Implement fail-closed expression-template macro expansion. ✅
- TASK-1757: Preserve macro expansion origin/hygiene metadata through notation expansion. ✅

### Phase 4: Cross-boundary validation and closeout

- TASK-1758: Add cross-boundary macro execution and negative-leakage tests. ✅
- TASK-1759: Close out Phase 172 with verification, review, and status reconciliation. ✅

## Dependency graph

```text
TASK-1751
  -> TASK-1752
      -> TASK-1753
          -> TASK-1754
              -> TASK-1755
                  -> TASK-1756
                      -> TASK-1757
                          -> TASK-1758
                              -> TASK-1759
```

## Implementation constraints

- Start from live code in `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_expr.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/lower.rs`, `crates/ash-engine/src/module_loader.rs`, and typechecker validation paths that currently see `Expr::MacroInvocation`.
- Reuse `MacroInvocation`, `SurfaceOrigin::MacroExpansion`, `ExpansionId`, `ExpandedSurfaceOrigin`, `expand_surface_module`, and expression traversal helpers from Phase 171.
- Keep all macro execution before notation/operator-section resolution and before Core lowering.
- Unsupported macro declarations, invocations, delimiters, arity mismatches, imported macro attempts, duplicate macro names, and unsafe template forms must produce explicit diagnostics and fail closed.
- Macro expansion cannot grant capability authority, latent rows, failures, contracts, or proof/evidence obligations. Those remain properties of the expanded ordinary expression and downstream typechecking.
- Any new public surface carriers require `cargo check --workspace` and explicit downstream consumer updates.

## Baseline closeout commands

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

## Acceptance criteria

- [x] Specs describe the executable MVP and all fail-closed non-goals without overclaiming full macro expansion.
- [x] Macro declarations parse into source-preserving surface carriers.
- [x] Supported `name!(expr, ...)` invocations expand before notation/operator-section resolution.
- [x] Unsupported macro forms continue to fail before Core lowering and public export acceptance.
- [x] Macro expansion products carry stable expansion IDs and origin chains suitable for diagnostics.
- [x] Macro-generated identifiers cannot silently capture source identifiers; binder-introducing templates are rejected or proven safe by tests.
- [x] Imported/re-exported macro activation is not accepted unless a task adds real summary carriers and positive/negative tests; the default Phase 172 plan keeps it rejected.
- [x] Parser, engine/module-loader, typechecker, and lowering tests cover both positive MVP execution and negative leakage paths.
- [x] PLAN-INDEX, this plan, task files, and CHANGELOG agree on Phase 172 status.

## Packet creation evidence

Created in TASK-1751. Structural verification must prove that this plan references TASK-1751 through TASK-1759, every task file exists exactly once, `PLAN-INDEX.md` has the Phase 172 row and section, and `CHANGELOG.md` records the planning packet under `[Unreleased]`.

## TASK-1752 evidence

`docs/audit/phase-172-macro-execution-mvp-audit.md` records the live Phase 171 macro carrier, parser, expansion, module-loader, lowering, and typechecker seams. It freezes Phase 172 execution to local parenthesized expression-position macros only, keeps bracket/brace/qualified/imported/binder/typed/token-tree forms fail-closed, classifies every current `Expr` variant for template safety, and maps downstream task/file ownership.

## TASK-1753 evidence

`SPEC-095c` now states the parser-first expression macro MVP grammar (`MacroDecl` plus parenthesized `name!(ExprList?)` invocations), local-only scope, authority-neutral syntax substitution, fail-closed unsupported forms, and origin-chain requirements. `SPEC-098c` now states that supported local macros must expand before Core while unsupported macro declarations/invocations reject before Core/export/typecheck acceptance. `SPEC-INDEX.md` references PLAN-172 for both specs.

## TASK-1754 evidence

`crates/ash-parser/src/surface.rs` now has `Definition::Macro` and `MacroDef` carriers. `crates/ash-parser/src/parse_module.rs` parses module-level `macro name(params) => expr;`, and `crates/ash-parser/src/parse_expr.rs` preserves structured expression arguments for executable parenthesized macro invocations while leaving bracket/brace invocations as non-executable diagnostic carriers. `crates/ash-parser/tests/task_1754_macro_declaration_parse.rs` covers declaration parsing, structured parenthesized args, bracket/brace non-executable carriers, and qualified-path rejection.

## TASK-1755 evidence

`crates/ash-parser/src/surface.rs` now builds local macro tables, rejects duplicate declarations, and classifies invocation failures as unknown, unsupported MVP syntax, or deferred execution. `crates/ash-parser/tests/task_1755_macro_registry_scope.rs` covers local table visibility, duplicates, missing macros, unsupported bracket/brace forms, structured-argument requirements, and macro declarations crossing the expanded-surface boundary without callable export semantics. `crates/ash-engine/tests/task_1755_macro_registry_scope.rs` verifies `pub macro` declarations are not imported as callables and imported macros do not activate in callers.

## TASK-1756 evidence

`crates/ash-parser/src/surface.rs` now expands local parenthesized expression macros before notation/operator-section elaboration, substitutes macro params in whitelisted expression templates, rejects arity mismatches and unsupported templates explicitly, and enforces a bounded recursion/depth diagnostic. `crates/ash-parser/tests/task_1756_expression_macro_expansion.rs` covers successful call expansion, fail-closed errors, recursive depth diagnostics, binary substitution, and macro output re-entering notation expansion. TASK-1755 registry tests were updated for the now-executable local macro path.

## TASK-1757 evidence

Macro expansion now threads local notation tables through template expansion so nested macro expansions and notation/operator sections produced inside macro products are elaborated immediately with `SurfaceOrigin::MacroExpansion` as parent metadata. Macro declaration bodies remain source templates until invocation; expanded-surface residual-carrier checks skip those declarations and reject only executable residual carriers. `crates/ash-parser/tests/task_1757_macro_origin_hygiene.rs` verifies macro origin sidecars, nested macro parent origins, nested notation parent origins, free-template-variable rejection, generated helper-name fencing, and fail-closed unsupported templates without Core/runtime provenance schema changes.

## TASK-1758 evidence

`crates/ash-engine/tests/task_1758_macro_execution_boundaries.rs` and `crates/ash-parser/tests/task_1758_macro_lowering_boundaries.rs` validate the Phase 172 macro execution MVP across high-level engine/module checks, ordinary callable import/export collection, parser high-level lowering, and direct expanded-surface lowering gates. The tests prove local supported macro execution works while imported, missing, unsupported, and manually injected raw macro carriers fail closed before semantic/Core acceptance.

## TASK-1759 evidence

Closeout reconciled PLAN-172, PLAN-INDEX, task files, specs, and CHANGELOG. Focused closeout review found two blockers before final closeout: free template variables could capture call-site bindings, and nested macro expansion origins were not parented to the outer macro origin. The remediation rejects non-parameter template variables, preserves macro-to-macro parent origins, substitutes nested macro invocation arguments, keeps macro declaration bodies as source templates, and updates raw-carrier diagnostics to describe unexpanded macro carriers rather than unimplemented macro expansion. Focused and broad gates were rerun after remediation.
