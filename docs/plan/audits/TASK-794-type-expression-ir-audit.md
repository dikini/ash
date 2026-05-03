# TASK-794 Audit: Type-Expression IR and Kinding Gate

## Purpose

Freeze the live Phase 110 starting point before canonical projection and kind/arity substrate work. This audit records the exact parser/core/typechecker seams that SPEC-058 and PLAN-106 were allowed to change, plus the work explicitly deferred to later packets.

## Sources Reviewed

- `docs/design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md`
- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-035-ASSOCIATED-TYPES.md`
- `docs/spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md`
- `docs/spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md`
- `crates/ash-parser/src/parse_type_def.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/lower.rs`
- `crates/ash-core/src/ast.rs`
- `crates/ash-core/src/lib.rs`
- `crates/ash-core/src/semantic_summary.rs`
- `crates/ash-typeck/src/types.rs`
- `crates/ash-typeck/src/type_env.rs`
- `crates/ash-typeck/src/kind.rs`
- `crates/ash-typeck/src/lib.rs`

## Live Contradictions At Phase-110 Start

1. Shared kind ownership was split.
   - DESIGN-034 / SPEC-058 require one shared `Kind` carrier owned by `ash-core`.
   - The live implementation still had effective kinding logic centered in `ash-typeck`, which was too local for a computation-grade IR contract.
   - Resolution gate: Phase 110 must re-home shared `Kind` ownership into `ash-core` before canonical IR rollout.

2. Parser ordinary-type paths were not an explicitly frozen pair.
   - `crates/ash-parser/src/parse_type_def.rs` and `crates/ash-parser/src/parse_module.rs` both participate in ordinary type-expression handling, but Phase 110 needed them treated as one parity boundary.
   - Resolution gate: TASK-797 must own both supported associated-projection acceptance and explicit rejection evidence for deferred syntax.

3. Associated projections were still represented through stringly / non-canonical seams in the typechecker.
   - The live `ash-typeck` substrate still centered computation-relevant projection handling around surface-like names rather than canonical interface/member identity keys with ordered argument spines.
   - Resolution gate: Phase 110 must introduce identity-backed canonical projection lowering before claiming computation-grade internal IR.

4. Source/import interface-member identity plumbing was incomplete as a computation substrate.
   - SPEC-057 had already created reserved identity carriers, but they were not yet the fully active lookup keys for canonical projection elaboration across source-local and imported summaries.
   - Resolution gate: TASK-798 must make `TypeEnv` own registry/storage/registration for interface/member identities before TASK-800 replaces live projection consumers.

5. Kind/arity validation existed only as a partial foothold.
   - The repo already had nominal arity checks and partial kind structure, but not the explicit early gate needed for future computation heads.
   - Resolution gate: TASK-799 must harden nominal/projection validation before later packets consume canonical type expressions.

## Downstream File Targets Frozen By This Audit

### Parser boundary
- `crates/ash-parser/src/parse_type_def.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/surface.rs`

### Core substrate
- `crates/ash-core/src/kind.rs`
- `crates/ash-core/src/type_ir.rs`
- `crates/ash-core/src/semantic_summary.rs`
- `crates/ash-core/src/lib.rs`
- `crates/ash-core/src/ast.rs`

### Typechecker substrate
- `crates/ash-typeck/src/kind.rs`
- `crates/ash-typeck/src/types.rs`
- `crates/ash-typeck/src/type_env.rs`
- `crates/ash-typeck/src/lib.rs`

## Compatibility Constraints Frozen Up Front

1. `base::Assoc` remains the only normative public projection spelling in Phase 110.
2. Current SPEC-035 simple associated-type substitution remains supported as a compatibility path.
3. Existing ADT/interface/workflow/capability/resource/do/comprehension behavior must remain non-regressed.
4. Parser rejection-boundary evidence belongs to TASK-797 alone; later tasks may rerun but not replace that evidence owner.
5. TASK-800 is not allowed to start until shared kind ownership, parser parity, and source/import interface-member identity plumbing exist.

## Explicit Deferrals To Later Packets

The following are out of scope for SPEC-B / Phase 110 and remain deferred to later packets in the DESIGN-034 sequence:

- sealed type-level domains and marker-domain coverage work
- public `type fn` syntax
- general normalization / normalize-and-compare equality
- recursive associated type-family computation
- computation-summary export/import across module boundaries
- propositions / disequality / proof search
- public kind binder syntax
- holes
- partial type-constructor application
- generalized new public projection spellings beyond current `base::Assoc`

## Completion Gate

Phase 110 was only allowed to claim honest completion once the repo satisfied all three preconditions this audit named as blockers for TASK-800 and later closeout:

1. shared `Kind` ownership in `ash-core`
2. explicit parser parity/rejection evidence across `parse_type_def.rs` and `parse_module.rs`
3. source/import interface-member identity plumbing in `TypeEnv`

Those gates are now reflected by the completed implementation tasks and the TASK-804 closeout evidence.