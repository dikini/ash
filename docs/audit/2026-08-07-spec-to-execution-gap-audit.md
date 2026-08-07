# Ash specification-to-execution gap audit

**Date:** 2026-08-07

**Audited revision:** `8b1b7ac5` (`main`)

**Question:** What prevents Ash programs described by the latest target specifications from being
parsed and executed through the production Surface Ash → checked Core → checked CPS → Engine →
CLI/daemon route?

## 1. Verdict

Ash can execute a real but bounded subset of ordinary programs end to end. A simple ordinary
`fn main`, static file or inline child modules, local/imported first-order function calls, primitive
arithmetic and comparisons, Boolean control flow, variable `let`, structural records and field
projection, and the current expression-macro subset reach checked Core, checked CPS, Engine
admission, and the same terminal envelope through CLI and daemon.

Ash cannot execute the latest target language as a whole. Although parsing and type checking have
specific gaps, the principal blocker is the incomplete expanded-surface-to-checked-Core projection
and the still narrower production admission integration around rows, handlers/providers,
contracts, and runtime frames. The private CPS evaluator is not the principal boundary.

The most direct counterexample is valid, parsed, and type-checked Ash:

```ash
fn main() -> Int {
    match 1 {
        1 => 1,
        _ => 0,
    }
}
```

The canonical route rejects this before producing a module Core artifact because module lowering
only projects Boolean `true`/`false` matches. The production CLI has a regression test proving that
it rejects this program without falling back to the legacy evaluator.

The overall target is therefore:

- **Implementation:** `partial`
- **Evidence:** `tested` for the bounded positive and negative slices described below
- **Parity:** `below_spec`

This agrees with the canonical module spec's own aggregate status: SPEC-103 §12 says the complete
rule is not implemented and has no evidence for the complete rule. Several planning ledgers now
describe the frozen Phase 207 subset as implemented; that does not supersede the broader target
language.

## 2. Scope and authority

The audit uses the target reading path in [SPEC-INDEX](../spec/SPEC-INDEX.md), principally:

- [CANONICAL-CORE](../spec/CANONICAL-CORE.md)
- [SPEC-095b target grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-095c surface AST, macros, and notation](../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-096b target effects](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b target types](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-098c surface-to-Core lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-099b target operational semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-099 Core language](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100 Core type checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-103 module realization](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md)

The following are excluded from the required completion domain:

- dynamic/lazy module loading, hot reload, packages, registries, and runtime module values;
- dedicated role syntax, policy syntax, role/policy authority, persistence, inheritance, and
  runtime behavior.

TASK-2077 removed those forms without a compatibility obligation. References to role or policy
language in target specs are specification residue, not implementation requirements for this
audit. Host sandbox/provenance “policy” configuration and “dynamic” contract discharge are distinct
concepts and are not excluded merely because they use those English words.

Non-callable declarations are not required to acquire runtime behavior merely because their syntax
exists. They are required to parse and check according to their specified phase, propagate through
imports when public, affect callable checking where referenced, and not prevent an otherwise valid
callable from executing.

## 3. The production routes that actually exist

### 3.1 Canonical static-module route

The normal route for a canonically parsed ordinary source is:

```text
parse_surface_file_with_path
  → CanonicalModuleGraphResolver
  → CanonicalExpandedModuleGraph
  → collect_canonical_expanded_module_graph
  → resolve_parsed_imports_from_collection
  → finalize_canonical_module_collection
  → build_checked_public_module_interface_closure
  → lower_complete_checked_module_route_closure
  → linked_module_closure_from_checked_definition_lowering
  → Engine::admit_linked_module_closure
  → Engine::execute_admitted_program
  → canonical terminal envelope
```

The orchestration is in
[ash-engine/src/lib.rs](../../crates/ash-engine/src/lib.rs), lines 4470–4644. Once canonical
finalization has selected a callable, lowering errors are returned and do not select a direct
evaluator. Engine admission revalidates the complete transport, links local/imported callable CPS,
rejects any linked CPS containing `Handle` or `Raise`, validates the resulting CPS, and executes the
root through the private checked CPS evaluator (lines 2991–3097 and 3297–3319).

