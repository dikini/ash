---
id: language.reference.effects.rows-aliases-groups-and-operations
title: Computation Rows, Effect Aliases, Groups, and Operations
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/**", "crates/ash-engine/src/row_admission.rs"]
---

# Computation Rows, Effect Aliases, Groups, and Operations

[Effects index](index.md) · [Resources and roles](resources-roles-and-authority-boundaries.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Callable computation row | accepted | partial | lowered | closed | partial | tested | below_spec |
| `effect alias` and `effect group` | accepted | checked | lowered | closed | partial | tested | below_spec |
| Concrete operation path / source call | accepted | checked | lowered | closed | partial | tested | below_spec |
| Resource, role, policy, channel, process, failure, evidence, and group row items | accepted | partial | lowered | closed | partial | tested | below_spec |
| Whole-row variable and open tail | accepted | partial | lowered | closed | partial | tested | below_spec |

The parser source is `crates/ash-parser/src/parse_module.rs::{parse_computation_row_from_open_brace,
parse_computation_row_item,parse_effect_row_definition}`. `ash-typeck` registers local aliases
and groups as non-granting summaries, validates their visibility and cycles, and preserves row
facts for the checked route. `ash-engine/src/lib.rs::surface_computation_row_to_core_row` carries
the items into `CoreRow`; `ash-engine/src/row_admission.rs` derives requirements without mutating
authority.

Focused evidence:

- `crates/ash-parser/tests/task_1809_computation_row_parser.rs`
- `crates/ash-parser/tests/task_2001_effect_alias_group_surface.rs`
- `crates/ash-typeck/tests/task_1814_row_cross_boundary_non_authority.rs`
- `crates/ash-typeck/tests/task_2001_local_effect_row_resolution.rs`
- `crates/ash-engine/tests/task_1822_row_authority_neutrality.rs`
- `crates/ash-engine/tests/task_1829_1830_1831_1832_1833_row_admission.rs`
- `crates/ash-engine/tests/task_2011_declared_concrete_operation_source_call.rs`

## What computation rows are for

A computation row is an annotation of requirements. It can appear immediately after a callable
arrow, or in a function's `where row` clause. The annotation says which requirements travel with
the callable's checked metadata; it does not run an operation, select a provider, or make the
callable admitted.

**Checked metadata example; not an executable program.** The parser and checker cover the
row-bearing function shape below. `Audit` is an alias reference that is expanded during row
validation; the group is still a requirement description, not an authority bundle.

```ash
pub effect alias Audit = { evidence audit_log };
pub effect group PublishedAudit = { Audit };

fn inspect() -> Int where row { PublishedAudit } {
    0
}
```

Aliases and groups are not interchangeable concepts internally: the summary marks an alias as
transparent and a group as diagnostic. Both are non-granting, must respect public visibility when
exposed by a public callable, and fail static validation on direct or mutual expansion cycles.
They do not install a provider, resource initializer, role, policy discharge, handler frame, or
runtime module.

## Operation identities

An operation row entry is a path. The checked declared-operation route requires a concrete
implementation identity such as `TestClock::sleep`; the checker rejects an unknown implementation,
unknown method, or a type-mismatched argument before admission. A selected fixture also lowers
that exact source call to a checked CPS inspection `Raise` with its declared argument/result
types. That is lowering evidence, not provider execution evidence.

**Checked/lowered inspection example; not an admitted provider call.**

```ash
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }

fn main() -> Null { TestClock::sleep(0) }
```

`task_2011_declared_concrete_operation_source_call.rs` proves that this source call retains the
singleton `TestClock::sleep` row and a CPS inspection artifact. Its admission test also proves the
opposite of an authority claim: the row alone is rejected because it does not register a matching
provider. Even after a host registers a provider for a row requirement, the selected source
application reaches the current closed checked-Core/CPS admission boundary and no host operation
is invoked.

## Row families and current boundaries

The parser accepts every family in the following table. `lowered` means that the Engine's
surface-to-Core metadata conversion has a matching `CoreRowItem`; it is not execution evidence.

| Source family | What it records | Static / lowering boundary | Admission boundary |
|---|---|---|---|
| operation path | An operation identity, e.g. `PosixFs::read` or `fs.read` | Concrete declared `Impl::operation` identities have checked resolution; other path forms remain partial. The parser accepts `::` and `.`, but the Surface AST retains only the separator before the final segment alongside the complete path segments. | Requires independently registered operation authority; a row never registers it. |
| `resource` | A resource path and optional mode | Parsed and transported to `CoreRowItem::Resource`; path/mode are requirement metadata. | Requires a host-selected resource initializer. It cannot allocate or select one. |
| `role` | A role-name path | Parsed and transported as metadata; source role declarations are not automatically linked to the row. | Requires a matching role already admitted on the request. |
| `policy` | A policy path | Parsed and transported as metadata. | Explicitly unsupported by the current admission substrate and rejects. |
| `channel` | A channel path and optional mode | Parsed; the current parser supplies no payload type and lowering records metadata. | Unsupported and rejects. |
| `process` | An optional process operation | Parsed and transported as metadata. | Unsupported and rejects. |
| `fail` | An optional failure-type path | Parsed and transported as metadata. | Requires failure-handler discharge, which is unsupported and rejects. |
| `evidence` | An evidence name path | Parsed and transported as metadata. | Always rejects in the current admission checker, including for a recognized family: no evidence-record/discharge strategy is implemented. It is never authority. |
| `group` | A named effect-group reference | Parsed and structurally represented; static group resolution is partial and visibility/cycle checked where applicable. | Group expansion is unsupported by current admission and rejects. |
| whole row / tail | A whole-row name or final open-tail variable | The checker validates exported-name resolution and that an open tail is final; broader polymorphic row semantics remain partial. | It does not create a discharge or authority. |

The name `process` is the parser-supported spelling in this row grammar. A source row `channel`
item has no payload grammar in the current parser even though the internal carrier has a payload
field. Internal fields never widen source syntax.

## Syntax

The grammar is the current `parse_computation_row_from_open_brace` slice. A solitary unqualified
name in a row, such as `{r}`, is normalized by the parser to a whole-row entry. An open tail is
introduced by `|` and must be final. A bare name in a larger row can instead be resolved as a
named row declaration by the checker, so the syntax alone does not decide its static meaning.

```ebnf
effect_row_declaration = [ visibility ] "effect" ( "alias" | "group" ) identifier "=" computation_row ";" ;
computation_row = "{" [ whole_row_item [ "," ] | row_items ] "}" ;
whole_row_item = identifier ;
row_items = row_tail | row_item { "," row_item } [ "," ] [ row_tail ] ;
row_tail = "|" identifier ;
row_item = non_whole_row_item | named_row_reference ;
named_row_reference = identifier ;
non_whole_row_item = resource_row_item | role_row_item | policy_row_item | channel_row_item | process_row_item | fail_row_item | evidence_row_item | group_row_item | qualified_operation_row_item ;
resource_row_item = "resource" ( row_mode row_path | row_path [ row_mode ] ) ;
role_row_item = "role" row_path ;
policy_row_item = "policy" row_path ;
channel_row_item = "channel" [ row_mode ] row_path ;
process_row_item = "process" [ identifier ] ;
fail_row_item = "fail" [ row_path ] ;
evidence_row_item = "evidence" row_path ;
group_row_item = "group" row_path ;
qualified_operation_row_item = qualified_operation_row_path ;
qualified_operation_row_path = identifier ( "." | "::" ) identifier { ( "." | "::" ) identifier } ;
row_path = identifier { ( "." | "::" ) identifier } ;
row_mode = "read" | "write" | "split" | "join" | "send" | "receive" | "select" | "close" ;
visibility = "pub" | "pub" "(" "crate" ")" | "pub" "(" "super" ")" | "pub" "(" "self" ")" | "pub" "(" "in" visibility_path ")" ;
visibility_path = identifier { "::" identifier } ;
```

The parser accepts resource modes both before and after a resource path, but only before a channel
path. Omitted resource and channel modes are given internal lowering defaults (`use` and `send`,
respectively); those defaults are not additional source keywords. `whole_row_item` is deliberately
its own alternative and may carry the parser-accepted trailing comma: only a solitary bare
identifier is normalized as a whole row. The `row_items` sequence also permits the parser's
comma-before-tail form, such as `{ Audit, | r }`. A bare name in a multi-item row or before an
open tail is the separate `named_row_reference` form and must still pass checker resolution. The
EBNF deliberately omits checker-only resolution, visibility, cycle, operation-signature, and
tail-finality constraints.

## Semantics: requirement derivation is non-granting

The precise current semantic boundary is the Engine's conversion from a lowered `CoreRow` to a
list of `RowAdmissionRequirement` values. It reads existing Engine/request state during checking;
it does not mutate that state. The following rule records that transport boundary rather than
inventing a source evaluator.

```sequent
RowRequirementNonGranting :=
  [ derive_requirements(core_row) = requirements ] [ authority_before = A ]
  ===>
  admit_check(requirements, A) leaves authority_after = A
```

For an operation requirement, the separate admission check succeeds only when a provider was
already registered (or separately evidenced); for a resource it needs a selected initializer;
for a role it needs a role already carried by the request. Evidence is different: the current
`RowAdmissionCheck` always rejects an evidence requirement, including a recognized family,
because no record/discharge strategy is implemented. The corresponding tests assert that rows do
not add authority facts and do not call host `observe` or `execute` hooks.

## Errors and limits

- A public callable cannot expose a private local alias/group, including through an otherwise
  public expansion chain.
- A direct or mutual alias/group cycle fails static row resolution.
- A concrete operation with an unknown implementation, unknown method, or bad argument type
  fails before admission.
- Missing operation authority, resource initialization, or admitted role yields a fail-closed
  admission rejection. Supplying one of those facts only satisfies that requirement; it does not
  bypass the separate application admission boundary.
- Policy, channel, process, failure, and group requirements have no current supported admission
  discharge. Evidence requirements always fail closed in the current checker, even with a
  recognized family: no record/discharge strategy route is implemented.
- The manual does not provide a copyable source declaration for a provider, direct capability
  grant, or policy declaration. Those are not current language-reference examples.

## Related evidence

- [Effects index](index.md)
- [Resources, roles, and authority boundaries](resources-roles-and-authority-boundaries.md)
- [TASK-2050](../../../plan/tasks/TASK-2050-language-reference-rows-operations-authority.md)
- `cargo test -p ash-parser --test task_1809_computation_row_parser --test task_2001_effect_alias_group_surface`
- `cargo test -p ash-typeck --test task_1814_row_cross_boundary_non_authority --test task_2001_local_effect_row_resolution`
- `cargo test -p ash-engine --test task_1822_row_authority_neutrality --test task_1829_1830_1831_1832_1833_row_admission --test task_2011_declared_concrete_operation_source_call`
