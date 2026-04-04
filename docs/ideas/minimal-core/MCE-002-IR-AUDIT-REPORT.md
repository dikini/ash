# MCE-002 Audit Report: IR Core Forms Audit

Status: complete
Task: [TASK-370](../../plan/tasks/TASK-370-ir-core-forms-audit.md)
Primary source: `crates/ash-core/src/ast.rs`
Supporting sources: `crates/ash-core/src/workflow_contract.rs`, `crates/ash-core/src/stream.rs`, `crates/ash-core/src/effect.rs`, `crates/ash-core/src/value.rs`

## Executive Summary

The current Ash core IR is centered on one de facto primary core-AST carrier in `crates/ash-core/src/ast.rs` containing:
- 30 `Workflow` forms (`ast.rs:17-198`)
- 13 `Expr` forms (`ast.rs:393-473`)

The main audit conclusion is that the immediate minimization opportunity is not the removal of large numbers of `Workflow` or `Expr` forms from `ast.rs`. Instead, the highest-value simplifications are:

1. Treat `ast.rs` as the de facto primary core workflow/expression carrier and the recommended future source of truth for the core AST layer.
2. Account explicitly for the fact that the current repository still has a major parser-surface/typechecker representation layer alongside the core AST, plus additional coupling through `workflow_contract.rs`.
3. Consolidate or remove duplicate mini-IR carriers that currently exist in `workflow_contract.rs`, while accounting for the fact that `ast.rs` and downstream code still depend on some of them today.
4. Consolidate overlapping receive and contract representations that are split across `ast.rs`, `workflow_contract.rs`, and `stream.rs`.
5. Treat `Expr::IfLet` as confirmed sugar.
6. Keep `Workflow::Seq` as primitive.
7. Defer elimination of `Match`, `Set`, `Send`, and statement/expression duplication (`Spawn`, `Split`, `CheckObligation`) until lowering and semantics are made more explicit.

In other words: the current IR surface is larger than necessary, but the main problem is duplicated representation layers rather than a single bloated enum, and those layers currently include not just `workflow_contract.rs` and `stream.rs`, but also an active parser-surface/typechecker representation path.

## Method

This audit used:
- direct inspection of the current AST/IR definitions in `ash-core`
- cross-checking against parser, typechecker, interpreter, REPL, benchmark, and test references across `crates/`
- reference-hit counts (`Workflow::X` / `Expr::X`) as a rough impact signal only

Important note: reference-hit counts below are not runtime-frequency measurements. They are repository reference counts from repository-wide `Workflow::X` / `Expr::X` searches under `crates/` and mainly indicate migration impact.

## Canonical Source Map

### Main canonical AST
- `crates/ash-core/src/ast.rs:17-198` — `Workflow`
- `crates/ash-core/src/ast.rs:393-473` — `Expr`
- `crates/ash-core/src/ast.rs:272-290` — `ReceiveArm`, `ReceivePattern`
- `crates/ash-core/src/ast.rs:294-307` — `Pattern`
- `crates/ash-core/src/ast.rs:375-389` — `Guard`, `Predicate`
- `crates/ash-core/src/ast.rs:612-689` — `ModuleItem`, `TypeBody`, `TypeExpr`, `Definition`

### Overlapping secondary carriers
- `crates/ash-core/src/workflow_contract.rs:53-65` — duplicate `Effect`
- `crates/ash-core/src/workflow_contract.rs:172-214` — duplicate `Parameter`, `TypeExpr`, `Span`, `WorkflowDef`, and a mini `Workflow`
- `crates/ash-core/src/stream.rs:87-178` — alternate `ReceiveMode`, `ReceiveArm`, and `Receive`
- `ash_parser::surface` + `ash-typeck` workflows/expressions — active downstream surface/typechecker representation layer that still coexists with the core AST in practice (for example `crates/ash-parser/src/lower.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/capability_check.rs`, and `crates/ash-typeck/src/names.rs` still operate on parser-surface carriers)

## Inventory Summary

### Workflow inventory

`Workflow` lives at `crates/ash-core/src/ast.rs:17-198`.

