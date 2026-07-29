# Library and Diagnostics

[Language reference](../index.md) · [Modules and imports](modules-and-imports.md) ·
[Diagnostics and errors](diagnostics-and-errors.md) · [Source of truth](../source-of-truth.md)

## Status and evidence

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `std/src/**` source corpus | accepted | checked | bounded-only | closed | partial | tested | below_spec |
| Ordinary stdlib module imports | parser-only | partial | bounded-only | closed | partial | tested | below_spec |
| Runtime-entry stdlib registry | parser-only | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Selected `time::sleep` route | accepted | checked | lowered | admitted-executed | partial | tested | below_spec |
| Parse, static, admission, and terminal observations | not-applicable | partial | bounded-only | fixture-bounded | partial | tested | below_spec |

The corpus and import rows do not establish execution. The one admitted standard-library witness
is the Engine-sealed `time::sleep` route described in [Modules and imports](modules-and-imports.md).
The terminal row is the normalized Engine/client observation boundary, not a promise that every
failure has a stable public diagnostic shape.

Primary evidence is `std/src/**`, `crates/ash-engine/src/{module_loader.rs,entry.rs,lib.rs}`, and
`crates/ash-engine/src/error.rs`. The focused tests are
`crates/ash-cli/tests/stdlib_corpus_check.rs`,
`crates/ash-parser/tests/stdlib_parsing.rs`,
`crates/ash-engine/tests/{task_968_installed_stdlib.rs,json_stdlib_e2e.rs}`, and the CLI
diagnostic/terminal tests linked by the detail pages.

## What this chapter covers

Ash's repository contains a source standard library under `std/src`. Its public declarations can
be parsed, checked, and—in selected ordinary-loader paths—resolved as imports. That source
inventory is not a runtime catalogue. Each callable needs evidence through the Engine admission
boundary before it may be described as executable.

The second page maps the observable diagnostic layers: parsing, static checking, admission, and
normalized terminal output. It records where those layers deliberately stop rather than inventing
a single error model for all paths.

## Scope boundary

This chapter does not revive historical Act/Proc/Workflow library material, promise automatic
prelude imports, or expose Rust helpers as source APIs. Existing historical pages may remain as
links outside this manual, but they are not current-library authority.

## Related evidence

- [AUDIT-206 LANG-017 and LANG-018](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#current-feature-census)
- [TASK-2053](../../../plan/tasks/TASK-2053-language-reference-stdlib-diagnostics-limitations.md)
- [Module grammar and loader boundary](../lexical-and-modules/modules-imports-and-visibility.md)
- [Entry and admission boundary](../execution/entry-lowering-and-admission.md)