This route is real, not a documentation-only plan.

### 3.2 Sealed compatibility routes

Sources that are deliberately kept off the canonical module route use `Engine::admit_program`.
These include selected handler/provider fixtures and an old entry-contract shape. They still produce
checked CPS and use the shared admitted-program dispatcher, but admission recognizes exact source or
checked-fact shapes rather than implementing a general semantic rule.

Examples include exact `time::sleep`, one declared operation shape, and named handler fixtures such
as `deep_affine_clock` and `forward_sleep`. The route classifier is a closed enum selected in
`seal_checked_admission_route_fact` at
[ash-engine/src/lib.rs](../../crates/ash-engine/src/lib.rs), lines 1115–1144. The source-specific
checks around lines 5350–5413 and 5528–5600 make these conformance witnesses, not general handler,
provider, or call semantics.

### 3.3 Client sharing and remaining client exceptions

Both main adapters submit `AdmittedProgramRequest` to `Engine::execute_admitted_program`:

- CLI adapter: [commands/run.rs](../../crates/ash-cli/src/commands/run.rs), lines 49–55 and
  373–429.
- daemon adapter: [commands/daemon.rs](../../crates/ash-cli/src/commands/daemon.rs), lines 43–53,
  452–464, and 1049–1074.

The selected Phase 207 parity corpus genuinely compares the same admitted linked closure through
the two adapters. However, `ash run` still has an explicit legacy-route selector for handler and
wrong-return-type compatibility sources (run.rs lines 227–235 and 1351–1379), and the daemon's
TASK-2035 descriptor endpoint independently parses and calls `admit_program` for one exact source
family (daemon.rs lines 467–516). These paths do not use a direct expression evaluator, but they do
mean that all clients are not yet one uniform source-to-admission pipeline.

## 4. Construct-by-construct execution status

Status meanings:

- **Runs:** demonstrated or directly covered by the canonical linked route.
- **Bounded:** only the stated subset runs; the target construct is not complete.
- **Metadata:** accepted as compile-time/non-authorizing information; it does not itself execute.
- **Rejects:** valid or representable surface syntax stops before production execution.

### 4.1 Entry, modules, imports, and visibility

| Construct | Production status | Concrete boundary |
|---|---|---|
| Ordinary zero-argument `fn main` | **Runs, bounded by body support** | A literal `fn main() -> Int { 42 }` executes through the real `ash run` command and the shared clients. Other valid bodies can fail later in lowering. |
| File-backed `mod child;` and inline `mod child { ... }` | **Runs** | Canonical graph, expansion, collection, finalization, Core/CPS, and linked execution are present for static closures. File/inline parity is extensively tested in TASK-2064. |
| Nested structural children | **Runs** | Nested child call and alias fixtures reach both clients. Missing, duplicate, and cyclic structural children reject atomically. |
| Explicit, grouped, glob, `self`, `super`, and qualified imports | **Runs for implemented namespace/path combinations** | The final canonical resolver is materially broader than the old scoped-slice documentation. Identity and visibility checks precede lowering. |
| Aliases and explicit re-exports | **Runs** | Imported/re-exported callables preserve defining identity and can execute transitively. Public child modules do not implicitly flatten exports. |
| `private`, `pub`, `pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ...)` | **Runs for checked static modules** | Positive restricted-visibility callable routes and negative private-access routes exist. |
| Public declarations in non-callable namespaces | **Metadata; propagated** | The final interface mapping covers types, type functions, propositions, promoted kinds, effect rows, interfaces, implementations, constructors, evidence, macros, and notation. TASK-2064 has file/inline import witnesses for each family. |
| Import/export closure as a standalone compiled interface | **Partial** | `PublicModuleInterface` retains binding identity, visibility, origin, dependencies, and typed identity, but the canonical builder supplies no `ModuleSemanticSummary`. It is not a complete serialized checked signature/row/body-independent interface as required by SPEC-103 §7. Static source-closure compilation compensates by retaining finalizer state. |
| Static import cycles | **Rejects by design** | This is permitted by SPEC-103's first realization. Recursive initialization is out of scope. |
| Dynamic loading/packages/hot reload | **Excluded** | Not a completion blocker. |

