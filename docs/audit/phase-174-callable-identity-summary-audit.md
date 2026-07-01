# Phase 174 Callable Identity Summary Audit

## Scope

This audit satisfies TASK-1779 and constrains TASK-1780. It defines which ordinary call expressions inside macro templates may participate in bounded macro type inference without treating macros as runtime callables or guessing through ambiguous names.

## Prior boundary

TASK-1772 kept ordinary calls uninferred. A template such as `pub macro inc(x: Int) => add(x, 1);` preserved the annotated parameter type but did not infer a result type because `add` could be unresolved, imported, overloaded, private, or otherwise not uniquely identified at syntax-summary time.

## Callable identity proof table

| Category | Available evidence now | Safe for TASK-1780? | Decision |
|---|---|---|---|
| Local public `fn` with complete parameter and return annotations | Same parsed definition list contains one public `Definition::Function` with matching name, arity, parameter types, and return type | Yes | Infer result only when argument types match exactly. |
| Local public `builtin fn` with complete parameter and return annotations | Same parsed definition list contains one public `Definition::BuiltinFn` with matching name, arity, parameter types, and return type | Yes | Infer result only when argument types match exactly. |
| Imported public function | Module-loader callable summaries exist in engine-oriented paths, not in parser-only macro summary collection | No | Defer until parser/LSP summary code can consume imported callable type summaries explicitly. |
| Overloaded/interface method | Multiple candidates or dictionary/interface dispatch may share a name | No | Remain annotation-required; no solving or dispatch selection in macro inference. |
| Module-qualified path | `Expr::Call` carries `module: Some(_)`, but parser-only inference has no imported source-location/type-summary proof here | No | Remain uninferred. |
| `MacroSummary` / imported macro summary | Syntax-phase metadata; explicitly not callable, no rows/authority/contracts/providers | No | Never a callable identity proof. |
| Private helper callable crossing module boundary | May be source-local but must not become an exported callable identity | No | Public macro summaries do not infer through private helper callables. |
| Unresolved name | No callable summary | No | Remain uninferred. |
| Wrong arity or argument type mismatch | Candidate exists but invocation shape does not match | No | Remain uninferred. |

## Implementation hook allowed for TASK-1780

TASK-1780 may use only a lightweight local `CallableTypeSummary` built from public callables in the same parsed definition list as the macro declaration. The summary may contain:

- callable name;
- parameter type list;
- return type;
- an ambiguity flag when another callable with the same name appears.

It must not contain or imply:

- macro summary metadata as a callable target;
- runtime authority, rows, contracts, providers, failures, or proof evidence;
- imported function identity unless a later phase threads real imported callable summaries into the parser/LSP macro-inference path.

## Required TASK-1780 fixtures

Positive:

- `fn add(a: Int, b: Int) -> Int { ... }` plus `pub macro inc(x: Int) => add(x, 1);` infers `Int`.

Negative:

- unresolved `add(x, 1)` remains uninferred;
- duplicate local `add` definitions make identity ambiguous and remain uninferred;
- wrong arity and argument-type mismatch remain uninferred;
- private helper callables remain uninferred for public macro summaries;
- module-qualified `math::add(x, 1)` remains uninferred;
- unannotated macro parameter identity remains summary-free;
- macro summaries are never used as callable proofs.

## Verification evidence

- `cargo test -p ash-parser --test task_1772_macro_type_inference -- --nocapture`: 9 tests passed.
- `cargo check -p ash-parser -p ash-lsp-core`: passed.
