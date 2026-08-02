---
id: language.reference.effects.resources-roles-and-authority-boundaries
title: Resource Types, Roles, and Authority
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/**", "crates/ash-engine/src/**"]
---

# Resource Types, Roles, and Authority

[Effects index](index.md) · [Rows, aliases, groups, and operations](rows-aliases-groups-and-operations.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `resource type` declaration | accepted | checked | not-applicable | closed | partial | tested | below_spec |
| `role` declaration | accepted | partial | not-applicable | closed | partial | tested | below_spec |
| Resource row requirement | accepted | partial | lowered | closed | partial | tested | below_spec |
| Role row requirement | accepted | partial | lowered | closed | partial | tested | below_spec |

`parse_resource_type_definition` parses resource fields; `TypeEnv::register_resource_type`
checks duplicate field names and field types. `parse_role_definition` preserves role names,
capability names/optional constraints, and obligation names. The production lowering path does
not turn a resource declaration or role declaration into authority: the retained role-to-Core
helper is compiled only under `cfg(test)`. Engine admission sees row metadata separately through
`RowAdmissionRequirement`.

Focused evidence:

- `crates/ash-parser/tests/phase_101_resource_binding_parser.rs`
- `crates/ash-parser/src/parse_module/tests.rs::test_parse_inline_module_with_role_definition`
- `crates/ash-typeck/src/type_env/associated_families_and_capabilities.rs::register_resource_type`
- `crates/ash-engine/tests/task_1822_row_authority_neutrality.rs`
- `crates/ash-engine/tests/task_1829_1830_1831_1832_1833_row_admission.rs`

## Resource types

A resource type is a named static description with named, typed fields. Use it to declare the
shape that static checking registers; it does not construct an instance, allocate storage, select
an initializer, or give a function access to a resource.

**Parser/static declaration example; no allocation or runtime claim.**

```ash
pub resource type ReviewKV {
    path: String
}
```

The parser requires a name and a colon in each field. The checker rejects a duplicate field name
or a field type that cannot be resolved as an ordinary type. None of those facts chooses a
resource initializer. In contrast, a row entry such as `resource vault write` is only a request
for an independently host-selected initializer named `vault`; it is not linked automatically to
a same-named resource-type declaration.

## Roles

A role declaration records a role name, optional capability-name entries, optional constraints,
and optional obligation names. It is accepted source metadata, not an active role assignment or
an admission grant.

**Parser-only metadata example; no role is admitted by this declaration.**

```ash
role reviewer {
    capabilities: [approve, review],
    obligations: [check_tests, audit_log]
}
```

The capabilities inside this form are identifiers in role metadata. They are not top-level
capability declarations, provider selections, or executable grants. The separate current
type-position form `capability Name` is documented in the
[types chapter](../types/data-newtypes-and-callables.md#capability-name-is-a-source-type-not-a-declaration);
it likewise does not grant authority.

A row requirement is separate again:

```ash
fn review() -> String where row { role tenant.reviewer } {
    "pending"
}
```

This example is parser/metadata evidence only. The Engine checks `role tenant.reviewer` against
an already-admitted role on the admission request; it does not search this module for a `role`
declaration, create a role, or merge role metadata into the request. With no matching admitted
role the check rejects. With one, the row check passes but the selected application still reaches
the distinct closed checked-Core/CPS admission boundary.

## Syntax

This is the active parser slice for resource types and roles. `surface_type` and
`constraint_block` are shared parser domains, so their detailed grammar belongs to the types and
constraint documentation rather than being silently widened here.

```ebnf
resource_type_declaration = [ visibility ] "resource" "type" identifier "{" [ resource_field { "," resource_field } ] "}" ;
resource_field = identifier ":" surface_type ;
role_declaration = "role" identifier "{" [ role_capabilities ] [ "," ] [ role_obligations ] [ "," ] "}" ;
role_capabilities = "capabilities" ":" "[" [ role_capability { "," role_capability } ] "]" ;
role_capability = identifier [ "@" constraint_block ] ;
role_obligations = "obligations" ":" "[" [ identifier { "," identifier } ] "]" ;
visibility = "pub" | "pub" "(" "crate" ")" | "pub" "(" "super" ")" | "pub" "(" "self" ")" | "pub" "(" "in" visibility_path ")" ;
visibility_path = identifier { "::" identifier } ;
```

The role parser permits an empty body, capabilities only, obligations only, or both in that
order. It does not make an omitted clause a default grant. `resource type` fields have ordinary
surface types, and the grammar does not introduce a resource constructor expression.

## Authority and admission boundary

The Engine's row admission code is deliberately a checker over pre-existing state:

- An operation row needs a provider already registered with the Engine.
- A resource row needs an initializer selected by the Engine builder; the row does not select it.
- A role row needs a matching role already attached to the admission request; the row does not
  admit it.
- A policy row is unsupported. Evidence rows always reject in the current admission checker,
  including a recognized family, because no evidence-record/discharge strategy is implemented.
  Channel, process, failure, and group rows are likewise not current general discharges.

The authority-neutrality tests additionally prove that parsing/importing row-bearing source and
performing row admission neither registers a provider nor a resource initializer, installs a
runtime module/handler frame, selects a capability implementation, or invokes host hooks.

There is therefore no implementation-backed source rule that derives authority from a resource
type, role declaration, row, alias, group, or imported summary. The `RowRequirementNonGranting`
rule on [the rows page](rows-aliases-groups-and-operations.md#semantics-requirement-derivation-is-non-granting)
states the available formal transport boundary.

## Errors and limits

- `resource type` without a name, or a resource field without `:`, is rejected by the parser.
- Duplicate resource fields and invalid ordinary field types fail static registration.
- `role` metadata can parse without a production authority/lowering route; do not treat the
  retained test-only lowerer as runtime evidence.
- A resource row missing a host-selected initializer or a role row missing an admitted role
  rejects at admission. Supplying either fact does not itself admit or execute the source program.
- This manual intentionally contains no source example for a direct provider grant, top-level
  capability declaration, or top-level policy declaration. Those are not current forms to teach.

## Related evidence

- [Effects index](index.md)
- [Rows, aliases, groups, and operations](rows-aliases-groups-and-operations.md)
- [TASK-2050](../../../plan/tasks/TASK-2050-language-reference-rows-operations-authority.md)
- `cargo test -p ash-parser --test phase_101_resource_binding_parser --test task_1809_computation_row_parser`
- `cargo test -p ash-typeck --test task_1814_row_cross_boundary_non_authority`
- `cargo test -p ash-engine --test task_1822_row_authority_neutrality --test task_1829_1830_1831_1832_1833_row_admission`