Every declaration marked public must remain import-visible subject to its enclosing path and
namespace rules. The current canonical route has broad propagation witnesses, but “the importing
program can use it” is stronger than “an identity carrier was transported.” The expression and
semantic gaps below still prevent some public declarations from being used in executable code.

### 4.2 Values and expressions

| Construct | Parse/check | Checked Core/CPS and execution |
|---|---|---|
| `Int`, `String`, `Bool`, `null` literals | Accepted | **Runs.** They map to checked Core atoms and CPS atoms. |
| Variables | Accepted | **Runs** when bound in the supported first-order environment. |
| Unary `-`, `!` | Accepted | **Runs.** Module lowering maps them to `Neg`/`Not`. |
| `+ - * / %`, equality, and ordering | Accepted | **Runs, bounded.** Module lowering maps these operators to Core primitives; Core typing determines supported operand types. Divide/remainder by zero become CPS traps. |
| `&&`, `||` | Accepted | **Runs.** The module projector explicitly builds short-circuit `If` terms, including selected nested call/record cases. |
| `if ... then ... else ...` | Accepted | **Runs** because parser lowering represents it as a two-arm Boolean match and the module projector accepts exactly that shape. |
| Boolean `match` with explicit `true` and `false` arms | Accepted | **Runs, bounded.** Both branches are required. |
| Integer/string/constructor/wildcard/general pattern `match` | Accepted and often type-checked | **Rejects before module Core.** The projector accepts only Boolean literal arms. The checked test at TASK-2069 lines 1243–1283 proves integer-pattern rejection. |
| `if let` | Accepted | **Runs only for a Boolean literal pattern.** Any other pattern reports “no checked boolean Core projection.” |
| `let` | Accepted | **Runs only for variable and wildcard bindings.** Tuple, record, constructor, list, or literal destructuring has no checked Core binding projection. |
| Ordinary blocks and expression statements | Accepted | **Runs when every nested expression belongs to the supported subset.** Statements become discard `let` bindings. |
| Ambient `do { ... }` with `let`, `<-`, expression statements, final `return` | Accepted | **Runs when it lowers to the same supported `let`/call subset.** It is direct-style sequencing, not a general effect runtime. Generic typed `do` and comprehensions reject in parser lowering. |
| Structural record literals | Accepted | **Runs**, including nested records, call-bearing fields, Boolean short-circuit fields, and field projection. |
| Field access | Accepted | **Runs for records** through `CorePrimOp::RecordGet`. |
| Index access | Parsed, then rejected by type checking | **Does not reach checked Core.** `check_expr` explicitly returns `UnsupportedExpression` for `IndexAccess`; the module projector also has no implementation. |
| ADT/newtype constructor expressions | Accepted and type-checkable | **Rejects in the canonical module body projector.** Constructor imports can propagate as metadata, but constructing one in an ordinary canonical callable has no projector case. The separate legacy entry route has a bounded structural-value special case and does not close the canonical gap. |
| List literals | Accepted | **Reject.** Parser lowering rewrites them to `Cons`/`Nil` constructors, which the module projector cannot lower. |
| General function calls | Accepted | **Runs for first-order local/imported ordinary callables**, parameters, aliases, re-exports, nested calls, and call results used by supported primitives/control flow. |
| Recursion/mutual recursion | Accepted in parts | **Rejects during linked callable cycle detection.** There is no production recursive-call realization for canonical modules. |
| Anonymous `fn`, closures, captures, function-valued expressions | Accepted in surface/core legacy AST | **Reject.** They become `Expr::FnDef`/general `FnApply`, but the module primitive-atom projector cannot create a checked Core lambda from them. Consequently operator sections that elaborate to closures are not generally executable. |
| Higher-order call through an arbitrary expression | Accepted in surface AST | **Reject except where the callee reduces to a supported variable identity.** There is no general closure conversion in module lowering. |
| Builtin calls (`len`, `map`, string/list/record helpers, etc.) | Accepted in the compatibility AST | **Not generally executable through canonical linked modules.** Bodyless builtin signatures can be transported, but Engine linking rejects a call with no lowered local callable. TASK-2064 lines 188–209 tests this fence. |
| `check obligation` | Accepted/lowered to legacy AST | **Rejects in module checked-Core projection.** No case exists in `surface_expr_to_primitive_atom`. |
| `fail` and `with_error` | Accepted/lowered to legacy AST | **Reject in canonical ordinary functions.** They never become checked Core `Trap`/`Raise`/`Handle` on this route. |
| `panic` | Parsed | **Explicitly rejects** in parser lowering. |
| Comprehensions | Parsed | **Explicitly reject** pending typed-do elaboration. |

