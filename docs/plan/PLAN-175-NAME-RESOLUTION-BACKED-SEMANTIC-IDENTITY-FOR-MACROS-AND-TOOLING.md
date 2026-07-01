# PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling

**Status:** ✅ Complete (10/10 tasks complete)
**Spec:** [SPEC-095c: Surface AST, Macro Expansion, and Notation](../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md); [SPEC-038: Language Server](../spec/SPEC-038-LANGUAGE-SERVER.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
**Depends on:** Phase 174 macro-aware tooling, summary identity, and bounded callable identity inference readiness.
**Task range:** TASK-1784 through TASK-1793.

## Goal

Introduce a conservative, name-resolution-backed semantic identity substrate for macro declarations and callable references so parser/LSP tooling can distinguish macro, function, and imported-summary identities without granting macros runtime callability.

## Rationale

Phase 174 made macros visible to tooling and added lightweight summary keys, but same-file references remain token-only and imported macro navigation is intentionally not overclaimed. Phase 175 should replace lexical macro/function splitting with resolved semantic identity while preserving the syntax-phase-only macro boundary that Phase 173 and Phase 174 established.

This phase is the substrate before richer macro inference or hygiene work: identifiers need stable declaration identity first, and LSP consumers need honest semantic references before cross-file navigation can be expanded.

## Scope

Phase 175 owns:

- canonical macro declaration identity design and task-local carriers;
- resolved macro/callable identity threading through parser-facing and LSP-facing summaries;
- same-file semantic reference splitting for macros versus ordinary callables;
- cross-file imported macro navigation preparation through summary identities, without claiming full workspace name resolution;
- validation that macro identities remain syntax-phase metadata and never become runtime callable authority;
- spec/docs/status reconciliation for the new identity substrate.

## Non-goals

- No runtime macro callability, Core macro forms, effect rows, contracts, provider authority, or proof evidence.
- No full workspace module graph or project-wide incremental name-resolution database.
- No generalized hygienic macro semantics beyond identity carriers needed by tooling.
- No generalized mixfix, imported notation activation, or notation-summary identity work.
- No speculative type inference through unresolved, overloaded, private, or module-qualified paths beyond identities proven by this phase.
- No cross-file references that require source roots or dependency graph semantics not explicitly modeled by this phase.

## Current context

Phase 174 closed with:

- macro-aware completion, hover, document symbols, and same-file macro-invocation goto;
- `ParseSummary` macro summary keys including parameter names, typed signature shape, and template fingerprints;
- bounded callable identity inference for unique public local `fn`/`builtin fn` summaries;
- explicit documentation that same-file references remain token-only/non-semantic.

The remaining debt is identity resolution rather than presentation:

- `crates/ash-lsp-core/src/goto.rs` can distinguish `m!(...)` from `m()` for goto, but reference scans are lexical.
- `crates/ash-lsp-core/src/hover.rs` and symbol summaries can present macro metadata, but do not share a canonical resolved identity carrier.
- `crates/ash-parser/src/surface.rs` has macro summaries and callable summaries, but no stable macro declaration identity that downstream consumers can compare.
- `crates/ash-engine/src/module_loader.rs` transports macro summaries as syntax-phase metadata, but imported macro navigation has no identity contract beyond names/aliases.

## Decision gates

| Gate | Question | Owner task | Default decision |
|---|---|---|---|
| D1 | What identity carriers already exist for declarations, imports, summaries, and LSP symbols? | TASK-1785 | Audit before adding new IDs. |
| D2 | What is the canonical macro declaration identity shape? | TASK-1786 | Use a stable syntax-phase identity separate from callable identity. |
| D3 | Where should parser-local name resolution produce resolved macro/callable references? | TASK-1787 | Same parsed module/file first; no workspace graph. |
| D4 | Which LSP summaries should carry resolved identity without storing full AST? | TASK-1788 | Thread compact identity keys, not full declaration payloads. |
| D5 | Can same-file references become semantic without overclaiming cross-file support? | TASK-1789 | Yes: split same-file macro/function refs by resolved identity only. |
| D6 | What imported macro identity is safe for navigation preparation? | TASK-1790 | Summary identity only; no runtime callable or full workspace references. |
| D7 | What negative tests prove identity does not grant callability? | TASK-1791 | Parser/engine/LSP cross-boundary non-leakage tests. |

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1784](tasks/TASK-1784-phase-175-plan-packet.md) | Create the Phase 175 semantic-identity plan packet | ✅ Complete |
| [TASK-1785](tasks/TASK-1785-identity-surface-audit.md) | Audit macro/callable identity surfaces and current name-resolution seams | ✅ Complete |
| [TASK-1786](tasks/TASK-1786-canonical-macro-identity-model.md) | Define canonical macro declaration identity and callable identity boundaries | ✅ Complete |
| [TASK-1787](tasks/TASK-1787-parser-local-name-resolution-identities.md) | Add parser-local resolved macro/callable identity carriers | ✅ Complete |
| [TASK-1788](tasks/TASK-1788-lsp-summary-identity-threading.md) | Thread resolved identities through LSP parse and symbol summaries | ✅ Complete |
| [TASK-1789](tasks/TASK-1789-semantic-same-file-references.md) | Replace token-only same-file references with semantic macro/function splitting | ✅ Complete |
| [TASK-1790](tasks/TASK-1790-imported-macro-navigation-prep.md) | Prepare imported macro navigation via summary identities without overclaiming | ✅ Complete |
| [TASK-1791](tasks/TASK-1791-identity-non-callability-validation.md) | Validate identity threading does not make macros runtime-callable | ✅ Complete |
| [TASK-1792](tasks/TASK-1792-phase-175-docs-spec-reconciliation.md) | Reconcile specs, docs, indexes, and changelog for Phase 175 | ✅ Complete |
| [TASK-1793](tasks/TASK-1793-phase-175-closeout.md) | Close out Phase 175 with broad gates and independent review | ✅ Complete |