| Form | Line | Repo ref hits | Current role | Assessment |
|------|------|---------------|--------------|------------|
| Observe | 19 | 63 | effectful observation/binding | Keep for now; overlaps conceptually with `Act`, but not yet reducible safely |
| Receive | 25 | 47 | mailbox/control intake | Keep; primitive in current runtime model |
| Orient | 31 | 31 | evaluate expression then continue | Keep for now |
| Propose | 36 | 26 | advisory action | Keep for now |
| Decide | 41 | 30 | policy-mediated decision | Keep for now |
| Check | 47 | 71 | deontic obligation check + continuation | Keep; semantically distinct from current `CheckObligation` shapes |
| Act | 52 | 57 | side-effecting action with guard/provenance | Keep; current primitive effect carrier |
| Oblig | 58 | 44 | role-scoped deontic wrapper | Keep for now; naming overlaps with `Oblige` |
| Let | 60 | 67 | binding primitive | Primitive |
| If | 66 | 51 | workflow control branch | Primitive |
| Seq | 72 | 66 | workflow sequencing | Primitive; keep |
| Par | 77 | 34 | parallel composition | Primitive |
| ForEach | 79 | 6 | collection iteration | Keep for now; low current footprint |
| Ret | 85 | 110 | return from workflow | Primitive |
| With | 87 | 29 | capability scoping | Keep for now |
| Maybe | 92 | 30 | fallback/optional workflow control | Keep pending deeper semantics review |
| Must | 97 | 26 | mandatory workflow wrapper | Keep pending deeper semantics review |
| Set | 99 | 22 | capability/channel mutation | Review candidate |
| Send | 105 | 23 | capability/channel send | Review candidate |
| Spawn | 112 | 6 | statement-level spawn with binding | Keep for now; overlaps with `Expr::Spawn` |
| Split | 120 | 6 | statement-level split with binding | Keep for now; overlaps with `Expr::Split` |
| Kill | 127 | 4 | lifecycle control | Primitive runtime op |
| Pause | 133 | 7 | lifecycle control | Primitive runtime op |
| Resume | 139 | 18 | lifecycle control | Primitive runtime op |
| CheckHealth | 145 | 7 | lifecycle inspection | Primitive runtime op |
| Oblige | 151 | 38 | linear obligation introduction | Keep semantics; representation should be consolidated |
| CheckObligation | 157 | 11 | workflow-level linear obligation check | Review candidate for consolidation |
| Yield | 166 | 43 | proxy suspension/resume protocol | Primitive runtime/proxy op |
| ProxyResume | 185 | 6 | proxy response completion | Primitive runtime/proxy op |
| Done | 197 | 252 | terminal workflow | Primitive |

### Expression inventory

`Expr` lives at `crates/ash-core/src/ast.rs:393-473`.

| Form | Line | Repo ref hits | Current role | Assessment |
|------|------|---------------|--------------|------------|
| Literal | 394 | 507 | constant values | Primitive |
| Variable | 395 | 163 | names/references | Primitive |
| FieldAccess | 396 | 21 | record projection | Primitive enough to keep |
| IndexAccess | 400 | 15 | indexed projection | Primitive enough to keep |
| Unary | 404 | 36 | unary ops | Primitive |
| Binary | 408 | 120 | binary ops | Primitive |
| Call | 413 | 37 | function/capability call | Primitive |
| Constructor | 419 | 55 | ADT construction | Primitive |
| Match | 425 | 21 | pattern-match expression | Review candidate |
| IfLet | 431 | 25 | sugar for `Match` | Derived sugar |
| Spawn | 440 | 6 | value-producing spawn | Overlaps with `Workflow::Spawn`; keep pending lowering decision |
| Split | 447 | 4 | value-producing split | Overlaps with `Workflow::Split`; keep pending lowering decision |
| CheckObligation | 467 | 10 | Bool-returning linear check | Strong consolidation candidate |

## Related Helper Types

### Receive and pattern helpers
- `ReceiveMode` — `ast.rs:267-270`
- `ReceiveArm` — `ast.rs:274-278`
- `ReceivePattern` — `ast.rs:282-290`
- `Pattern` — `ast.rs:294-307`
- `Guard` — `ast.rs:375-382`
- `Predicate` — `ast.rs:386-389`
- `MatchArm` — `ast.rs:502-505`

### Action, capability, role, obligation helpers
- `Capability` — `ast.rs:252-256`
- `Action` — `ast.rs:260-263`
- `Constraint` — `ast.rs:509-511`
- `Observe` helper struct — `ast.rs:517-526`
- `Changed` helper struct — `ast.rs:533-540`
- `Obligation` — `ast.rs:544-548`
- `RoleObligationRef` — `ast.rs:556-558`
- `Role` — `ast.rs:562-566`
- `InputCapability` — `ast.rs:573-588`
- `ProxyDef` — `ast.rs:595-606`

### Top-level carrier helpers
- `ModuleItem` — `ast.rs:612-623`
- `TypeDef` — `ast.rs:627-636`
- `TypeBody` — `ast.rs:640-647`
- `TypeExpr` — `ast.rs:671-680`
- `Definition` — `ast.rs:684-689`

