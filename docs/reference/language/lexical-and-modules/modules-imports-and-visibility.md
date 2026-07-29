# Modules, Imports, and Visibility

[Lexical and modules index](index.md) · [Source files and literals](source-files-names-and-literals.md) ·
[Language reference](../index.md)

## Status and evidence

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `mod name;` and inline `mod name { ... }` parser surface | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |
| Direct `parse_use` statement parser | accepted | not-applicable | not-applicable | not-applicable | implemented | tested | not-applicable |
| Engine ordinary-module import prelude | parser-only | partial | bounded-only | not-applicable | partial | tested | below_spec |
| Engine runtime-entry `use` prelude | parser-only | partial | bounded-only | fixture-bounded | partial | tested | below_spec |

The two Engine prelude rows are `parser-only` because `module_file` does not accept `use` as an
item. Ordinary-module loading has no independent runtime behavior, while the runtime-entry row is
`fixture-bounded` only for its registered leading-import route. Neither row makes an imported
callable generally executable.

Source evidence is `crates/ash-parser/src/parse_module.rs::parse_module_decl` and
`::module_file`, `crates/ash-parser/src/parse_use.rs::parse_use`,
`crates/ash-parser/src/parse_visibility.rs::parse_visibility`, and
`crates/ash-engine/src/{module_loader.rs,entry.rs}`. Focused evidence is
`crates/ash-engine/tests/{module_file_check_tests.rs,module_import_resolution_tests.rs}` and
the visibility tests in `crates/ash-parser/src/parse_visibility.rs`.

This page owns AUDIT-206 LANG-001's module portion and LANG-002. It deliberately separates
parser acceptance from module loading and from executable programs.

## What it is and how to use it

`mod name;` declares a file-based module and `mod name { ... }` declares an inline module in the
surface parser. Both can be preceded by a visibility modifier. `pub`, `pub(crate)`, `pub(super)`,
`pub(self)`, and `pub(in path::to::module)` are the visibility forms accepted by
`parse_visibility`.

Inline modules are a parser feature, not an accepted general Engine module-file route. In
particular, `Engine::check_module_file` has a test that rejects an inline module containing an
ordinary type rather than silently omitting it. Use file-based modules for the checked module-file
route unless the specific downstream feature supplies narrower evidence.

`use` needs special care because it has more than one implementation route:

- `ash_parser::parse_use::parse_use` is a direct statement parser. It accepts optional visibility,
  simple paths, aliases, globs, and nested selections, and it **requires a terminating semicolon**.
- The Engine ordinary module loader scans a leading run of `use` or `pub use` lines, then adds a
  missing semicolon before it invokes the direct parser. This is loader convenience, not evidence
  that `module_file` accepts `use` as a top-level definition.
- The Engine runtime-entry prelude recognizes leading `use` lines and masks them before parsing
  the entry body. It accepts a semicolon-free line but only whitelists a small registered runtime
  import set. It is not a general import execution facility.

For all three routes, a resolved import contributes checked module summary information only within
its supported path. It does not prove that an imported callable can be admitted or executed.

## Examples

**Surface-parser module declaration.** This is parser evidence for `mod`; it is not a claim that
the inline child is accepted by `Engine::check_module_file`.

```ash
pub mod math;
mod local { fn value() -> Int { 1 } }
```

**Direct `parse_use` form.** The semicolon is required by
`crates/ash-parser/src/parse_use.rs::parse_use`.

```ash
pub use math::{Number as N, add};
```

**Engine-prelude convenience, not direct-parser syntax.** The following lacks a semicolon. The
ordinary module loader/runtime-entry prelude can recognize this leading form, but calling direct
`parse_use` on the same text fails because `parse_use` requires `;`. The runtime entry path also
requires the import to be one of its registered forms.

```ash
use time::{sleep}
fn main() { 0 }
```

**Resolved-import summary example.** The test
`plain_function_with_target_body_is_importable_by_signature` in
`module_import_resolution_tests.rs` shows a public function signature becoming an imported
callable summary. It does not execute that callable.

```ash
use dispatch::{complete_with_tools}
fn main() { 0 }
```

## Syntax

The first production is the whole-file `module_file` route. `direct_use` is intentionally
separate: it is the `parse_use` parser, not an item accepted by `module_file`.

```ebnf
module_declaration = [ visibility ] "mod" identifier ( ";" | "{" { definition } "}" ) ;
visibility = "pub" | "pub" "(" visibility_scope ")" ;
visibility_scope = "crate" | "super" | "self" | "in" visibility_path ;
visibility_path = path_segment { "::" path_segment } ;
direct_use = [ visibility ] "use" use_path [ "as" path_segment ] ";" ;
use_path = simple_path | simple_path "::" "*" | simple_path "::" "{" [ use_item { "," use_item } [ "," ] ] "}" ;
simple_path = path_segment { "::" path_segment } ;
use_item = path_segment [ "as" path_segment ] ;
```

`path_segment` and `import_text` are route-specific parser inputs. A path segment uses the
direct-path parser's ASCII alphanumeric-or-underscore rule, which differs from an ordinary source
identifier's first-character rule.

The two Engine prelude routes are intentionally separate:

```ebnf
ordinary_module_import_prelude = ( "use" | "pub" "use" ) import_text [ ";" ] ;
runtime_entry_import_prelude = "use" import_text [ ";" ] ;
```

The ordinary module loader accepts a leading run of the first form and normalizes a missing
semicolon before direct `parse_use`. The runtime-entry prelude accepts only the second, bare-`use`
form and then applies its registered-import whitelist. Neither production claims that arbitrary
`import_text` becomes a valid program import.

## Semantics and implementation boundary

No source-level sequent is supplied because the implementation exposes parser and module-summary
procedures rather than a checked formal module calculus. The relevant operational facts are:

1. `module_file` stores `mod` declarations separately from top-level definitions.
2. `parse_module_imports` scans only a leading import prelude and normalizes a missing semicolon
   before calling `parse_use`.
3. `mask_leading_entry_use_prelude` removes an accepted runtime-entry prelude before source-body
   parsing; `validate_runtime_entry_import_prelude` rejects unsupported registrations.

These are bounded loader/entry mechanisms, not authority, provider, or general execution rules.

## Diagnostics and boundaries

- `use` is not a `module_file` branch. A full file parsed directly through `parse_surface_file`
  cannot use direct-`use` acceptance as proof of whole-file grammar acceptance.
- **Important:** semicolon-free `use` is an Engine prelude convenience. Direct `parse_use` needs
  `;`.
- Inline modules parse, but the authoritative Engine check rejects the tested inline ordinary-type
  case. Do not infer general checked module support from parser acceptance.
- Import resolution fails closed for missing modules, cycles, unavailable locked dependencies, and
  visibility violations demonstrated by the module-loader tests.
- Visibility and imported summary availability do not grant runtime authority or prove a callable
  executes.
- Removed workflow module syntax is excluded.

## Related evidence

- [AUDIT-206 LANG-001 and LANG-002](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2052: entry and Engine admission](../../../plan/tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md)
- `cargo test -p ash-engine --test module_import_resolution_tests`
- `cargo test -p ash-engine --test module_file_check_tests`
