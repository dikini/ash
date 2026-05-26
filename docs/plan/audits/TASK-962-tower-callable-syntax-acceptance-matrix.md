# TASK-962: SPEC-072 / PLAN-121 acceptance matrix

## Status

TASK-962 closeout evidence for SPEC-072 C72-1 through C72-8 and PLAN-121 §8.

## Acceptance matrix

| Criterion | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| C72-1 | Preferred pure callable type syntax such as `(Int, Int) -> Int` is accepted by parser/typechecker. | `crates/ash-parser/tests/task_957_callable_type_parser.rs`; `crates/ash-typeck/tests/task_958_callable_type_rendering.rs`; focused gates `cargo test -p ash-parser --test task_957_callable_type_parser -- --nocapture` and `cargo test -p ash-typeck --test task_958_callable_type_rendering -- --nocapture`. | Satisfied |
| C72-2 | Legacy `Fn(Int, Int) -> Int` remains accepted during compatibility and preferred rendering uses `(Int, Int) -> Int`. | `crates/ash-parser/tests/task_957_callable_type_parser.rs`; `crates/ash-typeck/tests/task_958_callable_type_rendering.rs`; `crates/ash-engine/tests/task_958_callable_module_summary.rs`; TASK-958 rendering/import-summary gates. | Satisfied |
| C72-3 | `(Int, Int) -> Bool` is n-ary, not a unary tuple argument; partial application is not accepted for callable application. | `crates/ash-parser/tests/task_957_callable_type_parser.rs`; `crates/ash-typeck/tests/task_958_callable_type_rendering.rs` tests `callable_application_requires_exact_arity`, `too_few_arguments_are_not_partial_application`, and `too_many_arguments_report_exact_arity`. | Satisfied |
| C72-4 | Pure closures use `|args| -> body`; old `|args| => body` is not silently accepted as pure syntax. | `crates/ash-parser/tests/task_959_pure_closure_arrow.rs`; `crates/ash-typeck/tests/task_959_pure_closure_arrow.rs`; `crates/ash-interp/tests/task_959_pure_closure_arrow.rs`. | Satisfied |
| C72-5 | `-*>`, `=>`, and `=*>` in callable-type position are reserved and fail closed with targeted diagnostics. | `crates/ash-parser/tests/task_960_reserved_callable_arrows.rs`; `crates/ash-typeck/tests/task_960_reserved_callable_arrows.rs`. Regression coverage includes comment-separated contexts and string/comment false-positive avoidance. | Satisfied |
| C72-6 | `|args| -*>`, `|args| =>`, and `|args| =*>` are reserved and fail closed with targeted diagnostics. | `crates/ash-parser/tests/task_960_reserved_callable_arrows.rs`; `crates/ash-typeck/tests/task_960_reserved_callable_arrows.rs`; TASK-959 match-arm `=>` noninterference coverage. | Satisfied |
| C72-7 | Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables and are distinct from reserved tower-callable arrows. | `crates/ash-typeck/tests/task_960_reserved_callable_arrows.rs`; `reference/language/functions.md`; `reference/language/functions/calls-and-values.md`; `reference/agents/cards/functions.md`. | Satisfied |
| C72-8 | SPEC-027/SPEC-031/reference/agent-card/generated rendering no longer teach stale `Fn(...) -> ...` or pure `|x| => ...` as preferred syntax. | `docs/spec/SPEC-027-PURE-FUNCTIONS.md`; `docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md`; `reference/language/functions.md`; `reference/language/functions/*.md`; `reference/agents/cards/functions.md`; `crates/ash-parser/tests/task_963_stdlib_reference_callable_syntax.rs`; `python3 tools/reference/check_frontmatter.py`; `python3 tools/reference/check_frontmatter.py --pilot`. | Satisfied |

## PLAN-121 closeout expectations

