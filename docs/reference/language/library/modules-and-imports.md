---
id: language.reference.library.modules-and-imports
title: Standard Library Modules and Imports
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["std/src/**", "crates/ash-engine/src/module_loader.rs", "crates/ash-engine/src/entry.rs", "crates/ash-engine/tests/**"]
---

# Standard Library Modules and Imports

[Library and diagnostics](index.md) · [Diagnostics and errors](diagnostics-and-errors.md) ·
[Module grammar](../lexical-and-modules/modules-imports-and-visibility.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| All 59 `std/src/**/*.ash` files | accepted | checked | bounded-only | closed | partial | tested | below_spec |
| Ordinary module-loader resolution of a stdlib import | parser-only | partial | bounded-only | closed | partial | tested | below_spec |
| `json::{parse,stringify,stringify_pretty}` import route | parser-only | checked | bounded-only | closed | partial | tested | below_spec |
| Runtime-entry registry imports | parser-only | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Exact `time::sleep` Engine fixture | accepted | checked | lowered | admitted-executed | partial | tested | below_spec |

`crates/ash-cli/tests/stdlib_corpus_check.rs` checks 59 files in `std/src`; every file passes
`ash check`. `crates/ash-parser/tests/stdlib_parsing.rs` adds parser tests. Ordinary imports use
`crates/ash-engine/src/module_loader.rs` and `module_loader/import_resolution.rs`.

`json_stdlib_e2e.rs` parses and checks selected JSON imports, then the Engine rejects them because
it has no production lowering for them. The only positive standard-library example here is the
`time::sleep` path in `crates/ash-engine/src/production_cps_driver.rs`.

## What the standard library is and how to use it

Ash has 59 `.ash` files under `std/src`. They are ordinary modules. Use an explicit `use` import,
then check the module's support table before relying on it at runtime. For a non-local import, the
resolver searches
the importing directory, override/environment dependency roots, the built-in `std/src` root, then
discovered locked-project vendor/cache roots. It transports selected public type definitions,
semantic summaries, callable declarations, and macro summaries; it does not make every
transported callable runnable.

The installed-root test proves that an ordinary loader can resolve a public type from a configured
stdlib root. It does not prove that a package's functions execute. In particular, a declaration in
`std/src/lib.ash` is an export inventory rather than a runtime prelude.

There is **no implemented automatic-prelude import**. `std/src/prelude.ash` is a source module;
the statement that it is automatically imported is stale. Write a supported explicit import and
keep the direct-parser semicolon rule in mind. The loader may normalize a missing semicolon only
for its leading-import convenience route; see [Module grammar](../lexical-and-modules/modules-imports-and-visibility.md#syntax).

## Module inventory and evidence matrix

The inventory below is organized by source location so it remains navigable without falsely
turning the source tree into a runtime-support matrix.

| Module set in `std/src` | Source inventory | Parser/static evidence | Runtime conclusion |
|---|---|---|---|
| Root modules: `lib`, `evidence`, `http`, `json`, `list`, `logging`, `map`, `markdown`, `option`, `predicate`, `prelude`, `process`, `record`, `regex`, `result`, `string`, `test`, and `time` | Public declarations and re-exports exist in the source corpus. `lib.ash` is the root re-export inventory. | The 59-file `ash check` baseline covers these files; targeted parser tests cover representative core modules. | No blanket runtime conclusion. Only the exact `time::sleep` fixture below is admitted/executed evidence. JSON imports are explicitly closed at admission. |
| `algebra/**` | Eight algebra files plus `algebra/mod.ash`. | Included in the corpus baseline; selected constraint tests inspect static interface/implementation evidence. | No general execution claim for interface methods, proofs, or helpers. |
| `io/{buf,dir,fs,meta,path,stdio,mod}.ash` | I/O source declarations and re-exports. | Corpus and parser coverage establish source visibility; `io_stdlib_wiring_test.rs` checks selected declaration/wiring conditions. | No current general source-to-admission claim in this manual. A host provider declaration is not execution authority. |
| `llm/**` | Conversation, dispatch, loading, OpenAI, prompt, supervised, tool-agent, and type modules. | Included in the corpus baseline. | No generic LLM execution route is established here. |
| `runtime/{args,error,mod,supervisor}.ash` | `Args`, `RuntimeError`, and related source declarations. | Runtime parser tests cover the declared spelling. | Only `runtime::RuntimeError` and `runtime::Args` are accepted in the narrow entry-import registry; `runtime::system_supervisor` is rejected there. |
| `test/**` and `test/quickcheck/**` | Test/evidence and quickcheck source modules. | Included in the corpus baseline. | They do not make source test metadata, laws, proofs, or quickcheck helpers executable on the Engine route. |

The table intentionally groups source modules rather than listing every exported name. For exact
source declarations, consult the named `.ash` file and verify its route before treating it as a
supported runtime API.

## Examples

### Ordinary import — static/module route only

This is the direct `parse_use` spelling: its semicolon is required by the direct parser. It is an
import example, not proof that calling `parse` will be admitted for execution.

```ash
use json::{parse};
```

The explicit test `json_stdlib_e2e.rs` constructs `use json::{parse}` source that parses and
checks, then asserts the production admission boundary rejects it.

### Narrow runtime-entry import registry — registry evidence only

When an Engine has loaded its runtime stdlib registry, its leading entry prelude accepts only
these normalized import forms:

```ash
use result::Result
use runtime::RuntimeError
use runtime::Args
use time::{sleep}
```

The `entry.rs` tests validate the first three forms. `time::{sleep}` is also a registered form.
This prelude is semicolon-free loader/entry convenience, not whole-file `module_file` grammar and
not a general runtime import mechanism.

### Exact selected runtime witness

The following source shape is admitted only through the sealed `time::sleep` production route,
with the Engine-owned profile and provider binding installed by its test. It is the sole positive
standard-library execution witness claimed by this page.

```ash
fn main() -> Null { time::sleep(0) }
```

It does not establish `time::now`, `time::epoch_millis`, arbitrary `sleep` programs, automatic
imports, or a generic standard-library runtime.

## Syntax

The normal direct import grammar is shared with the modules chapter. The standard library does not
add a distinct source import form.

```ebnf
stdlib_import = [ visibility ] "use" stdlib_use_path [ "as" path_segment ] ";" ;
stdlib_use_path = simple_path | simple_path "::" "*" | simple_path "::" "{" [ use_item { "," use_item } [ "," ] ] "}" ;
simple_path = path_segment { "::" path_segment } ;
use_item = path_segment [ "as" path_segment ] ;
```

Here `visibility`, `path_segment`, and the Engine-only semicolon-free prelude are defined by
[Module grammar](../lexical-and-modules/modules-imports-and-visibility.md#syntax). The EBNF only
states direct-parser acceptance; it makes no module, export, or runtime availability claim.

## How imports work

No source-level sequent is warranted. The implementation supplies import resolution and bounded
Engine admission procedures, not a source-level standard-library reduction calculus. The relevant
current rules are operational boundaries:

1. For non-local imports, ordinary resolution searches the importing directory,
   override/environment dependency roots, built-in `std/src`, and then discovered locked-project
   vendor/cache roots before it collects public exports into a checked module-loading result.
2. The runtime-entry path accepts only its small registered import set and masks that prelude
   before parsing the entry body.
3. The Engine admission boundary independently decides whether a checked source has a sealed
   production lowering. Import success cannot satisfy that decision.

## Diagnostics and limitations

- A missing ordinary module, private export, cycle, or unavailable locked dependency fails module
  loading; importing a name is not equivalent to evaluating it.
- `json` is a concrete counterexample to the common overclaim: selected calls parse and check,
  but the Engine test requires closed admission.
- The runtime registry's exact sources include `result`, `runtime`, `runtime::error`,
  `runtime::args`, and `time`; its accepted source import forms are still only `result::Result`,
  `runtime::RuntimeError`, `runtime::Args`, and `time::{sleep}`.
- `std/src/prelude.ash` is not automatically injected into source.
- Historical Act/Proc/Workflow library descriptions are excluded. They do not name current
  importable or executable source APIs.

## Related evidence

- `cargo test -p ash-cli --test stdlib_corpus_check`
- `cargo test -p ash-parser --test stdlib_parsing`
- `cargo test -p ash-engine --test task_968_installed_stdlib`
- `cargo test -p ash-engine --test json_stdlib_e2e`
- `crates/ash-engine/src/{module_loader.rs,entry.rs,production_cps_driver.rs}`
- [Diagnostics and errors](diagnostics-and-errors.md)