## Implementation order

1. TASK-1785 audits live identity and resolution surfaces and patches downstream tasks if current code differs from this plan.
2. TASK-1786 defines the canonical identity model and non-callability invariants before Rust implementation.
3. TASK-1787 adds parser-local resolved identity carriers.
4. TASK-1788 exposes compact identity keys to LSP summaries/indexes without storing full ASTs.
5. TASK-1789 uses resolved identity to replace token-only same-file macro/function reference splitting.
6. TASK-1790 prepares imported macro navigation by summary identity only, with honest unsupported cases.
7. TASK-1791 validates parser/engine/LSP boundaries and negative leakage cases.
8. TASK-1792 reconciles specs/docs/status/changelog.
9. TASK-1793 runs broad gates, obtains independent review, and closes the phase.

## Acceptance criteria

- [x] Canonical macro declaration identity is defined separately from callable identity.
- [x] Parser-local macro/callable references can carry resolved identity where resolution is proven.
- [x] LSP summaries and symbol indexes can compare compact identities without full AST retention.
- [x] Same-file references distinguish `m!(...)` macro uses from `m()` callable uses by semantic identity, not token spelling.
- [x] Imported macro navigation is prepared through summary identities but does not claim full workspace references.
- [x] Macro identities remain syntax-phase metadata and do not create runtime callable bindings.
- [x] Negative tests cover same-name macro/function collisions, aliases, private macros, imported summaries, and unresolved identifiers.
- [x] PLAN-INDEX, task files, specs/docs, and CHANGELOG agree on Phase 175 status including final closeout.

## Verification baseline

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-engine
cargo test -p ash-lsp-core
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

## Expected follow-on after Phase 175

If Phase 175 closes cleanly, the next plausible packets are richer macro inference over resolved identities, hygienic binder identity hardening, or cross-file workspace/module-graph-backed macro navigation. Generalized mixfix/notation summary identity should remain separate unless Phase 175 reveals shared carriers that are already proven safe.

## Completion evidence

- TASK-1784 created and registered this Phase 175 planning packet, task files TASK-1784 through TASK-1793, PLAN-INDEX entries, and a CHANGELOG planning entry.

- TASK-1785 through TASK-1792 implemented canonical syntax-phase macro identities, callable identity separation, parser-local identity helpers, LSP compact identity keys, semantic same-file macro/callable reference splitting, imported alias identity preparation, non-callability validation, and spec/changelog reconciliation.

- TASK-1793 closed Phase 175 after broad gates and independent review remediation.
