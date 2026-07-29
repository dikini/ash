# Data Types, Newtypes, Callable Types, and Capability Types

[Types index](index.md) · [Generics, kinds, interfaces, and implementations](generics-kinds-interfaces-and-impls.md) ·
[Language reference](../index.md)

## Status and evidence

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Ordinary `type` declaration | accepted | partial | lowered | closed | partial | tested | below_spec |
| Nominal `newtype` declaration and singleton constructor/pattern checking | accepted | checked | lowered | closed | partial | tested | below_spec |
| Callable annotation and alias arrows | accepted | partial | lowered | closed | partial | tested | below_spec |
| Pure closure type | accepted | checked | lowered | closed | partial | tested | below_spec |
| `capability Name` source type | accepted | partial | lowered | fixture-bounded | partial | tested | below_spec |

The type-declaration parser is `crates/ash-parser/src/parse_type_def.rs`; the module parser
attaches it through `crates/ash-parser/src/parse_module.rs::parse_type_definition`. The distinct
newtype parser is `parse_newtype_definition`. `crates/ash-parser/src/lower.rs` transports ordinary
type declarations and newtype summaries into Core/summary metadata, while
`crates/ash-typeck/src/{lib.rs,type_env}` owns nominal registration and checking. Callable type
spelling is parsed by `parse_surface_type_with_holes`; pure closure checking is in
`check_expr`. `surface_type_lowering` maps a capability type to the operational capability type,
and `ash-engine/src/entry.rs` has the narrow entry verification/binding use.

Focused tests include:

- `crates/ash-parser/tests/task_782_modulefile_type_surface.rs`
- `crates/ash-parser/tests/task_957_callable_type_parser.rs`
- `crates/ash-parser/tests/task_960_reserved_callable_arrows.rs`
- `crates/ash-typeck/tests/task_959_pure_closure_arrow.rs`
- `crates/ash-engine/tests/task_2001_local_nominal_newtype_checking.rs`
- `crates/ash-engine/tests/task_2001_nominal_newtype_match_patterns.rs`
- `crates/ash-engine/tests/entry_verification.rs`

The newtype tests exercise the normal Engine parse/check boundary, not source execution. The
entry test verifies a capability-typed `main` parameter and creates a name-only `Value::Cap`
input binding. Neither fact is general type-declaration execution or an authority grant.

## Ordinary `type` declarations

Use `type` to name a record body, a variant family, or an alias. A declaration may be public or
crate-visible and may declare ordinary type parameters. Variant bodies provide the constructor
metadata that current lowering exports. A record-bodied declaration, by contrast, is not evidence
that its type name can be called as a value constructor.

**Parser/lowering declaration shape.** The module-file test preserves this type definition and
the parser records the two variants. It is a declaration example, not an admitted program.

```ash
pub type Result<T, E> =
    Ok { value: T }
  | Err { error: E };
```

Ordinary aliases are transparent in the selected checker/import route. For example, the imported
`pub type Counter = Int;` fixture in
`task_2001_imported_nominal_newtype_checking.rs` is accepted where `Int` is expected. That does
not make every alias body, record, or variant application runnable through the Engine.

`builtin type Name;` is the parser-supported bodyless declaration form. It is an opaque substrate
boundary: a normal `type Name;` is rejected, and the current newtype checker rejects an opaque,
bodyless representation as uninhabited. Do not use a bodyless ordinary `type` declaration as a
current example.

## Nominal `newtype` declarations

Use `newtype` for a one-field nominal wrapper with an explicit tuple constructor. The checker
registers the wrapper as distinct from its representation and from sibling wrappers. A constructor
call must receive exactly the declared representation type; a tuple pattern binds exactly one
representation value.

**Checked source shape; no execution claim.** The local-newtype Engine test checks this style of
program. It does not execute the function or demonstrate runtime representation erasure.

```ash
newtype OrderId = OrderId(Int);

fn next_id(order: OrderId) -> Int {
    let OrderId(value) = order;
    value + 1
}
```