The decisive code is
[module_core_cps_lowering.rs](../../crates/ash-typeck/src/module_core_cps_lowering.rs), lines
1833–2330 and 3369–3455. Its final atom conversion accepts only primitive literals, variables,
records, field access, unary operations, and the selected binary operations. Everything else returns
“surface expression ... has no checked Core projection in this slice.”

### 4.3 Patterns and algebraic data

The target grammar and Core language specify general patterns and Core/CPS matching. The production
CPS evaluator also has a `Term::Match` implementation. That does not make source matches runnable:
the canonical module projector rewrites only Boolean control to `CoreExpr::If` and never produces
general checked Core pattern-match terms.

Concrete missing work is:

1. lower constructor, tuple, record, list, literal, wildcard, and nested patterns to checked Core;
2. lower constructor/list/tuple values in ordinary callable bodies;
3. perform exhaustiveness and irrefutability checks against the same lowered pattern domain;
4. lower general match decision trees or Core match terms to CPS;
5. preserve constructor identity across imports and re-exports in executable values, not only
   interface metadata;
6. add positive, negative, mutation, and client-parity tests over those values.

Until this is done, ADT declarations and constructor imports are mainly type/interface metadata for
the canonical route.

One target declaration fails even earlier: ordinary bodyless nominal types such as
`type PosixFs;` are required by SPEC-095b §6.6 as unconstructable identity carriers, but
`parse_type_def` accepts the semicolon-only form only when preceded by `builtin`. A target-valid
non-builtin identity declaration therefore does not parse. This blocks the specification's minimal
effect-identity pattern before import, checking, or execution.

### 4.4 Functions, interfaces, implementations, and host callables

| Declaration/use | Production status | Gap |
|---|---|---|
| Ordinary functions | **Runs, bounded** | Only supported first-order bodies lower. All ordinary callable bodies in a closure are lowered atomically, so one unsupported unused function can reject the entire closure. |
| Parameters and imported calls | **Runs** | Demonstrated for primitive parameters and multiple imported callables. Generic values and higher-order parameters remain limited by type/body lowering. |
| Generic functions | **Partial** | Generic signatures and monomorphization machinery exist, but the canonical module execution corpus does not establish the full target generic/inference domain. Higher-kinded and row-polymorphic execution remains below spec. |
| Interface declarations | **Metadata/type checking** | Public interfaces import and can appear in checked metadata. Interface methods are parent-scoped and skipped as standalone module callables. |
| `impl` declarations | **Metadata/type checking** | Coherence/identity work exists, but parent-scoped impl methods are not emitted as ordinary local callable entries by module lowering. |
| Qualified `ImplType::op` operation calls | **Very bounded** | Type checking retains concrete operation identities. General operation calls in canonical modules would lower to `Raise`, and linked admission rejects all `Raise`. Exact declared-operation/provider compatibility routes are separate source-shape admissions. |
| `builtin fn` declarations | **Signature metadata only** | No general host implementation registry is linked to canonical callable entries. A call is intentionally fenced rather than invented. |

The declaration closure in module_core_cps_lowering.rs lines 1065–1135 emits body artifacts only
for top-level ordinary functions and handlers. Parent-scoped interface/implementation members are
explicitly skipped. Bodyless ordinary/builtin callables do not gain executable bodies.

### 4.5 Macros, notation, and operator sections

