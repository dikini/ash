---
id: language.reference.types.type-level-domains-functions-families-and-propositions
title: Type-Level Domains, Functions, Families, and Propositions
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/**", "crates/ash-engine/src/module_loader/**"]
---

# Type-Level Domains, Functions, Families, and Propositions

[Types index](index.md) · [Generics, kinds, interfaces, and implementations](generics-kinds-interfaces-and-impls.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Sealed type domains and marker constructors | accepted | checked | lowered | not-applicable | partial | tested | below_spec |
| Proper-type/sealed-domain `type fn` declarations and closed reduction | accepted | checked | lowered | not-applicable | partial | tested | below_spec |
| Constructor-kinded `type fn`/`prop` binder syntax | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Sealed associated type families and projection normalization | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |
| Named `prop` declarations and proposition tails | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |
| `data kind Name from type Source;` | parser-only | not-applicable | not-applicable | not-applicable | partial | tested | below_spec |

This is type-level metadata and checking, not an Engine program-execution path. The primary
surface routes are `crates/ash-parser/src/parse_module.rs::{parse_sealed_domain_definition,
parse_type_fn_definition,parse_proposition_predicate_decl,parse_data_kind_definition}`. The
metadata lowerer is `crates/ash-parser/src/lower.rs::lower_module_type_metadata`; source type
function registration and normalization are in `ash-typeck::TypeEnv` and
`ash-typeck::normalizer`. The Engine's module-loader tests below cover selected summary/import
transport, not source-program admission or execution.

Focused evidence includes:

- `crates/ash-parser/tests/task_813_sealed_domain_diagnostics.rs`
- `crates/ash-parser/tests/task_846_public_type_fn_visibility.rs`
- `crates/ash-parser/tests/task_874_proposition_surface.rs`
- `crates/ash-parser/tests/task_881_proposition_parse_diagnostics.rs`
- `crates/ash-parser/tests/task_893_promoted_constructor_parser_surface.rs`
- `crates/ash-parser/tests/task_900_type_hole_surface.rs`
- `crates/ash-parser/tests/task_906_hkt_kinded_binder_surface.rs`
- `crates/ash-typeck/tests/task_827_normalizer_diagnostics.rs`
- `crates/ash-typeck/tests/task_837_type_function_recursion.rs`
- `crates/ash-typeck/tests/task_838_type_function_normalizer.rs`
- `crates/ash-typeck/tests/task_866_associated_family_normalizer.rs`
- `crates/ash-typeck/tests/task_868_associated_family_diagnostics.rs`
- `crates/ash-typeck/tests/task_875_proposition_environment.rs`
- `crates/ash-typeck/tests/task_876_proposition_solver.rs`
- `crates/ash-typeck/tests/task_878_named_predicate_registration.rs`
- `crates/ash-typeck/tests/task_879_proposition_summary_import.rs`
- `crates/ash-typeck/tests/task_880_proposition_checking_points.rs`
- `crates/ash-typeck/tests/task_881_proposition_diagnostics.rs`
- `crates/ash-typeck/tests/task_882_spec_h_acceptance_matrix.rs`
- `crates/ash-typeck/tests/task_906_hkt_fail_closed.rs`
- `crates/ash-engine/tests/task_811_domain_summary_transport.rs`
- `crates/ash-engine/tests/task_849_type_computation_summary_transport.rs`
- `crates/ash-engine/tests/task_867_associated_family_summary_transport.rs`
- `crates/ash-engine/tests/task_879_proposition_summary_transport.rs`
- `crates/ash-engine/tests/task_880_proposition_public_integration.rs`
- `crates/ash-engine/tests/task_882_spec_h_transport_non_interference.rs`

## Sealed type domains

A sealed domain names a finite, module-owned collection of marker constructors for type-level
pattern matching. Use a field slot of `Type` for an unconstrained type argument, or the name of a
sealed domain for an argument constrained to that domain. The parser accepts the declaration;
lowering exports a domain summary with canonical domain/constructor identities; the checker
validates domains and uses their structural-field information for recursive checks.

**Static-only source declaration.** This is the source shape exercised by the parser, summary,
and type-function tests. It declares metadata; it does not construct a runtime value.

```ash
pub sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}
```

Domains have no generic parameter list. Constructor visibility is inherited from the domain,
not declared per constructor. Field slots are deliberately narrow: list, tuple, path-like, and
generic/applied field-slot forms are parser-rejected. A cross-domain reference is retained by
lowering as a constraint and is validated later; lowering alone does not prove that the referenced
domain exists.

## Type functions and normalization

Use `type fn` to declare equations over type patterns. The parser records a type or kind
annotation for each parameter, followed by a result type, optional `decreases` parameter,
optional proposition tail, and one or more `case` equations. The checked normalization route
documented here is the proper-type/sealed-domain subset. A recursive function in that subset must
identify a suitable sealed-domain parameter with `decreases`; recursion must use a direct
structural subcomponent.

**Checked normalization example; not a program.** `task_838_type_function_normalizer.rs`
registers this source-shaped `Append` function and normalizes closed `Nil` and `Cons` inputs.
The page does not claim that this declaration can be run by the Engine.

```ash
type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
    case Append<Nil, ys> = ys;
    case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
}
```

The checker rejects a recursive declaration with no `decreases` clause, an unknown or unsuitable
decreasing parameter, same-argument or rebuilt-argument recursion, and mutual recursion. It also
rejects unresolved heads, wrong-domain constructors, overlapping or unreachable equations, and
non-exhaustive definitions. An open scrutinee does not trigger inverse solving: it normalizes to a
neutral computation application. Fuel or cycle exhaustion is an implementation guard diagnostic,
not a successful normalization and not semantic stuckness.

Public type functions in that checked subset have selected import/summary transport evidence.
Privacy and export closure still constrain that transport; a parser-accepted declaration does not
by itself make its identity available to another module.

### Constructor-kinded binder boundary

The parser preserves constructor-kinded parameter syntax such as `F : * -> *` on both `type fn`
and `prop` declarations. This is a grammar fact, not evidence for the normalizing route above.
`task_906_hkt_kinded_binder_surface.rs` exercises both surfaces, while
`task_906_hkt_fail_closed.rs` shows that registration of a constructor-kinded proposition
predicate is rejected after parsing. No focused evidence here establishes registration or
normalization of a constructor-kinded type-function parameter. It therefore remains a partial
static boundary, not an extension of the checked `Append` example.

## Associated type families

Inside an interface, `sealed type family` declares a type-level associated family with a result
constraint (`Type` or a sealed domain) and an optional decreasing parameter. An implementation's
ordinary `type Name = Type;` member supplies the associated binding. The checker registers
selected closed-world family schemes, enforces ownership, arity, result constraints, non-overlap,
and its recursive boundary;
the normalizer can reduce a selected concrete projection.

**Checked family shape; no method-dispatch claim.** The form below is parser and family-summary
evidence. It is not a complete executable program, nor does it establish general interface-method
execution.

```ash
interface Iterator<I> {
    sealed type family Item: Type
}

impl Iterator<List<A>> {
    type Item = A;
}
```

For a selected local `<Iterator<List<String>>>::Item` projection, the normalizer has a test-backed
reduction to `String`. It first normalizes arguments, so a transparent alias for `List<String>`
can select the same family scheme. A rigid or otherwise unmatched projection remains a boundary:
the checker records a blocked/deferred projection rather than inverting a family to discover its
inputs. Missing/extra bindings, duplicate heads, unauthorized extension outside the owner module,
overlap, invalid result constraints, non-exhaustive rows, and invalid recursion have explicit
diagnostic evidence.

The source parser accepts the family declaration as an interface member. Its lowerer/summary path
is bounded by the closed-world registration and selected module transport tests; it is not a
runtime dictionary or a callable source function. Ordinary interface associated types and generic
interface syntax are documented in [the adjoining types page](generics-kinds-interfaces-and-impls.md).

## Propositions and `where` tails

`prop` declares a named type-level predicate, with zero or more parameters whose domains are
written after `:`. A `where` proposition tail is accepted on `type fn`, `fn`, and `builtin fn`
declarations. Its clauses are equality, disequality, interface-bound, or named-predicate forms;
a proposition tail may also contain one `row { ... }` clause, whose row grammar is owned by
[TASK-2050](../../../plan/tasks/TASK-2050-language-reference-rows-operations-authority.md).

**Parser/static declaration shape.** This declaration is accepted by the proposition-surface
tests. It names a predicate for later static use; it neither runs a proof search nor provides a
runtime boolean.

```ash
pub prop NonEmpty<Xs: TypeList, Witness: Type>;
```

The following is a parser-tested type-function tail, shown as a static declaration fragment. It
uses all four clause forms without asserting that every clause is solvable in every context.

```ash
type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
    where Append<Nil, ys> == ys, Cons<A, T> != Nil, T: Iterator, NonEmpty<xs>
{
    case Append<Nil, ys> = ys;
}
```

The parser rejects proposition tails on ordinary type aliases and reports the stable unsupported
proposition-surface diagnostic there. A proposition can be satisfied, deferred, or rejected by
the checker depending on normalized inputs, registered facts, and interface evidence. In
particular, a neutral computation or rigid associated projection is not solved by inversion, and
the current evidence does not establish unrestricted proof search or SMT solving.

## `data kind` is parser-only

`data kind Name from type Source;` is a distinct declaration that records an intended promoted
data-kind relationship in the surface AST. The parser tests cover private/public declarations and
reject shorthand variants. There is no corresponding current source-to-summary or source-lowering
route in `lower_module_type_metadata`; promoted-constructor registration tests construct their
summary inputs directly. Therefore this page records parser acceptance only.

```ash
type Nat = Z | S(Nat);
pub data kind NatKind from type Nat;
```

This is a complete parser example but not a static-registration, lowering, or execution example.
Do not infer promotion, type-level constructor use, or normalizer behavior from this declaration.
The inactive `dtype` spelling is excluded from the current language surface and has no grammar or
example here.

## Syntax

This grammar covers the current parser slice. `visibility`, `identifier`, and `surface_type` are
shared parser domains. `kind_annotation` is the current kind syntax accepted where the relevant
parser route permits it. `proposition_row_clause` intentionally delegates its body to the rows
chapter rather than duplicating row grammar here. The grammar does not encode checker-only
ownership, structural-recursion, exhaustiveness, normalization-fuel, or visibility side
conditions.

```ebnf
sealed_domain_declaration = [ visibility ] "sealed" "type" "domain" identifier "{" { domain_constructor } "}" ;
domain_constructor = identifier [ "<" [ domain_field { "," domain_field } ] ">" ] ";" ;
domain_field = identifier ":" domain_slot ;
domain_slot = "Type" | identifier ;
type_function_declaration = [ visibility ] "type" "fn" identifier "(" type_function_parameter { "," type_function_parameter } ")" "->" surface_type [ decreases_clause ] [ proposition_tail ] "{" type_function_equation { type_function_equation } "}" ;
type_function_parameter = identifier ":" ( surface_type | kind_annotation ) ;
decreases_clause = "decreases" identifier ;
type_function_equation = "case" identifier "<" type_pattern { "," type_pattern } ">" "=" surface_type ";" ;
type_pattern = "_" | identifier | identifier "<" type_pattern { "," type_pattern } ">" ;
sealed_associated_family_declaration = "sealed" "type" "family" identifier ":" surface_type [ decreases_clause ] ;
proposition_declaration = [ visibility ] "prop" identifier [ "<" proposition_parameter { "," proposition_parameter } ">" ] ";" ;
proposition_parameter = identifier ":" ( surface_type | kind_annotation ) ;
proposition_tail = "where" [ proposition_tail_items [ "," ] ] ;
proposition_tail_items = proposition_clause_sequence [ "," proposition_row_clause [ "," proposition_clause_sequence ] ] | proposition_row_clause [ "," proposition_clause_sequence ] ;
proposition_clause_sequence = proposition_clause { "," proposition_clause } ;
proposition_clause = equality_proposition | disequality_proposition | interface_bound_proposition | named_predicate_proposition ;
equality_proposition = surface_type "==" surface_type ;
disequality_proposition = surface_type "!=" surface_type ;
interface_bound_proposition = surface_type ":" surface_type ;
named_predicate_proposition = identifier [ "<" surface_type { "," surface_type } ">" ] ;
proposition_row_clause = "row" computation_row ;
data_kind_declaration = [ visibility ] "data" "kind" identifier "from" "type" identifier ";" ;
visibility = "pub" | "pub" "(" "crate" ")" ;
```

`type_function_declaration` requires at least one parameter, at least one equation, and at least
one equation pattern in the parser. `"_"` in `type_pattern` is a type-function wildcard, not a
type hole; the latter has distinct, separately bounded parser sites. The
`sealed_associated_family_declaration` has no trailing semicolon in its current interface-member
parser route. In an implementation, the separate ordinary associated-binding syntax is owned by
the interface/implementation page. A tail may contain any number of ordinary proposition clauses
and at most one row clause, in either position among those clauses.

## What the checker does

The normalizer has an exact, narrow source-backed reduction route: a registered type function
first normalizes its arguments, matches an equation, substitutes the matched type-pattern
variables into the equation result, and normalizes that result. The following rule states only
that successful closed/matching path. It does not add inversion, arbitrary equation selection,
termination, admission, or runtime semantics.

```sequent
TypeFunctionClosedStep :=
  [ GAMMA contains registered type function F with equation case F<patterns> = result ] [ normalize(arguments) = normalized_arguments ] [ match(patterns, normalized_arguments) = sigma ] [ normalize(sigma(result)) = normal ]
  ===>
  GAMMA |- normalize(F<arguments>) = normal
```

The rule corresponds to the source-backed `Append` reductions in
`task_838_type_function_normalizer.rs`. In the current normalizer, unmatched/open inputs can
remain neutral; recursive definitions must separately pass the structural `decreases` validation;
and fuel/cycle guards can stop normalization. Associated-family reduction likewise needs a
registered, selected scheme and does not invert rigid projections. The existing evidence is not a
general operational semantics for type declarations or propositions.

## Errors and limits

- Sealed domains reject generic parameters and per-constructor visibility. Their field slots are
  only `Type` or a domain-name reference.
- Type functions reject empty parameter/equation/pattern lists at parsing and reject bad heads,
  domains, coverage, recursion, and decreases clauses during static registration.
- A neutral computation head or rigid family projection is a deferred/blocked normalization
  boundary, not evidence that the checker searched backwards for arguments.
- Associated families require their declared owner, complete/valid bindings, and non-overlapping
  schemes. They do not create a runtime dispatch table.
- Proposition-tail grammar is limited to `type fn`, `fn`, and `builtin fn`; an ordinary type alias
  with such a tail is rejected. Predicate declaration/summary evidence is static, not a blanket
  proof solver.
- `data kind` is parser-only, and `dtype` is excluded. Neither is a current source-execution
  route.

## Related evidence

- [AUDIT-206 LANG-010](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2049](../../../plan/tasks/TASK-2049-language-reference-type-level-computation.md)
- [Types index](index.md)
- `cargo test -p ash-parser --test task_813_sealed_domain_diagnostics --test task_846_public_type_fn_visibility --test task_874_proposition_surface --test task_881_proposition_parse_diagnostics --test task_893_promoted_constructor_parser_surface`
- `cargo test -p ash-typeck --test task_827_normalizer_diagnostics --test task_837_type_function_recursion --test task_838_type_function_normalizer --test task_866_associated_family_normalizer --test task_868_associated_family_diagnostics`
- `cargo test -p ash-engine --test task_811_domain_summary_transport --test task_849_type_computation_summary_transport --test task_867_associated_family_summary_transport`
- `cargo test -p ash-typeck --test task_875_proposition_environment --test task_876_proposition_solver --test task_878_named_predicate_registration --test task_879_proposition_summary_import --test task_880_proposition_checking_points --test task_881_proposition_diagnostics --test task_882_spec_h_acceptance_matrix`
- `cargo test -p ash-engine --test task_879_proposition_summary_transport --test task_880_proposition_public_integration --test task_882_spec_h_transport_non_interference`
