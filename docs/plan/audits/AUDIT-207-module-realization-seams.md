# AUDIT-207: Module Realization Seams

**Status:** Complete — planning baseline only; no implementation claim.
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Target rule:** [SPEC-103](../../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md)

## Finding

Ash has useful module fragments but not one complete module realization. The parser accepts file-backed and inline declarations. The resolver builds a file graph. The engine transports selected summaries and imports. The typechecker has a binder. These fragments do not yet share one AST-driven route or prove file/inline parity through Engine execution.

## Live seam inventory

| Layer | Existing seam | Required replacement or extension | Owner |
|---|---|---|---|
| Surface parsing | `crates/ash-parser/src/parse_module.rs` parses `ModuleFile` and `ModuleDecl` variants | Preserve one parsed declaration carrier and make every downstream graph edge originate from it | TASK-2057 |
| File graph | `crates/ash-parser/src/resolver.rs` discovers `mod` declarations with a line scan | Consume parsed `ModuleFile` declarations; carry parsed source and origins; reject disagreement with source scans | TASK-2057, TASK-2059 |
| Graph identity | `crates/ash-core/src/module_graph.rs` stores graph topology and file/inline source tags | Add canonical crate-qualified paths, stable interface/artifact identity, and inline source ownership | TASK-2058 |
| Imports | `crates/ash-parser/src/import_resolver.rs` and Engine loader have distinct import paths | Resolve imports from checked module interfaces, not filename walking or engine-private exports | TASK-2061 |
| Binding | `crates/ash-typeck/src/name_binding.rs` models scope precedence | Feed the binder canonical identities and checked public/private interface facts | TASK-2061 |
| Summaries | `crates/ash-core` semantic summaries and `crates/ash-engine/src/module_loader.rs` carry selected metadata | Make one versioned, export-closed module interface for all ordinary exported declarations | TASK-2060 |
| Inline modules | Parser stores inline definitions; Engine has explicit unsupported-inline guards | Construct ordinary inline module units and send them through the same check/interface/lowering route as files | TASK-2059, TASK-2060 |
| Lowering | `crates/ash-engine` has bounded module-file and callable routes | Lower resolved checked module definitions to Core and CPS with identity/origin preservation | TASK-2062 |
| Entry/runtime | PLAN-203 requires one Engine-owned route | Link reachable module artifacts before admission; prove CLI/daemon terminal parity | TASK-2063, TASK-2064 |

## Retired semantic authority

The following are not valid semantic authorities for the completed system:

- line-oriented discovery of `mod name;` after a `ModuleFile` has been parsed;
- snippet extraction of declarations, exports, or imports from source text;
- Engine-private export tables as the definition of language visibility;
- filesystem walking initiated by `use` rather than a resolved module identity;
- a direct evaluator fallback when a linked checked module artifact is absent.

A compatibility seam may persist during migration only if it compares against the AST result, fails closed on disagreement, and cannot publish semantic facts.

## Complete semantic-scanner inventory and retirement gate

This inventory is the planning baseline for the prohibition in SPEC-103 §5. It covers every
known production path that currently discovers a module/import/export fact from source text after
an authoritative parse is available. TASK-2057 through TASK-2061 must update this table whenever
a call site is removed, replaced, or newly found.

| Scanner or text-derived seam | Current production caller/authority | Replacement owner | Completion criterion |
|---|---|---|---|
| `ModuleResolver::parse_module_decls` in `crates/ash-parser/src/resolver.rs` | Creates file-child graph edges from line scanning | TASK-2057 | Graph edges originate only from parsed `ModuleDecl` nodes; comments, strings, and malformed text cannot create edges. |
| `strip_module_metadata_non_definition_lines` in `crates/ash-engine/src/module_loader.rs` | Masks `use`/`mod` before selected metadata lowering | TASK-2059, TASK-2060 | Expanded parsed module items feed collection directly; no masked text becomes semantic input. |
| leading-import line accumulation and `import_needs_more_lines` in `crates/ash-engine/src/module_loader.rs` | Builds Engine loader import prelude from text | TASK-2061 | Parsed `use` items and interface bindings replace the prelude reader. |
| `source_scan::{extract_pub_mod_declarations, extract_semicolon_snippets}` | Finds public modules/imports/exports by prefixes and snippets | TASK-2060, TASK-2061 | Export and re-export facts come only from expanded AST/interface traversal. |
| `collect_module_exports` text supplements in `crates/ash-engine/src/module_loader.rs` | Adds public capabilities, builtins, functions, child modules, imports, and re-exports after partial parsing | TASK-2060 | One checked interface collects all exported namespaces; no supplement scans are semantic authority. |
| path/string-keyed Engine module caches and raw path walking | Selects import/export identity by filesystem/path strings | TASK-2058, TASK-2063 | Canonical `ModuleKey` and checked interface/artifact identity are the sole semantic/cache keys. |

TASK-2065 must run a repository-wide scanner denylist/allowlist check. Any remaining raw scanner
must be explicitly listed as test-only or disagreement-only, must fail closed on disagreement, and
must have no path to graph construction, binding, interface publication, lowering, admission, or
execution. An unclassified production scanner blocks phase closeout.

## Required rule families

| ID | Rule | Current status | Planned owner |
|---|---|---|---|
| MOD-REAL-001 | A module declaration creates one stable child identity and structural edge from parsed AST | partial / tested / below_spec | TASK-2057, TASK-2058 |
| MOD-REAL-002 | File and inline sources produce equivalent module units after acquisition | not_implemented / none / below_spec | TASK-2059 |
| MOD-REAL-003 | Checked public interfaces are export-closed and preserve defining identities | partial / tested / below_spec | TASK-2060 |
| MOD-REAL-004 | Imports and visibility resolve only through interfaces | partial / tested / below_spec | TASK-2061 |
| MOD-REAL-005 | Resolved modules lower to linked Core/CPS artifacts without source rediscovery | not_implemented / none / below_spec | TASK-2062 |
| MOD-REAL-006 | Engine admission and CLI/daemon execution use the same linked module artifact | not_implemented / none / below_spec | TASK-2063, TASK-2064 |

## Non-goals

This phase does not add dynamic imports, package discovery, import-cycle initialization, hot reload, runtime module values, or a full incremental workspace database. Structural and import cycles reject in the initial realization.