## Expressibility Analysis

### Confirmed result: `Workflow::Seq` is primitive

`Workflow::Seq` composes two workflows directly:
- `crates/ash-core/src/ast.rs:72-75`

`Workflow::Let` expects an expression in its binding position:
- `crates/ash-core/src/ast.rs:60-64`

Therefore the proposed rewrite:
- `Seq(a, b) => Let { pattern: _, expr: a, continuation: b }`

is invalid because `a` is a `Workflow`, not an `Expr`.

Decision:
- keep `Seq`
- treat it as the canonical primitive sequencing form
- do not spend further elimination effort on it unless the IR is radically restructured around effectful expressions

### Confirmed result: `Expr::IfLet` is sugar

The source comment explicitly states:
- `crates/ash-core/src/ast.rs:430` — `If-let expression (sugar for match)`

Decision:
- `Expr::IfLet` is derived sugar over `Expr::Match`
- it can remain as parser/front-end sugar even if a future normalized IR lowers it away

### `Expr::Match`

Current shape:
- `crates/ash-core/src/ast.rs:425-428`
- `MatchArm` at `crates/ash-core/src/ast.rs:502-505`

Assessment:
- probably expressible via nested tests and destructuring only if the IR first gains explicit primitive pattern tests/extractors
- current IR does not have such a small explicit extractor vocabulary
- eliminating `Match` now would move complexity into hidden lowering rules and worsen readability/debuggability

Decision:
- keep `Expr::Match` for now
- revisit only after the pattern subsystem is itself normalized

### `Set` and `Send`

Current shapes:
- `Set` — `ast.rs:99-103`
- `Send` — `ast.rs:105-109`
- `Act` — `ast.rs:52-56`

Assessment:
- conceptually these look like specialized capability operations that might be representable as `Act`
- but the current AST gives them narrower, structured capability/channel payloads that likely matter for capability checking, parser lowering, and runtime handling
- collapsing them into `Act` now would hide distinctions before effect/capability contracts are clarified

Decision:
- keep both for now
- revisit after capability operation semantics are unified

### `Check`, `Workflow::CheckObligation`, `Expr::CheckObligation`

Current shapes are not identical:
- `Workflow::Check` — deontic obligation + continuation (`ast.rs:47-50`)
- `Workflow::CheckObligation` — workflow-form linear obligation op (`ast.rs:156-160`)
- `Expr::CheckObligation` — Bool-returning expression form (`ast.rs:449-472`)

Assessment:
- these forms overlap in name and intent but do not yet encode the same semantics
- the expression form is the clearest carrier for “check and return Bool”
- however, `Expr::CheckObligation` is not yet fully viable as a normalization target because current typechecking still rejects it as unsupported (`crates/ash-typeck/src/check_expr.rs`) even though the interpreter can execute it (`crates/ash-interp/src/eval.rs`)
- the workflow-form `CheckObligation` is also executable today (`crates/ash-interp/src/execute.rs`), so current implementation evidence mainly shows different gaps between workflow execution, expression execution, and typechecker support rather than a cleanly dominant representation

Decision:
- do not unify `Check` with `CheckObligation`
- treat any attempt to remove `Workflow::CheckObligation` in favor of `Expr::CheckObligation` plus `Let`/`If`-based workflow composition as blocked until expression-level obligation checking is supported cleanly by the typechecker and downstream control-flow sites
- do not read the current implementation as proving `Workflow::CheckObligation` is categorically better supported overall; the present evidence mainly shows different gaps between workflow execution, expression execution, and typechecker support
- first confirm all interpreter/typechecker sites can express the same control flow cleanly

### `Workflow::Spawn` vs `Expr::Spawn`; `Workflow::Split` vs `Expr::Split`

Current distinction:
- workflow forms are statement-like and continuation/binding oriented
- expr forms are value-producing

Assessment:
- these are duplicates in concept, not in type role
- removing either side prematurely would force either:
  - effectful expressions everywhere, or
  - awkward statement-to-expression adapters
- current IR does not yet have a clear normalized effectful-expression discipline

Decision:
- keep both pairs for now
- revisit only when the language decides whether spawn/split are fundamentally expressions with statement sugar, or statements with value-binding sugar

## Structural Duplication Findings

These are the strongest simplification opportunities discovered by the audit.

### 1. Duplicate mini-IR in `workflow_contract.rs`

`workflow_contract.rs` defines a second copy of several core carriers:
- `Effect` — `workflow_contract.rs:53-65`
- `Parameter` — `workflow_contract.rs:172-177`
- `TypeExpr` — `workflow_contract.rs:179-185`
- `Span` — `workflow_contract.rs:187-192`
- `WorkflowDef` — `workflow_contract.rs:194-203`
- `Workflow` — `workflow_contract.rs:205-214`