| Construct | Production status | Gap |
|---|---|---|
| Current binder-free expression macros | **Runs, bounded** | Local/imported expression macros that expand to supported ordinary expressions can execute. A focused TASK-2064 file/inline macro test passes through both clients. |
| Token-tree/general/item-generating/recursive/binder-introducing macros | **Reject or partial by design** | The target macro spec itself defines several bounded/non-goal regions. Any expansion that leaves a macro carrier must reject before Core. |
| Notation declaration/import | **Metadata/syntax phase** | Public summaries propagate and activate during canonical expansion. The Phase 207 execution fixture imports notation but does not use it in the executed body. |
| Operator sections | **Surface expansion exists; execution partial** | Expansion can eta-expand a section to an anonymous function. The module projector cannot lower anonymous functions/closures, so the target examples in SPEC-098c §10 do not generally run. |
| Generalized mixfix use sites | **Not implemented** | SPEC-103 explicitly says notation activation does not add generalized mixfix use-site parsing/elaboration. |

### 4.6 Rows, effects, handlers, and providers

This is the largest semantic integration gap.

1. Callable rows are retained in type metadata and transported across imports.
2. Channel row syntax is lossy: `parse_computation_row_item` parses the channel mode and path but
   always constructs `ComputationRowItem::Channel { payload: None, ... }`, even though the surface
   carrier has an optional payload field. A specified message type therefore cannot survive the
   parser into checking or admission.
3. Core rows normalize multiple requirement kinds.
4. Core-to-CPS lowering supports closed rows but rejects every open row tail
   (`core_ash_lower.rs` lines 1614–1618).
5. A separate `admit_application_with_explicit_rows` API checks rows on a legacy application
   admission request ([row_admission.rs](../../crates/ash-engine/src/row_admission.rs), lines
   522–607).
6. `admit_linked_module_closure`, the canonical module execution boundary, does not invoke that row
   admission logic. It validates/link-checks CPS and rejects only handler/raise authority before
   minting the linked admission.

Therefore row transport is not the same as production row discharge. The canonical route has no
general admission environment that takes the reachable entry's closed requirement row, verifies
kind-specific discharge, constructs only authorized frames, and passes those frames to execution.

#### Handlers

The type checker retains substantial handler facts, and checked Core/CPS supports `Handle`, `Raise`,
continuations, residual rows, and multiplicity. Module lowering nevertheless accepts only a narrow
handler declaration:

- exactly one operation clause;
- identity `done` clause;
- variable computation binder and variable operation payload;
- clause body is payload identity, one direct resume, or one same-operation raise.

See module_core_cps_lowering.rs lines 651–862. Even those lowered handler artifacts cannot execute
through the linked module route because Engine admission rejects any module CPS containing `Handle`
or `Raise` (ash-engine/src/lib.rs lines 3040–3057).

Selected source handlers do execute through separately sealed, name/source-shape-specific routes.
They prove that the private CPS evaluator can implement the needed mechanics; they do not implement
target handler application for arbitrary checked source.

#### Providers and operation effects

Provider frames are implemented in private CPS and in selected sealed drivers. General target
operation dispatch is still missing because canonical admission has no generic mapping from checked
operation requirements plus host bindings to Engine-issued frame-installation instructions. The
current production providers are exact `time::sleep` or exact declared-operation slices. Resources,
channels, processes, failures, and evidence do not have a unified linked-route discharge/execution
integration.

Rows remain non-authorizing metadata, which is correct. What is absent is the separate authorization
and discharge step required by SPEC-096b, SPEC-097b, and SPEC-099b.

### 4.7 Contracts, facts, proofs, evidence, traces, and monitors

| Construct | Current status | Missing execution behavior |
|---|---|---|
| `requires` / `ensures` and contract predicates | **Partial sidecars/checking** | Predicate lowering and discharge records exist in legacy/checker paths, but the canonical module route does not compose them into general pre/post runtime checks around every reachable call. |
| Core/CPS `RecordDischarge` | **Carrier only in private CPS evaluator** | The evaluator matches `Term::RecordDischarge { body, .. }` and evaluates `body`, ignoring the discharge at lines 185–187 of private_cps/mod.rs. Enforcement must occur before or around this point; the linked route does not supply it generally. |
| Facts, propositions, laws, proofs, evidence declarations | **Compile-time metadata, partial semantics** | Public identities propagate, but full proposition discharge, proof checking/refinement, and their effect on callable acceptance are not established for the target domain. They should not become runtime values. |
| Dynamic contract violations/faults | **Selected infrastructure** | Structured trap types exist, but ordinary canonical source does not generally lower and execute runtime check plans. |
| Trace contracts and temporal monitors | **Not active on linked route** | The target machine includes trace ledger and monitor set. Linked admissions set `permits_trace: false`; `ash run --trace` rejects them. The private CPS runtime has no general monitor advancement path for source-lowered trace contracts. |

