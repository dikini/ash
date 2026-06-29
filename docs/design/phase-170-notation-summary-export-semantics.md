# Phase 170 notation summary/export semantics

## Status

Design decision for TASK-1739. Phase 170 selects **explicit non-propagation** for notation declarations across module imports/exports until module-summary carriers can transport notation metadata honestly.

## Inputs

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md` §7, §8, §10, §11.
- `docs/spec/SPEC-012-IMPORTS.md` §3-§5.
- `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`.
- `docs/audit/phase-170-expanded-surface-boundary-audit.md`.
- Live Phase 169/170 implementation in `crates/ash-parser/src/surface.rs` and `crates/ash-engine/src/module_loader.rs`.

## Decision summary

Notation declarations are source-level sugar. They do not create Core definitions, runtime authority, evidence, rows, capabilities, contracts, or importable values. The only semantic authority is the resolved callable target after ordinary visibility, type, effect, contract, and capability checks.

Phase 170 keeps the active notation table **module-local only**:

1. A notation declaration is active only in the module body where it appears.
2. Inline child modules do not inherit the parent's notation declarations.
3. Parent modules do not inherit inline child notation declarations.
4. File/module imports do not activate the imported module's notation declarations.
5. `pub` notation declarations are accepted as source syntax but are not transported in module summaries yet.
6. Re-export of notation declarations is not implemented in Phase 170.

This is intentionally narrower than SPEC-095c's target sentence that an active notation table may include imported/exported notation. The sentence remains the target model; Phase 170 documents and tests the current conservative slice rather than pretending summary propagation exists.

## Why not implement propagation now?

Current import/export carriers are value/type/callable/module-summary carriers. They do not yet encode a notation declaration as a distinct exported surface item with:

- pattern spelling and spans;
- fixity class and precedence;
- source module identity;
- declaration visibility;
- resolved target callable path and target visibility evidence;
- conflict identity for duplicate/conflicting imported declarations;
- re-export provenance.

Implementing propagation without those fields would require either reparsing source snippets at import sites or stringly attaching notation rows to existing callable summaries. Both would overclaim the module-summary contract and make later macro/notation hygiene harder.

## Visibility model

### Current Phase 170 behavior

| Declaration form | Parsed? | Active locally? | Exported/imported? | Notes |
|---|---:|---:|---:|---|
| `infixl 6 <+> = combine;` | yes | yes | no | Local source sugar only. |
| `pub infixl 6 <+> = combine;` | yes | yes | no | `pub` is preserved as declaration metadata but no summary carrier exports it. |
| notation in inline child module | yes | child only | no | Parent cannot use it. |
| notation in parent module | yes | parent only | no | Child cannot use it unless declared again locally. |
| notation in imported file module | yes in exporter | exporter only | no | Importing callables does not import notation aliases. |
| notation re-export through `pub use` | no dedicated behavior | no | no | Future design requires named notation export identities. |

### Future target behavior

If a later phase implements propagation, notation visibility should be checked independently from callable visibility:

1. The notation declaration must be visible/exported from the declaring module.
2. The target callable must also be visible to the importing module.
3. Importing notation never imports authority. The imported alias expands to a call and the call is still checked normally.
4. A notation summary must keep declaration provenance separate from target callable provenance.

A public notation whose target callable is private should be rejected before summary export, not silently exported as a broken alias.

## Conflict and precedence model

### Current Phase 170 behavior

Only local declarations in a single module participate in one table. The existing local table rules apply:

- duplicate notation declarations for the same spelling/fixity/target are accepted as one effective entry only if the implementation treats them as identical;
- conflicting declarations for the same spelling/fixity with different targets or fixity metadata are rejected;
- inline modules have independent tables and therefore do not conflict with parent tables.

No local-vs-imported or imported-vs-imported conflicts can arise in Phase 170 because imported notation is not active.

### Future target behavior

If propagation is implemented later, conflict resolution should be conservative:

| Conflict class | Future rule |
|---|---|
| local vs imported same spelling/fixity different target | reject unless an explicit shadowing/import-alias design is specified |
| imported selected vs imported selected conflict | reject before type inference |
| glob vs selected conflict | selected import wins only if SPEC-012-style import precedence is explicitly extended to notation; otherwise reject |
| glob vs glob conflict | reject on use or table construction before type inference |
| same declaration through multiple re-export paths | deduplicate by declaration identity/provenance, not by target string only |

The conservative default is rejection, not silent last-wins, because notation changes parse/association and can otherwise make source meaning depend on import order.

## Scope matrix for TASK-1740

TASK-1740 should implement and test **non-propagation**, not summary transport.

| Case | Expected Phase 170 behavior | TASK-1740 test shape |
|---|---|---|
| local notation used in same module | accepted | parser expansion positive test already exists; keep/regress |
| duplicate/conflicting local notation | existing local-table behavior | keep TASK-1732 tests |
| parent uses inline child notation | rejected | negative expansion test: parent body `(<+>)` unresolved when only child declares `<+>` |
| inline child uses parent notation | rejected | negative expansion test: child body `(<+>)` unresolved when only parent declares `<+>` |
| importing module uses notation declared in imported file | rejected | engine/module-loader negative import test or parser-level two-module fixture; callable import may work but notation alias remains inactive |
| importing module uses imported callable directly | accepted | non-interference positive: `combine(a,b)` remains importable/visible |
| `pub` notation declaration without import use | accepted locally | parser/expansion positive: `pub infixl ...` is local sugar only |
| re-exported notation via `pub use` | unsupported/no activation | negative docs/test if syntax becomes representable; otherwise explicitly deferred |

## Module-summary carrier requirements for future propagation

A future `NotationSummary` should carry at least:

```text
NotationSummary {
  declaration_id,
  declaring_module,
  visibility,
  fixity_kind,
  precedence,
  pattern_spelling,
  pattern_span_or_source_anchor,
  target_callable_path,
  target_visibility_anchor,
  export_provenance,
}
```

The summary must not be keyed only by rendered operator spelling. Two declarations may share a spelling in different modules; duplicate detection needs declaration identity and import provenance.

## Non-goals for Phase 170

- No imported/exported notation activation.
- No notation `pub use` or named notation export identity.
- No summary schema migration for notation.
- No generalized mixfix propagation.
- No authority/effect shortcut through notation.
- No Core-visible notation item.

## Consequences

- Phase 170's behavior is conservative and easy to explain: notation is local sugar.
- Users must redeclare notation in each module that wants custom operator sugar.
- Imported callables remain usable by ordinary call syntax.
- Later macro/notation phases get a clear carrier checklist instead of inheriting an accidental string-based propagation model.