| Expectation | Evidence | Status |
| --- | --- | --- |
| PLAN-INDEX, PLAN-121, task files, spec index, legacy amended specs, `std/`, `reference/`, and CHANGELOG agree. | TASK-962 status reconciliation updates these surfaces; TASK-963 scan proves current `std/` and top-level `reference/` daily-use examples avoid unlabelled legacy callable syntax. | Satisfied by closeout patch |
| Focused tests prove n-ary callable domain vs tuple argument distinction. | TASK-957 parser tests and TASK-958 exact-arity typechecker tests. | Satisfied |
| Focused tests prove old pure closure `|args| => body` is no longer silently accepted. | TASK-959 parser/typeck/interpreter tests. | Satisfied |
| Reserved higher-stratum arrows fail closed with targeted diagnostics. | TASK-960 parser/typeck tests. | Satisfied |
| Current `std/` and `reference/` examples use preferred syntax except explicitly labeled compatibility material. | TASK-963 std/reference scan test and focused Python scan in the TASK-963 verification block. | Satisfied |
| Independent review checks parser ambiguity, stale docs, stdlib/reference migration coverage, and callable-stratum/return-type conflation. | Independent reviews requested changes for duplicate changelog entries, stale review-status checkboxes, residual runtime partial application, unary reserved-arrow diagnostics, task-range metadata drift, and final evidence wording; TASK-962 remediated those findings before commit. | Satisfied |
| Broad gates run. | After the final TASK-962 remediation diff, `git diff --check`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `bash scripts/check-rust-tests.sh --workspace --all-targets`, `cargo doc --workspace --no-deps` with warning scan, `python3 tools/reference/check_frontmatter.py`, and `python3 tools/reference/check_frontmatter.py --pilot` all exited 0. The broad gate also exposed and then verified stale pipe-operator, stdlib-surface, capability-implementation, and runtime action-control assertions. | Satisfied |

## Broad-gate note

An initial combined broad gate hit `No space left on device` while writing Cargo incremental artifacts because this worktree's `target/` had grown to 73 GiB and the root filesystem had 0 available bytes. `cargo clean` removed 75.5 GiB from this worktree only; the broad gate was then restarted with `CARGO_INCREMENTAL=0`.

The restarted broad `cargo test --workspace` first reached `crates/ash-interp/tests/pipe_operator_e2e.rs` and failed because the test still used old implicit partial-application examples (`filter(ends_with(".md"))`, `filter(starts_with("src/"))`). SPEC-072/TASK-958 now require exact callable arity, so TASK-962 remediated the tests to use explicit pure closures (`filter(|file| -> ends_with(file, ".md"))`, `filter(|path| -> starts_with(path, "src/"))`) and pure `Type::Fn` predicate fixtures.

A later broad rerun reached `crates/ash-parser/tests/stdlib_surface.rs` and failed because the stdlib surface assertion still expected the pre-TASK-963 bare unary callback spelling `f: A -> Proc<B>`. TASK-963 migrated `std/src/proc.ash` to `f: (A) -> Proc<B>`, so TASK-962 updated that stale broad assertion rather than reverting the stdlib migration.

A subsequent broad rerun reached `crates/ash-typeck/tests/task_730_capability_implementation_conformance.rs` and failed because `operation_body_is_checked_in_effectful_context` still expected a pure `fn` expression in an effectful capability operation body to become a legacy `Type::Fun(..., Effect)`. TASK-959/SPEC-072 intentionally keeps pure closures as `Type::Fn` in higher contexts, so TASK-962 renamed the test and now asserts successful registration for the pure callable result.

A final pre-merge verification review found residual runtime partial-application behavior, missing targeted diagnostics for unary reserved tower callable arrows, SPEC-072 task-range metadata drift, duplicate TASK-963 changelog wording, and a `main...HEAD` whitespace issue in the TASK-956 audit artifact. The final remediation removes the runtime partial-application paths, adds unary reserved-arrow coverage for bare and generic domains, reconciles the metadata/changelog, and re-runs the focused and broad gates before merge.

## Final verification

Final commands on the closeout diff:

```bash
git diff --check
python3 tools/reference/check_frontmatter.py
python3 tools/reference/check_frontmatter.py --pilot
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/check-rust-tests.sh --workspace --all-targets
cargo doc --workspace --no-deps 2>&1 | tee /tmp/phase126-cargo-doc.log
! grep -i '^warning:' /tmp/phase126-cargo-doc.log
```

All final commands exited 0 after remediating stale broad-gate tests. A raw foreground `cargo test --workspace` run reached the tool timeout before completion, so the final broad test evidence uses the repo-owned serial all-target wrapper above.