The current checker rejects either direction of implicit coercion between `OrderId` and `Int`, a
different wrapper such as `CustomerId`, a wrong constructor/pattern arity, wrong payload type,
recursive representations, collisions with ordinary local declarations, and an opaque bodyless
representation. Direct public imports and one-hop public re-exports have selected checked
evidence; generic newtypes, multi-hop/unproved routes, and all runtime representation/execution
claims remain outside this page's evidence.

## Callable type spellings and closure types

Callable types use `->`. In general surface-type positions, `A -> B` denotes one argument and
`(A, B) -> C` denotes two arguments; the latter is not a one-argument tuple domain. The parser
test also distinguishes `(Pair) -> Bool`, which has one `Pair` parameter, from a two-parameter
callable.

**Parser-only function annotation.** This exact parenthesized two-parameter shape is covered by
`task_957_callable_type_parser.rs`. It is not a promise that values of the type can be called by
an admitted Engine program.

```ash
fn keep(f: (Int, String) -> Bool) -> Bool { true }
```

The ordinary alias parser has a narrower rule than general surface-type parsing: use a
parenthesized domain even for one argument. This current alias spelling is parser-tested:

```ash
type Predicate = (Int) -> Bool;
```

Do not rewrite it as `type Predicate = Int -> Bool;`; that unparenthesized form is not accepted
by the ordinary `type`-body parser. A pure closure expression has a checked `Fn` type in the
separate expression route:

```ash
|x: Int| -> x + 1
```

The closure example is an expression fragment checked by
`task_959_pure_closure_arrow.rs`, including under an ambient operational profile. It is not a
complete module or a runtime example. Historical `Fn(Int, String) -> Bool`, `-*>`, `=>`, and
`=*>` spellings are rejected as callable forms. `=>` remains valid only in a match arm.