Two concrete compile-time limitations matter even though these declarations do not execute:

- the normalizer explicitly supplies no type-function reduction/equation semantics, so public
  signatures that require such reduction cannot realize the complete target type system;
- `check_proof_totality` is still a stub: the regression test intentionally accepts a proof body
  containing `loop_forever`. Proof declarations therefore cannot yet serve as sound evidence for
  execution-sensitive obligations.

### 4.8 Evaluation modes

Checked Core, Core-to-CPS lowering, and the private CPS evaluator contain `LetMode`, thunk, force,
lazy, and memo machinery. The ordinary surface/module route does not project target evaluation-mode
syntax into these checked Core forms. Consequently:

- strict ordinary execution works;
- target lazy/memo behavior has Core/CPS unit evidence but no general source-to-client path;
- open latent rows fail Core-to-CPS lowering;
- mode/force behavior must not be counted as language execution merely because private IR tests pass.

## 5. Layer gap matrix

| Target family | Parse | Surface check/finalize | Checked Core | Checked CPS | Engine admission/runtime | CLI/daemon |
|---|---:|---:|---:|---:|---:|---:|
| Static modules/imports/visibility | implemented for broad static domain | implemented for frozen Phase 207 domain | implemented | implemented | implemented, handler-free | tested selected parity |
| Primitive first-order functions | implemented | implemented | implemented | implemented | implemented | tested |
| Records/field projection | implemented | implemented | implemented | implemented | implemented | tested |
| General patterns/ADT execution | implemented broadly, except bodyless nominal types | partial | not produced by module route | private IR support only | absent | rejects |
| Closures/higher-order/recursion | parser/legacy carriers exist | partial | not produced generally | IR support exists | canonical linking incomplete/rejects cycles | absent |
| Builtin/host calls | declarations/calls parse | signatures partial | no canonical callable body | no linked target | exact sealed providers only | general call rejects |
| Closed requirement rows | partial; channel payload is lost | retained for represented forms | retained | retained | disconnected from linked row admission | metadata/parity only |
| Open rows/row polymorphism | implemented in types | partial | retained | lowering rejects open tail | absent | rejects |
| Source handlers | implemented | substantial checked facts | one-clause lowering subset | subset lowers | linked route rejects; exact sealed fixtures run | exact fixtures only |
| Contracts/evidence | implemented in parts | partial sidecars | carriers exist | carriers exist | no general linked enforcement | selected fixtures only |
| Trace/monitors | syntax/sidecars in parts | partial | carrier infrastructure | incomplete source emission | linked trace disabled | no general route |
| Lazy/memo | syntax/type work in parts | partial | IR support | IR support | private evaluator support | no general source route |
| Macros | bounded expansion | bounded | erased correctly | ordinary result | works if expansion is in runnable subset | tested one imported subset |
| Notation/operator sections | declaration/import and section expansion | partial | closure result cannot generally lower | absent for closures | absent | declaration import only |

## 6. What concretely prevents complete execution

### Blocker A — the canonical target documents are internally inconsistent

TASK-2077 and SPEC-INDEX retire dedicated role/policy machinery, but the current target grammar,
effects, type-system, lowering, and module specs still contain role/policy productions, namespaces,
row kinds, and completion text. Examples include SPEC-095b §§8.1/9, SPEC-096b §§6.3–8,
SPEC-097b §§8.2–8.3, SPEC-098c's row table, and SPEC-103's namespace table.

This does not block implementing the user-directed language once those clauses are excluded, but it
does block an automated or honest claim that “latest specs” are one coherent executable contract.
The target specs must be reconciled before a full conformance gate can be definitive.

### Blocker B — there is no total expanded-surface-to-checked-Core lowering function