Meanwhile `ast.rs` already defines the primary AST workflow/expression carriers, but it still directly embeds `crate::workflow_contract::...` types for `Span`, `TypeExpr`, and `Contract`, and downstream parser/interpreter code also consumes those contract-side carriers today.

Recommendation:
- treat `ast.rs` as the preferred long-term AST source of truth
- migrate duplicate mini-IR carriers out of `workflow_contract.rs` only after the remaining `workflow_contract::Span`, `TypeExpr`, and `Contract` dependencies are explicitly unwound
- keep only contract-specific structures there (`Contract`, `Requirement`, `PostPredicate`, `ObligationSet`, errors) once those migrations are complete
- reuse canonical `ast.rs` and `effect.rs` types where feasible

Priority: high
Risk: moderate migration effort and coupling cleanup across parser/interpreter/typechecker users

### 2. Duplicate effect lattice

There are two `Effect` enums:
- `crates/ash-core/src/effect.rs`
- `crates/ash-core/src/workflow_contract.rs:53-65`

Recommendation:
- make `crates/ash-core/src/effect.rs` the only `Effect`
- import it from contract code

Priority: high
Risk: moderate; the consolidation itself is straightforward, but current `workflow_contract::Effect` users in downstream code mean the migration surface is larger than the enum definition alone suggests

### 3. Duplicate receive representation

AST receive:
- `ast.rs:265-290` and `Workflow::Receive`

Stream receive:
- `stream.rs:87-178`

These are clearly related but not identical:
- AST uses `ReceivePattern`
- stream uses plain `Pattern`
- AST `Workflow::Receive` has `control: bool`
- stream `Receive` has `control_arms: Option<Vec<ReceiveArm>>`

Recommendation:
- treat `ast.rs` receive carriers as the leading candidate for canonical IR because current core execution already bridges through them
- keep runtime mailbox/container logic in `stream.rs`
- avoid maintaining two quasi-IR receive carrier sets

Priority: high
Risk: moderate

### 4. Possible top-level carrier overlap

`ast.rs` has both:
- `ModuleItem` (`ast.rs:612-623`)
- `Definition` (`ast.rs:684-689`)

Assessment:
- this is still a weaker duplication concern than the `workflow_contract.rs` and `stream.rs` overlaps
- however, current repository evidence is not fully symmetric: `ModuleItem` has at least test usage, while `Definition` appears to have little or no meaningful downstream use beyond its declaration

Recommendation:
- confirm whether `Definition` is effectively legacy or unused
- if that holds, prefer removing `Definition` rather than treating both carriers as equally likely survivors

Priority: medium
Risk: low to moderate; current evidence already points toward asymmetry, even if a final cleanup should still verify usage before deletion

## Minimal Core Proposal

This proposal is intentionally conservative. It minimizes the IR only where the source already gives strong evidence.

### Essential workflow forms to keep

This list is intentionally conservative and should be read as the minimum workflow set the audit is willing to defend today, not as proof that every omitted form should be removed next.

#### Control and binding
- `Let`
- `If`
- `Seq`
- `Ret`
- `Done`

#### Concurrency and runtime control
- `Par`
- `Spawn`
- `Split`
- `Kill`
- `Pause`
- `Resume`
- `CheckHealth`
- `ForEach`

#### Communication
- `Receive`
- `Send`
- `Yield`
- `ProxyResume`

#### Effects/policy/capability
- `Observe`
- `Orient`
- `Propose`
- `Decide`
- `Act`
- `With`
- `Check`
- `Oblig`
- `Oblige`

#### Workflow control forms still kept pending deeper semantics review
- `Maybe`
- `Must`

### Essential expression forms to keep
- `Literal`
- `Variable`
- `FieldAccess`
- `IndexAccess`
- `Unary`
- `Binary`
- `Call`
- `Constructor`
- `Match`

### Confirmed sugar/derived forms
- `Expr::IfLet` — semantically sugar over `Expr::Match`; current lowering/documentation should move toward normalizing it, but the repository does not yet lower it away everywhere today

### Forms to review next, but not eliminate yet
- `Set`
- `Send` as specialized `Act`
- `Workflow::CheckObligation`
- `Expr::Spawn`
- `Expr::Split`
- `Expr::CheckObligation`
- `Maybe`
- `Must`

### Structural non-form cleanups with highest ROI
- remove duplicate `workflow_contract.rs` mini-IR carriers
- remove duplicate `workflow_contract.rs::Effect`
- unify receive carriers between `ast.rs` and `stream.rs`
- choose one top-level carrier: `ModuleItem` or `Definition`