Callable result rows are parsed after `->`; a row prefix can also form a zero-argument general
surface callable. Their grammar, normalization, and authority boundary belong to
[TASK-2050's rows documentation](../../../plan/tasks/TASK-2050-language-reference-rows-operations-authority.md).
The presence of a row does not make the callable executable or grant an authority.

## `capability Name` is a source type, not a declaration

`capability Name` is accepted only in a type position. The parser creates a type carrier, and
surface type lowering maps it to an operational capability type. The Engine's entry verifier
accepts capability-typed parameters and creates a name-only capability value for each such
parameter.

**Selected entry-verification shape.** This is the capability parameter form exercised by
`entry_verification.rs`; its imports and complete entry contract are documented by TASK-2052.
It is not a provider selection or execution example.

```ash
fn main(args: capability Args) -> Result<(), RuntimeError> {
    Ok { value: {} }
}
```

The spelling does not declare a capability, choose a provider, grant permission, add an operation
row, or install a runtime binding with authority. Current top-level `capability` declarations are
removed source forms even though historical/internal carriers remain elsewhere. TASK-2050 owns
the later resources, roles, and authority-boundaries page.

## Syntax

This grammar records the current slice relevant to ordinary declarations and callable type
spellings. `type_alias_expression` is the `parse_type_def::parse_type_expr` domain; it has no
computation-row result form. `surface_type` is the broader module-parser domain used in
annotations and newtype payloads. `identifier` and `computation_row` are shared domains; the
latter is owned by TASK-2050. The ordinary alias rule intentionally shows only the
parenthesized callable spelling accepted by `parse_type_def`.

```ebnf
ordinary_type_declaration = [ visibility ] "type" type_name [ ordinary_type_parameters ] "=" ordinary_type_body ";" ;
builtin_type_declaration = [ visibility ] "builtin" "type" type_name [ ordinary_type_parameters ] [ "=" ordinary_type_body ] ";" ;
ordinary_type_parameters = "<" type_parameter { "," type_parameter } ">" ;
ordinary_type_body = record_body | enum_body | type_alias_expression ;
record_body = "{" [ field { "," field } ] [ "," ] "}" ;
field = identifier ":" type_alias_expression ;
enum_body = payload_variant | variant "|" variant { "|" variant } ;
variant = variant_name | payload_variant ;
payload_variant = variant_name record_body | variant_name "(" [ type_alias_expression { "," type_alias_expression } ] ")" ;
type_alias_expression = parenthesized_alias_callable_type | type_atom { "::" type_name } ;
parenthesized_alias_callable_type = "(" [ type_alias_expression { "," type_alias_expression } ] ")" "->" type_alias_expression ;
type_atom = type_name | type_constructor | tuple_type | record_body | associated_family_projection ;
type_constructor = type_name "<" type_alias_expression { "," type_alias_expression } ">" ;
tuple_type = "(" type_alias_expression { "," type_alias_expression } ")" ;
associated_family_projection = "<" type_name "<" type_alias_expression { "," type_alias_expression } ">" ">" "::" type_name ;
newtype_declaration = [ visibility ] "newtype" type_name [ newtype_parameters ] "=" constructor_name "(" surface_type ")" ";" ;
newtype_parameters = "<" newtype_parameter { "," newtype_parameter } ">" ;
newtype_parameter = identifier [ ":" kind ] ;
surface_zero_argument_callable_type = computation_row surface_type ;
surface_callable_type = surface_type_atom "->" surface_callable_result | "(" [ surface_type { "," surface_type } ] ")" "->" surface_callable_result ;
surface_callable_result = [ computation_row ] surface_type ;
surface_type_atom = capability_type | list_type | surface_tuple_type | surface_record_type | surface_named_type | associated_family_projection ;
list_type = "[" surface_type "]" ;
surface_tuple_type = "(" [ surface_type { "," surface_type } ] ")" ;
surface_record_type = "{" [ surface_field { "," surface_field } ] "}" ;
surface_field = identifier ":" surface_type ;
surface_named_type = identifier [ "<" surface_type { "," surface_type } ">" ] { "::" identifier } ;
capability_type = "capability" identifier ;
visibility = "pub" | "pub" "(" "crate" ")" ;
```

`type_name`, `variant_name`, and `constructor_name` have source naming constraints enforced by the
parser. EBNF does not encode the parser's enum-versus-alias disambiguation or checker-only
inhabitation, nominal-identity, and kind side conditions.

## Semantics and implementation boundary

The first rule summarizes the narrow nominal-constructor check exercised by the current local and
imported newtype tests. The second is the exact source-type lowering relationship used for a
capability type. Both are static/lowering statements; neither is an admission, provider, or
runtime-authority rule.

```sequent
NominalNewtypeIntro :=
  [ GAMMA contains newtype N = C(R) ] [ GAMMA |- value : R ]
  ===>
  GAMMA |- C(value) : N
```

```sequent
CapabilityTypeLower :=
  [ source_type = capability K ]
  ===>
  lower_type(source_type) = Capability(K, Operational)
```

The newtype rule has explicit current limits: its checked fixture domain is a single tuple
constructor with exact nominal identity. The capability lowering rule describes only the type
carrier; the entry binder later maps a verified parameter to `Value::Cap(K)`, and that transport
still grants no authority. There is no evidence-backed general evaluation, coercion, interface
dispatch, or type-declaration execution rule to state here.

## Diagnostics and boundaries

- A normal `type` declaration needs a body; only the `builtin type` form may end after its name.
- A newtype payload must match its representation exactly. Nominal wrappers do not coerce to or
  from their representation or another wrapper.
- Current generic newtype syntax may parse, but the nominal checker does not establish a general
  generic newtype route.
- Parentheses decide the callable spelling in an ordinary type alias. An unparenthesized unary
  arrow is available in general annotation positions, not in `type Alias = ...`.
- Callable arrows, closure typechecking, and capability type lowering do not prove a callable or
  a capability provider is admitted for execution.
- `dtype`, source `raise`, workflow/tower callable forms, `Fn(...)`, `-*>`, `=>`, and `=*>` are
  not current callable/type examples. The exception is match-arm `=>`, which is documented with
  patterns.

## Related evidence

- [AUDIT-206 LANG-008 and LANG-020](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2048](../../../plan/tasks/TASK-2048-language-reference-ordinary-types-interfaces.md)
- `cargo test -p ash-parser --test task_782_modulefile_type_surface --test task_957_callable_type_parser --test task_960_reserved_callable_arrows`
- `cargo test -p ash-typeck --test task_959_pure_closure_arrow --test task_2001_local_newtype_identity`
- `cargo test -p ash-engine --test task_2001_local_nominal_newtype_checking --test task_2001_nominal_newtype_match_patterns --test entry_verification`
