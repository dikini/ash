---
id: language.reference.types.generics-kinds-interfaces-and-impls
title: Generics, Kinds, Interfaces, and Implementations
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/**", "crates/ash-engine/src/module_loader/**"]
---

# Generics, Kinds, Interfaces, and Implementations

[Types index](index.md) · [Data types, newtypes, callable types, and capability types](data-newtypes-and-callables.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Generic callable signatures | accepted | checked | lowered | closed | partial | tested | below_spec |
| Kinded interface binders and constructor applications | accepted | partial | rejected | closed | partial | tested | below_spec |
| Interface declarations and evidence constraints | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| Explicit interface implementations | accepted | partial | bounded-only | closed | partial | tested | below_spec |

The parser routes are `crates/ash-parser/src/parse_module.rs::{parse_interface_definition,
parse_impl_definition,parse_optional_interface_type_params}`. Static registration and
closed-world evidence are in `ash-typeck/src/type_env/associated_families_and_capabilities.rs`;
the generic callable-signature path is in `ash-typeck`'s signature lowering. Core lowerers reject
kinded interface/implementation parameters before producing their ordinary Core definitions.

Focused tests include:

- `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs`
- `crates/ash-typeck/tests/task_1971_generic_signature_type_params.rs`
- `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs`
- `crates/ash-typeck/tests/task_906_hkt_fail_closed.rs`
- `crates/ash-typeck/tests/closed_world_interfaces_task_422.rs`
- `crates/ash-engine/tests/task_1021_std_algebra_namespace_and_interfaces.rs`
- `crates/ash-engine/tests/task_1041_interface_constraint_summary_transport.rs`

The Engine tests establish selected parsing, checking, import/summary transport, and stdlib
constraint evidence. They do not establish arbitrary interface-method dispatch, generic method
execution, or a general runtime dictionary model.

## Generic parameters and kind annotations

Generic callable signatures bind the parameter before the signature is lowered. The current
`fn keep<a>(items: List<a>) -> List<a>` test verifies that the two `a` occurrences refer to the
same fresh checker variable. Generic binders may carry a kind annotation where that parser route
allows it.

Interfaces and implementation heads also accept constructor-kinded binders, such as
`F : * -> *`. A kinded constructor may be applied at the arity it declares; a bare constructor
where a proper type is needed, an application of a proper type variable, or a wrong number of
arguments is rejected after parsing.

**Checked interface shape; no general runtime claim.** This is the parser/typechecker shape used
by the HKT tests. It is not a runnable interface program.

```ash
interface Functor<F : * -> *> {
    map(F<Int>) -> F<Int>
}
```

An ordinary `type` alias does not share this kinded-binder route: `type Alias<F : * -> *> =
F<Int>;` is rejected. Type-level domains/families and their more general kind semantics are owned
by TASK-2049, not inferred from this parser evidence.

## Interfaces

An `interface` names a collection of method signatures, optional ordinary associated type
declarations, optional evidence constraints, and parser-supported law declarations. Its
signatures provide static information for an implementation/evidence lookup; they do not by
themselves create a callable value or runtime method table.

**Checked evidence shape; no dispatch claim.** The HKT acceptance matrix registers this style of
`Monad<Option>` evidence after its interface is registered. The handler, `do`, and method/runtime
consequences remain independently bounded and belong to TASK-2050 and TASK-2051.

```ash
interface Monad<M : * -> *> {
    return(Int) -> M<Int>
}

impl Monad<Option> {
    return(value) = Some { value: value }
}
```

Interface-level `where` constraints and associated type declarations are source-visible. Their
current typechecking and summary transport is partial. A `sealed type family` member is a
type-level form documented by TASK-2049; it is not expanded on this page.

## Implementations and the closed-world boundary

An `impl` states evidence for one interface head and can give associated type bindings and method
bodies. The current checker requires the interface to be registered first, checks that the number
and kinds of head arguments match, rejects invalid non-generic targets at the concrete-nominal
boundary, and rejects duplicate or overlapping evidence. A parser-valid implementation can
therefore fail static registration.

The kinded `impl` boundary is intentionally narrower than the parser: a kinded implementation
parameter parses but the static registration and Core lowering paths reject that generalization.
The checked partial-application case `impl <E : *> Monad<Result<_, E>> {}` is retained as shape
evidence only; it must not be represented as a generalized runtime method implementation.

The current standard-library algebra tests show that named interfaces can be parsed, checked, and
imported through the module-loader summary route. They do not make `Interface::method` calls or
arbitrary `impl` bodies executable. The selected `do` evidence test is also a static boundary, not
a transferable runtime dispatch proof.

## Syntax

The grammar records the accepted interface/implementation surface. `surface_type`, `expression`,
`law_declaration`, handler members, and sealed associated families are shared domains and are not
redefined here. `impl_head_surface_type` is a distinct parser domain: it has the same structural
surface-type alternatives but permits `_` recursively only while parsing the angle-bracketed
arguments immediately after an `impl` interface name. A valid parsed implementation still must
pass the closed-world and overlap checks.

```ebnf
interface_declaration = [ visibility ] "interface" interface_name [ interface_parameters ] [ interface_where_clause ] "{" { interface_member [ "," ] } "}" ;
interface_parameters = "<" interface_parameter { "," interface_parameter } ">" ;
interface_parameter = identifier [ ":" parameter_domain ] ;
parameter_domain = kind | surface_type ;
kind = kind_atom [ "->" kind ] ;
kind_atom = "*" | "Prop" | "Row" ;
interface_where_clause = "where" interface_evidence_constraint { "," interface_evidence_constraint } ;
interface_evidence_constraint = surface_type ":" surface_type ;
interface_member = associated_type_declaration | interface_method_signature | law_declaration | sealed_associated_family ;
associated_type_declaration = "type" identifier ";" ;
interface_method_signature = identifier "(" [ surface_type { "," surface_type } ] ")" "->" surface_type ;
implementation_declaration = [ visibility ] "impl" [ interface_parameters ] interface_name [ impl_head_type_arguments ] [ "for" surface_type ] [ implementation_where_clause ] "{" { implementation_member [ "," ] } "}" ;
impl_head_type_arguments = "<" impl_head_surface_type { "," impl_head_surface_type } ">" ;
impl_head_surface_type = impl_head_zero_argument_callable_type | impl_head_callable_type | impl_head_type_atom ;
impl_head_zero_argument_callable_type = computation_row impl_head_surface_type ;
impl_head_callable_type = impl_head_type_atom "->" impl_head_callable_result | "(" [ impl_head_surface_type { "," impl_head_surface_type } ] ")" "->" impl_head_callable_result ;
impl_head_callable_result = [ computation_row ] impl_head_surface_type ;
impl_head_type_atom = type_hole | capability_type | "[" impl_head_surface_type "]" | impl_head_tuple_type | impl_head_record_type | impl_head_named_type | impl_head_associated_family_projection ;
type_hole = "_" ;
impl_head_tuple_type = "(" [ impl_head_surface_type { "," impl_head_surface_type } ] ")" ;
impl_head_record_type = "{" [ impl_head_record_field { "," impl_head_record_field } ] "}" ;
impl_head_record_field = identifier ":" impl_head_surface_type ;
impl_head_named_type = identifier [ "<" impl_head_surface_type { "," impl_head_surface_type } ">" ] { "::" identifier } ;
impl_head_associated_family_projection = "<" identifier "<" impl_head_surface_type { "," impl_head_surface_type } ">" ">" "::" identifier ;
implementation_where_clause = "where" where_bound { "," where_bound } ;
where_bound = identifier ":" identifier ;
implementation_member = associated_type_binding | implementation_method | handler_declaration | derived_handler_declaration | proof_declaration ;
associated_type_binding = "type" identifier "=" surface_type ";" ;
implementation_method = identifier "(" [ identifier { "," identifier } ] ")" "=" expression ;
derived_handler_declaration = "derive" "handler" identifier ";" ;
visibility = "pub" | "pub" "(" "crate" ")" ;
```

The `for` alternative is parser-supported in an implementation head; checker constraints still
decide whether the resulting head is valid. The separate `impl_head_surface_type` domain models
the deliberate hole-policy exception: `impl <E : *> Monad<Result<_, E>> {}` parses, while `_` is
rejected in routine function, interface-method, and other ordinary surface-type positions.
Ordinary type aliases do not accept kind annotations.

## What the checker does

The checker has multiple rule-shaped registration paths, but their exact side conditions include
kind resolution, interface lookup, argument arity, target restrictions, evidence constraints, and
overlap detection. The evidence here is not broad enough to collapse those checks into a single
general sequent without hiding rejecting cases. This page therefore states no synthetic interface
or implementation sequent.

The narrow static consequence is: a successfully registered implementation contributes checked
interface evidence in the closed-world `TypeEnv`; it does not install a runtime dictionary,
provider, handler frame, capability grant, or Engine dispatch target. Core lowering further
rejects kinded interface/implementation parameters. No general admission/runtime semantics is
established for interface methods or implementation bodies.

## Errors and limits

- A malformed kind annotation, an unsupported kinded ordinary type alias, and holes in ordinary
  type positions are parser diagnostics.
- A parser-valid interface/implementation can be rejected for a missing interface, wrong arity or
  kind, non-concrete target, duplicate implementation, or overlap.
- Generic callable signatures have checked lowering evidence; kinded interface/implementation
  lowerers remain deliberately closed at their Core boundary.
- Associated families, proposition/law semantics, type functions, and sealed domains are not
  ordinary interface dispatch and are documented by TASK-2049 or their owning chapter.
- Neither an interface declaration nor registered evidence grants authority. Rows, provider
  bindings, handlers, and Engine admission remain separately owned boundaries.
- Historical workflow/tower interfaces and `dtype` are excluded from the current language surface.

## Related evidence

- [AUDIT-206 LANG-009](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2048](../../../plan/tasks/TASK-2048-language-reference-ordinary-types-interfaces.md)
- `cargo test -p ash-parser --test task_910_hkt_diagnostics_surface`
- `cargo test -p ash-typeck --test task_1971_generic_signature_type_params --test task_910_hkt_acceptance_matrix --test task_906_hkt_fail_closed --test closed_world_interfaces_task_422`
- `cargo test -p ash-engine --test task_1021_std_algebra_namespace_and_interfaces --test task_1041_interface_constraint_summary_transport`