## Recommended Rewriting / Normalization Rules

### Safe now
1. `Expr::IfLet(pattern, expr, then, else)`
   -> lower to `Expr::Match { scrutinee: expr, arms: [pattern => then, _ => else] }`

### Candidate future rewrites after semantic cleanup
2. `Workflow::CheckObligation`
   -> likely lower to expression-level `CheckObligation` plus explicit binding/branching
3. duplicate contract mini-IR carriers
   -> rewrite all users to canonical `ast.rs` and `effect.rs` carriers
4. duplicate receive carriers
   -> rewrite stream-facing IR users to one receive representation

### Explicitly rejected rewrite
5. `Workflow::Seq`
   -> no valid rewrite to `Let` in current typed AST

## Impact Assessment on Existing Tests and Examples

### High-impact forms
These have broad repo footprints and would be expensive to change mechanically:
- `Done` — 252 reference hits
- `Literal` — 507 reference hits
- `Variable` — 163 reference hits
- `Ret` — 110 reference hits
- `Check` — 71 reference hits
- `Let` — 67 reference hits
- `Seq` — 66 reference hits
- `Observe` — 63 reference hits
- `Act` — 57 reference hits
- `If` — 51 reference hits
- `Receive` — 47 reference hits
- `Yield` — 43 reference hits

### Medium-impact cleanup targets
- `Expr::IfLet` — 25 hits; likely easy to normalize
- `Set` / `Send` — 22 / 23 hits; moderate parser/typechecker/interpreter blast radius
- `Workflow::CheckObligation` / `Expr::CheckObligation` — 11 / 10 hits; manageable but semantically delicate
- `Spawn` / `Split` duplicates — low raw hits, but runtime-sensitive

### Examples
Repository examples currently show ongoing real use of obligation constructs, especially surface `oblige` forms, for example:
- `examples/multi_agent_research.ash`
- `examples/code_review.ash`
- `examples/03-policies/02-time-based.ash`
- `examples/04-real-world/customer-support.ash`
- `examples/workflows/40_tdd_workflow.ash`

This suggests obligation semantics are not dead weight, but it does not prove the current internal representation split is necessary.

### Interpretation
The safest near-term simplification work is:
1. remove duplicate internal carrier types
2. normalize confirmed sugar (`IfLet`)
3. postpone deeper form elimination until semantics and lowering are cleaner

## Final Recommendations

### Do next
1. Make `crates/ash-core/src/ast.rs` the explicit long-term AST source of truth, but only after unwinding remaining `workflow_contract` type dependencies.
2. Remove duplicate `workflow_contract.rs` carrier types after migrating users.
3. Delete duplicate `workflow_contract.rs::Effect` and reuse `effect.rs::Effect`.
4. Normalize `Expr::IfLet` as sugar in lowering/documentation.
5. Choose one canonical receive IR shape.

### Do not do yet
1. Do not eliminate `Workflow::Seq`.
2. Do not eliminate `Expr::Match` yet.
3. Do not fold `Set`/`Send` into `Act` yet.
4. Do not collapse statement/expression `Spawn` and `Split` forms until the language chooses a clearer effect/value discipline.
5. Do not unify `Check` with `CheckObligation`; they are currently different abstractions.

## Decision Summary

| Item | Decision |
|------|----------|
| `Workflow::Seq` | Keep as primitive |
| `Expr::IfLet` | Treat as sugar over `Match` |
| `Expr::Match` | Keep for now |
| `Set` / `Send` | Keep, review later |
| `Workflow::CheckObligation` | Review for consolidation |
| `Expr::CheckObligation` | Keep for now; natural Bool-returning carrier once typechecker support exists |
| `Workflow::Spawn` / `Expr::Spawn` | Keep both for now |
| `Workflow::Split` / `Expr::Split` | Keep both for now |
| `workflow_contract.rs` mini-IR duplicates | Remove / consolidate |
| duplicate `Effect` enum | Remove / consolidate |
| duplicate receive carriers | Consolidate |
| `ModuleItem` vs `Definition` | Measure usage first; consolidate only if overlap is confirmed |

## Follow-on Work Unblocked by This Audit

- MCE-004: Big-step semantics alignment
- MCE-007: Full layer alignment

Both should now proceed using this working assumption:
- `ast.rs` is the de facto primary core-AST carrier and the recommended future source of truth for the core layer, though current parser-surface/typechecker flows and `workflow_contract` coupling remain significant and should not be ignored in migration planning
- the next cleanup wins come from representation consolidation first, and only then from selective form elimination