The production module bridge first calls the broad legacy `ash_parser::lower_expr`, then calls a
second, narrow function `surface_core_expr_to_checked_core`. The first function accepts or rewrites
many constructs that the second cannot represent. This two-step shape is the source of most
“parses/checks but does not run” outcomes.

The target requires one exhaustive lowering decision over the expanded surface AST. Each target
construct must either produce checked-Core-ready syntax and sidecars or return an explicit
unsupported diagnostic at the declared target boundary. Today support is determined by nested
shape recognizers and fallback atom conversion.

Before that boundary, the parser/checker must also close its explicit target gaps: non-builtin
bodyless nominal types, typed channel payload rows, and index access are concrete examples. These
are independent of the much larger Core-projection deficit.

### Blocker C — production admission is not a general validator over checked semantics

The linked route admits any complete, valid, handler-free CPS closure, but does not integrate:

- entry/reachable-row collection and kind-specific discharge;
- provider/handler frame construction from Engine-owned bindings;
- contract runtime-check plans;
- trace/monitor plans;
- source handler installations.

The compatibility route does integrate some of these, but only after exact source/fact recognition.
The Engine needs one semantic admission algorithm over checked artifacts, not additional named
fixture recognizers.

### Blocker D — declarations and imports are ahead of executable use

Phase 207 transported almost every public namespace. That was necessary, but several transports are
identity-only or metadata-only. Constructor, interface, implementation, row, type-function,
proposition, evidence, macro, and notation imports can all be present while the selected `main`
returns a literal. Such a test proves that the metadata does not interfere with stable execution;
it does not prove that every imported construct can perform its specified compile-time role in an
executed callable.

### Blocker E — conformance evidence is organized by historical task slices

The test suite has strong evidence for many slices, but no generated target-domain matrix that asks,
for every current grammar/semantic form, whether it:

1. parses;
2. expands;
3. checks;
4. lowers to checked Core;
5. lowers to checked CPS;
6. is admitted or intentionally rejected;
7. executes with the same terminal result through CLI and daemon.

`SEMANTIC-RULE-COVERAGE.md` also contains historical `partial` records followed by frozen-domain
`implemented` records for the same aggregate area. It is useful provenance but not a reliable
language-wide status answer without reading each declared domain.

## 7. Narrow implementation programme

The work should extend the existing canonical route, not add another evaluator or preserve obsolete
compatibility behavior.

### Slice 1 — reconcile the executable target

- Remove residual dedicated role/policy grammar and semantics from the target read path.
- Keep dynamic module loading explicitly excluded.
- Mark every remaining declaration as runtime, compile-time, or syntax-phase.
- Give every remaining expression and declaration one canonical rule identifier and expected
  terminal/admission behavior.

Exit condition: the target read path yields one machine-readable or reviewable construct inventory
with no contradictory domain.

### Slice 2 — make surface-to-Core lowering exhaustive

- Replace the legacy-AST/narrow-projector split for canonical modules with one expanded-surface
  lowering boundary.
- Preserve source origins, normalized rows, contracts/evidence, and trace sidecars.
- Implement values and patterns first: tuples, constructors, lists, index access, destructuring,
  and general match.
- Keep explicit rejection for constructs intentionally not selected yet.

Exit condition: every target surface expression has a tested Core product or a target-authorized
rejection, with no “in this slice” fallback for a target construct.

### Slice 3 — complete first-order callable execution

- Support all target first-order argument/result values and general pattern control.
- Define and implement recursive local/imported callable linking where the target permits it.
- Connect bodyless builtin declarations to an explicit Engine host-callable binding or reject them
  earlier as unavailable; never invent a body.
- Ensure all public callable signatures and rows survive the public interface and are usable by
  importers.

Exit condition: the first-order, handler-free target fragment is complete through both clients.

### Slice 4 — add higher-order functions and evaluation modes

- Lower anonymous functions, captures, function-valued arguments/results, and operator-section
  closures to checked Core lambdas and CPS closures.
- Integrate generic specialization/row inference needed by those calls.
- Connect target lazy/memo syntax to existing checked Core/CPS thunk machinery.
- Decide and implement the closed/open-row boundary for CPS rather than failing on every row tail.

