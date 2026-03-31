# TASK-S57-4 Stdlib Import Rules Design

## Context

TASK-S57-4 resolves an inconsistency between legacy task prose that used dot-style standard-library imports such as `use result.{Result, Ok, Err}` and the current specifications, which already define module and import paths with `::` separators.

## Decision

- `::` remains the only normative module and import path separator.
- The standard library is a compiler-provided root namespace; modules are referenced as top-level names such as `result`, `option`, `runtime`, and `io`, without a required `std::` prefix.
- The prelude is defined by `std/src/prelude.ash`.
- Only prelude-exported names are implicitly available in every module; other standard-library modules require explicit `use` declarations.

## Rationale

This keeps module paths consistent across SPEC-009 and SPEC-012, avoids introducing a second namespace separator, and aligns with newer task examples that already use `use result::Result` and `use runtime::Args`.

## Scope

This design updates:

- `docs/spec/SPEC-009-MODULES.md`
- `docs/spec/SPEC-012-IMPORTS.md`
- task tracking and changelog metadata

It does not add new runtime modules or surface syntax beyond clarifying the existing import contract.
