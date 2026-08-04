# TASK-2068 Scoped Glob Ordinary-Function Imports Evidence Promotion

> **Evidence scope:** Promote only the delivered M-GLOB Type-layer evidence. This is historical
> evidence for TASK-2068's completed foundation; Phase 207 remains In progress.

**Goal:** Record the delivered, tightly bounded Type-only binding route for inherited glob imports of ordinary functions.

**Authority:** SPEC-103 §6 (resolution order and visibility), §8 (`M-IMPORT-EDGE`,
`M-IMPORT-CYCLE`, and `M-BIND`), and §9 properties 3, 5, and 7, under MOD-REAL-004 /
SEM-MODULE-REALIZATION-004.

## Semantic accounting

M-GLOB is `partial / tested / below_spec`. Type and verification are `partial`; Core, CPS, and
admission/runtime are `not_applicable`; run-route impact is `prerequisite`. This bounded result
does not complete TASK-2068 or Phase 207, establish a proof, publish a final interface, alter
generic-binder authority, or establish Core/CPS/Engine/admission/runtime/client parity.

## Delivered boundary

The dedicated resolver
`resolve_scoped_glob_ordinary_function_imports_with_scopes(graph, scopes)` and binding-only
projection `bind_scoped_glob_ordinary_function_imports(graph, scopes)` admit only inherited
`use crate::<public structural-child>...::*` routes. The importer has exactly one `use` and zero
local ordinary functions. The path traverses public structural children and the target contains
only visible public ordinary functions.

The route consumes TASK-2067 graph units, parser-owned full `Use::span`, and TASK-2068
provisional scopes. It retains defining identity, declaration origin/span/visibility, and the full
use span for every selected function; it stages one cross-module edge per function before
atomically publishing an opaque plan and its binding projection. It does not choose a
local/explicit/glob precedence rule.

The source shape matrix has 15 valid parser representations. A leading `::` is not
`UsePath::Glob`; a private structural module is an `Inaccessible` visibility case, not an
unsupported-shape case. `self`, root/repeated `super`, non-`crate` paths, multiple globs, local
declarations, explicit/group imports, aliases, re-exports or `pub use`, non-function namespaces,
and all remaining import forms remain deferred.

## Boundary mutation interpretation

CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY are boundary mutation evidence only.
A local function returns `Unsupported` at the zero-local-function boundary; a second glob returns
`Unsupported` at the exactly-one-use boundary; and the cycle-shaped attempted program returns the
same boundary `Unsupported`. Each returns no plan and no bound set. These witnesses do not claim
`LocalDeclarationCollision`, `DuplicateBinding`, generic ambiguity, `ImportCycle`, a precedence
rule, or an in-domain cycle. Defensive planner collision/duplicate/cycle branches remain
unclaimed.

## Delivered test anchors

All ten nodes below are `tested` in
`crates/ash-typeck/tests/task_2068_scoped_glob_ordinary_function_imports.rs`.

| Canonical node | Source anchor | Evidence class |
| --- | --- | --- |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-POSITIVE` | `scoped_glob_imports_two_public_ordinary_functions` | positive |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-IDENTITY` | `scoped_glob_import_plan_and_binder_preserve_each_function_identity_and_full_use_provenance` | positive |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-VISIBILITY-DIAGNOSTIC` | `scoped_glob_imports_report_private_structural_and_function_visibility_before_any_binding` | negative |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-SHAPE-DIAGNOSTIC` | `scoped_glob_imports_reject_unsupported_shapes_atomically` | negative |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-CONFLICT-ATOMICITY` | `scoped_glob_imports_reject_conflict_atomically` | boundary mutation |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-AMBIGUITY-ATOMICITY` | `scoped_glob_imports_reject_ambiguous_candidate_attempt_atomically` | boundary mutation |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-CYCLE-ATOMICITY` | `scoped_glob_imports_reject_cycle_shaped_boundary_attempt_atomically` | boundary mutation |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-FILE-INLINE-PARITY` | `scoped_glob_imports_match_file_and_inline_scope_facts` | positive |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-PROPERTY` | `scoped_glob_imports_generated_depth_count_visibility_and_source_forms` | positive; 16 cases across child depth, function count, function/path visibility, and inline/file-backed form |
| `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-AUTHORITY-FENCE` | `scoped_glob_import_route_has_only_dedicated_binding_authority_and_no_later_layer_path` | negative |

Tests are evidence, not proof or client-parity evidence.

## Source traceability

- Implementation node `IMPL-MODULE-SCOPED-GLOB-ORDINARY-FUNCTION-IMPORTS` is implemented at
  `crates/ash-typeck/src/canonical_structural_module_binder.rs#projects-scoped-glob-ordinary-function-imports-into-binding-only-facts`:
  `sha256:6fd37ea25cf3aa6767b9c2175a57f3761cf947d7a23bdf4020fff653ab250aa9`.
- Scoped planner: `sha256:568bb73d47f3b96633b256a857dc606ac868ef18bd314e07968b85a9b8f795e9`.
- `lib.rs` export boundary: `sha256:8dfaa8852bdbc697b00f5d509e9359f687284e2d502fdd918c695c8e5bc5ddd1`.
- Unchanged generic binder: `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.

The imported `sha2` dev-dependency supports portable test-only authority-fence hashing; it is not
a broader dependency claim.

## Handoffs and next obligation

M-GLOB consumes TASK-2067 canonical graph units and TASK-2068 provisional scopes. It produces
only Type-layer opaque import-plan, binding-projection, and per-function-edge facts. It grants no
final-interface or execution authority. TASK-2069 owns lowering and Engine transport; TASK-2063
owns admission; TASK-2064 owns file/inline and client parity.

**Next obligation:** Extend the canonical graph-only Type-layer slice beyond delivered M-GLOB,
M-SUPER, M-GROUP parser-span/resolver/binder, M-SIMPLE, dedicated scope-backed structural binder,
scoped structural import-cycle gate, canonical provisional-module-scope/structural-path visibility,
direct-public primitive re-export interface, and local-binding root-client fragments to every
required namespace, remaining definition/body and export-closure check, every remaining parsed
import/visibility/alias/re-export/cycle rule, and atomic M-BIND publication; TASK-2069 then owns
complete lowering and Engine transport fencing, and TASK-2064 owns integration parity.

## Documentation-only completion constraints

This evidence promotion changes no Rust, tests, Cargo/dependency policy, changelog, task status,
phase status, or git state. It promotes no deferred rule beyond the M-GLOB nodes and makes no
claim for defensive branches outside the admitted boundary.