Exit condition: target higher-order and mode examples run from source, not only hand-built IR tests.

### Slice 5 — integrate rows, handlers, and providers at one admission seam

- Compute the reachable entry requirement row from linked checked artifacts.
- Perform kind-specific discharge against Engine-owned operation/resource/channel/process/failure/
  evidence bindings.
- Convert validated source handlers and admitted providers into ordered frame instructions.
- Remove exact handler-name/source-spelling selectors once equivalent semantic admission exists.
- Execute the admitted CPS with the resulting frame chain.

Exit condition: arbitrary target-valid closed-row operation and handler programs execute, while
missing discharge produces the specified structured terminal and rows remain non-authorizing.

### Slice 6 — contracts, evidence, traces, and monitors

- Carry contract/evidence/trace sidecars through canonical module artifacts.
- Run preconditions, postconditions, and dynamic predicate plans at their specified boundaries.
- Make `RecordDischarge` semantically meaningful or prove that enforcement has already occurred.
- Install and advance trace monitors; expose their structured terminal outcomes.
- Preserve compile-time-only facts/proofs as non-runtime metadata.

Exit condition: the target operational state components `τ` and `Ω`, contract traps, and evidence
requirements are observable through the canonical client route.

### Slice 7 — remove route exceptions and establish full conformance

- Route CLI and daemon source acquisition through the same canonical source-to-admission function.
- Delete legacy route selection when its selected behavior is covered.
- Generate positive, negative, mutation, and CLI/daemon parity cases from the target inventory.
- Update semantic coverage by target rule, with one current record per rule and separate
  implementation/evidence/parity axes.

Exit condition: every in-scope target rule is `implemented / tested-or-proved / matches_spec` and no
client can select an alternate semantic path.

## 8. Recommended order and immediate acceptance tests

The shortest path to a meaningfully larger language is:

1. spec reconciliation;
2. exhaustive value/pattern/match lowering;
3. first-order callable and builtin completion;
4. linked row admission plus general handlers/providers;
5. higher-order/modes;
6. contracts/traces/monitors;
7. client-route deletion and complete generated conformance.

The first new acceptance corpus should include at least:

```ash
// General data and pattern execution.
type Option<A> = None | Some(A)
fn main() -> Int {
    match Some(41) {
        Some(x) => x + 1,
        None => 0,
    }
}
```

```ash
// Imported public declaration is used, not merely transported.
pub mod values {
    pub type Pair = Pair(Int, Int)
    pub fn make() -> Pair { Pair(40, 2) }
}
use crate::values::{Pair, make}
fn main() -> Int {
    match make() { Pair(a, b) => a + b }
}
```

```ash
// General checked operation and handler admission; names are not special.
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int)
impl Clock<TestClock> { sleep(ms) = ms }
handler identity(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(42) with identity }
```

Each case must enter as source, produce checked Core and CPS without caller-authored IR, be admitted
from checked facts rather than exact source spelling, and produce the same normalized terminal via
CLI and daemon.

## 9. Evidence run during this audit

The following focused commands passed at revision `8b1b7ac5`:

```text
cargo test -p ash-cli --test task_2064_module_conformance_and_client_parity \
  real_parser_checked_ordinary_root_route_reaches_both_clients -- --exact

cargo test -p ash-cli --test task_2064_module_conformance_and_client_parity \
  real_parser_imported_macro_file_and_inline_routes_reach_both_clients -- --exact

cargo test -p ash-typeck --test task_2069_complete_module_lowering \
  unsupported_checked_module_definition_body_is_rejected_before_artifact_creation -- --exact

cargo test -p ash-cli \
  task_2064_production_run_uses_canonical_module_route_for_an_ordinary_root -- --nocapture

cargo test -p ash-cli \
  task_2064_production_run_rejects_parseable_unsupported_callable_without_fallback -- --nocapture
```

These establish:

- a real `ash run` canonical-module success returning `42`;
- the same ordinary checked result through CLI and daemon adapters;
- imported expression-macro expansion and execution for file/inline modules;
- an explicit parse/check-to-Core gap for integer pattern matching;
- no fallback reinterpretation after that canonical lowering failure.

They do not establish full target-language conformance.
