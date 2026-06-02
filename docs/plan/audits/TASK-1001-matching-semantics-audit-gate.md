# TASK-1001: Matching semantics audit gate

## Status

Complete for the TASK-1001 docs gate. No Rust matching semantics were changed.

## Scope

This audit freezes live pattern-use callsites for PLAN-126/SPEC-076 before TASK-1002+ implementation. Pattern uses are classified as:

- irrefutable binder
- exhaustive eliminator
- explicit complement eliminator
- explicit refutable filtering construct

## Callsite Matrix

| Area | Live callsite | Current classification | Current behavior |
|------|---------------|------------------------|------------------|
| Parser pattern entrypoint | `crates/ash-parser/src/parse_pattern.rs:26` `pattern` | shared syntax substrate | Parses variable, wildcard, tuple, record, list, literal, and variant patterns. It does not classify totality. |
| Surface function/body block let | `crates/ash-parser/src/parse_expr.rs:724`, `crates/ash-parser/src/parse_module.rs:3098` | irrefutable binder | Parses `let <pattern> = <expr>` block statements. Typecheck currently binds variables structurally without irrefutability. |
| Surface named local fn desugar | `crates/ash-parser/src/parse_expr.rs:746`, `crates/ash-parser/src/parse_module.rs:3123` | irrefutable binder | Desugars local `fn name(...)` to a variable-pattern block let. |
| Surface match arms | `crates/ash-parser/src/parse_module.rs:3382` | exhaustive eliminator | Parses `match scrutinee { pattern => expr }` inside function-body parser. |
| Surface `with_error` arms | `crates/ash-parser/src/parse_expr.rs:372` | exhaustive eliminator | Parses `with_error { body } handle { pattern => expr }`; total handler coverage is not currently enforced. |
| Surface if-let | `crates/ash-parser/src/parse_expr.rs:840` | explicit complement eliminator | Parses `if let pattern = expr then expr else expr`; `else` is mandatory in this parser. |
| Standalone observe expression parser | `crates/ash-parser/src/parse_observe.rs:93` | irrefutable binder parser substrate | Parses standalone `observe <capability>:<channel> ... as <pattern>` into `ObserveExpr { pattern }`; it does not classify totality. |
| Workflow `let` | `crates/ash-parser/src/parse_workflow.rs:1418` | irrefutable binder | Parses workflow `let <pattern> = <expr>`, except simple-name action sugar may lower to `Act { result_name }`. |
| Workflow observe `as` | `crates/ash-parser/src/parse_workflow.rs:1197` | irrefutable binder | Parses optional `observe <capability> as <pattern>`. Lowering preserves this as core observe pattern or wildcard. |
| Workflow orient `as` | `crates/ash-parser/src/parse_workflow.rs:1257` | irrefutable binder | Parses optional `orient <expr> as <pattern>`. Typecheck binds it, but lowering currently ignores the binding. |
| Workflow propose `as` | `crates/ash-parser/src/parse_workflow.rs:1281` | irrefutable binder | Parses optional `propose <action> as <pattern>`. Typecheck currently rejects it as unsupported; lowering ignores it. |
| Workflow for | `crates/ash-parser/src/parse_workflow.rs:1515` | irrefutable binder | Parses `for <pattern> in <expr> do <workflow>`. Runtime fails with workflow pattern error if an item does not match. |
| Workflow receive stream binding | `crates/ash-parser/src/parse_receive.rs:95`, `:162` | explicit refutable filtering construct | Parses receive arms and `capability:channel as <pattern>` stream patterns with optional guards. |
| Workflow yield arms | `crates/ash-parser/src/parse_module.rs:2686`, `:2730` | lowered irrefutable binder today; should be audited by TASK-1004 | Surface yield arms parse as patterns. Lowering converts arms to core `Workflow::Let` over the resume variable, so failed arms are not selective today. |
| Lower observe | `crates/ash-parser/src/lower.rs:814` | irrefutable binder | Lowers absent observe binding to wildcard and present binding via `lower_pattern`. |
| Lower workflow let/for | `crates/ash-parser/src/lower.rs:1083`, `:1122` | irrefutable binder | Lowers source patterns directly to core `Workflow::Let` / `Workflow::ForEach`. |
| Lower receive | `crates/ash-parser/src/lower.rs:1069`, `:2206` | explicit refutable filtering construct | Preserves receive arm order, guards, wildcard/literal/stream pattern forms. |
| Lower yield arms | `crates/ash-parser/src/lower.rs:1732` | irrefutable binder | Converts single and multi-arm yield to the first arm pattern as core `Workflow::Let`; recursive fallback is computed but not used. |
| Lower if-let | `crates/ash-parser/src/lower.rs:1910` | explicit complement eliminator | Lowers surface `Expr::IfLet` to core `Expr::IfLet`. |
| Lower match / with_error arms | `crates/ash-parser/src/lower.rs:1884`, `:1995` | exhaustive eliminator | Lowers each surface arm pattern through `lower_pattern`. |
| Core expression let | `crates/ash-core/src/ast.rs:542` | irrefutable binder | Core-only expression binder used by lowered function/block forms and runtime helpers. |
| Core match arms | `crates/ash-core/src/ast.rs:461`, `:588` | exhaustive eliminator | Core `Expr::Match` carries arm patterns. |
| Core if-let | `crates/ash-core/src/ast.rs:467` | explicit complement eliminator | Core `Expr::IfLet` carries pattern, scrutinee, then, else. |
| Core with_error arms | `crates/ash-core/src/ast.rs:517` | exhaustive eliminator | Reuses `MatchArm` for operational failure handlers. |
| Core workflow observe/let/foreach | `crates/ash-core/src/ast.rs:18`, `:72`, `:89` | irrefutable binder | Runtime pattern failure currently remains possible. |
| Core workflow spawn/split | `crates/ash-core/src/ast.rs:122`, `:130` | core-only irrefutable binder | No live surface parser found for core `Workflow::Spawn`/`Workflow::Split`; runtime executes and binds patterns defensively. |
| Core receive | `crates/ash-core/src/ast.rs:302` | explicit refutable filtering construct | Ordered receive arms with stream, literal, and wildcard receive patterns. |
| Legacy stream receive | `ash_core::stream::ReceiveArm { pattern: Pattern }`, `crates/ash-core/src/stream.rs:134`, `:175` | explicit refutable filtering construct | Legacy stream `Receive` arms carry `Pattern` and optional guards. They are selective receive arms, not irrefutable binders. |
| Typeck `check_pattern` | `crates/ash-typeck/src/check_pattern.rs:147` | shared pattern typing substrate | Validates pattern/type compatibility and returns bindings, but does not decide irrefutability. |
| Typeck exhaustiveness helper | `crates/ash-typeck/src/exhaustiveness.rs:60`, `:99` | central pattern-to-coverage helper for match exhaustiveness | Converts AST patterns to coverage cells and checks constructor universes for ordinary `match`; it is the shared coverage substrate TASK-1005 must harden. |
| Typeck match | `crates/ash-typeck/src/check_expr.rs:3491` | exhaustive eliminator | Runs canonical exhaustiveness where available and propagates arm pattern type errors as `UnsupportedExpression`. |
| Typeck with_error | `crates/ash-typeck/src/check_expr.rs:3291` | exhaustive eliminator | Checks handler pattern compatibility against a fresh payload type and branch type compatibility, but has no payload-universe exhaustiveness check yet. |
| Typeck if-let | `crates/ash-typeck/src/check_expr.rs:154` | explicit complement eliminator | **silent pattern gap:** `check_pattern` errors are ignored; only successful bindings extend the then environment. |
| Typeck block let | `crates/ash-typeck/src/check_expr.rs:567` | irrefutable binder | Calls `bind_pattern_variables`, not `check_pattern`; pattern type errors and refutability are not enforced here. |
| Typeck workflow binders | `crates/ash-typeck/src/lib.rs:956`, `:1190` | irrefutable binder | Observe/orient/let/for/receive/yield infer or validate by structural variable binding, not irrefutability. Propose binding is rejected in validation. |
| Name resolver duplicate binders | `crates/ash-typeck/src/names.rs:831` | binder validation substrate | Detects duplicate binders inside patterns for most bind positions. Expression `IfLet` currently does not bind its pattern during name resolution. |
| Interpreter pattern engine | `crates/ash-interp/src/pattern.rs:32` | runtime matching substrate | Performs dynamic matching and returns `PatternError` on non-match. |
| Interpreter expression let | `crates/ash-interp/src/eval.rs:2026`, `:2998` | irrefutable binder defensive runtime path | Failed match becomes `EvalError::LetPatternBindFailed`. |
| Interpreter match | `crates/ash-interp/src/eval.rs:2143`, `:3192` | exhaustive eliminator defensive runtime path | First matching arm wins; no arm becomes `EvalError::NonExhaustiveMatch`. |
| Interpreter if-let | `crates/ash-interp/src/eval.rs:2196`, `:3222` | explicit complement eliminator | Failed match evaluates else under original context. |
| Interpreter with_error | `crates/ash-interp/src/eval.rs:2167`, `:2907` | exhaustive eliminator defensive runtime path | First handler pattern matching operational payload wins; no handler re-raises original failure. |
| Interpreter standalone observe | `crates/ash-interp/src/execute_observe.rs:106` | irrefutable binder defensive runtime path | Matches the standalone observe result against `observe.pattern`; failed match becomes `ExecError::PatternMatchFailed`. |
| Interpreter workflow observe/let/for/spawn/split | `crates/ash-interp/src/execute.rs:610`, `:757`, `:1090`, `:1501`, `:1541` | irrefutable binder defensive runtime path | Failed match becomes `ExecError::PatternMatchFailed`; `:757` is the `Workflow::Observe` binding path. |
| Interpreter small-step workflow binders | `crates/ash-interp/src/small_step.rs:74`, `:216`, `:376`, `:526`, `:571`, `:587` | defensive runtime substrate | Small-step `ForEach`, `Let`, `Observe`, `Spawn`, and `Split` paths dynamically call `match_pattern` and map failures to `ExecError::PatternMatchFailed`; these remain host/unchecked-IR defensive paths. |
| Interpreter receive | `crates/ash-interp/src/execute_stream.rs:51`, `:262` | explicit refutable filtering construct | Ordered arms, guard fallthrough, wildcard fallback, and no-match `Value::Null` are current behavior. |
| Engine check | `crates/ash-engine/src/lib.rs:1291` | diagnostic transport | Runs `ash-typeck` and wraps errors as `EngineError::Type`; it does not reinterpret pattern semantics. |
| CLI check | `crates/ash-cli/src/commands/check.rs:61` | diagnostic transport | Runs engine parse/check and renders parse/type errors; no independent pattern semantics. |
| LSP diagnostics | `crates/ash-lsp-core/src/diagnostics.rs:92` | parse/lint diagnostic transport | Currently parses and lints; type checking is TODO, so pattern semantic diagnostics do not surface through LSP yet. |

## If-Let Parser Entrypoints

`if-let parser entrypoints` are split today:

- General expression entrypoint `crates/ash-parser/src/parse_expr.rs:356` tries `parse_if_let_expr` before ordinary pipe expressions, so raw expression tests accept `if let Some { value: x } = opt then { x } else { 0 }`, unit variants, variables, wildcard, tuple patterns, and complex scrutinees.
- `parse_if_let_expr` requires the live syntax `if let <pattern> = <expr> then <expr-or-block> else <expr-or-block>`. There is no accepted `if let` without `else` in this parser because `keyword("else")` is mandatory.
- Real module/function-body parsing uses `parse_fn_expr` in `crates/ash-parser/src/parse_module.rs:3203`. That function dispatches any input beginning with `if` to `parse_fn_if_expr` before the general expression parser. `parse_fn_if_expr` expects a boolean condition followed by `then`, so `if let ...` in that module/function-body context is currently blocked by the `if` parser rather than accepted as `Expr::IfLet`.
- Anonymous function bodies parsed through `crates/ash-parser/src/parse_expr.rs:704` use the general `expr` entrypoint for the tail expression, so their tail context can reach `parse_if_let_expr`.

TASK-1007 must add parser-entrypoint RED tests for both the raw expression parser and real module/function-body contexts, including the currently unsupported function-body `if let` path.

## Current Typechecking Notes

- Current if-let typechecking has a **silent pattern** failure path. `crates/ash-typeck/src/check_expr.rs:168` calls `check_pattern`, but the `Err` branch is absent; invalid or impossible patterns are silently ignored and the then branch is checked without those bindings.
- The helper `infer_surface_expr_type` reports branch type mismatch as `if-let branches must have compatible types: <then> vs <else>`; `check_expr` uses `merge_branch_results`, so TASK-1007 should pin both diagnostic routes as needed.
- `Expr::IfLet` name resolution in `crates/ash-typeck/src/names.rs:631` does not bind the pattern in the then branch and does not reject duplicate binders for the if-let pattern. That must be covered by scope/shadowing tests.
- Block-level `let` in `check_expr` and workflow binder inference use `bind_pattern_variables`; this structurally inserts names and fresh fallback types instead of checking shape, impossibility, or irrefutability.
- `match` is the strongest current path: it canonicalizes scrutinee types where possible, accepts universal wildcard/variable coverage, reports `ConstructorError::NonExhaustiveMatch`, and reports pattern type errors as `UnsupportedExpression`.
- `with_error` checks handler pattern compatibility against a fresh failure payload type but has no total handler coverage or closed payload universe check.

## Workflow Binder Split

### surface-level

- Workflow `let <pattern> = <expr>` is source-level and lowers to core `Workflow::Let`.
- `observe <capability> as <pattern>` is source-level and lowers to core `Workflow::Observe { pattern }`; absent binding lowers to wildcard.
- `orient <expr> as <pattern>` is source-level and typecheck/name resolution bind it, but lowering ignores the binding.
- `propose <action> as <pattern>` is source-level parseable, but current typecheck rejects it as unsupported and lowering ignores it.
- `for <pattern> in <expr> do ...` is source-level and lowers to core `Workflow::ForEach`.
- Receive stream `capability:channel as <pattern>` is source-level but classified as an explicit refutable filtering construct, not an irrefutable binder.
- yield arms are source-level patterns, but current lowering treats them as binders over the resume value rather than selective arms.

### lowered-only

- Surface function/block `BlockStmt::Let` lowers to core expression `Expr::Let`.
- Yield arms lower to core `Workflow::Let` over `resume_var`; only the first arm is used in the multi-arm path today.
- `let name = provider:action(...)` workflow sugar may become `Workflow::Act { result_name: Some(name) }`, which is name-only and not a pattern binder after lowering.

### core-only

- `Workflow::Spawn { pattern, ... }` and `Workflow::Split { pattern, ... }` are core-only pattern binders in the audited surface parser set. They execute in `ash-interp` and must be covered by TASK-1004/TASK-1008 defensive runtime tests.
- Core `Expr::Let` may be host/lowering-created and must not bypass TASK-1003 irrefutability checks.

## Runtime Error Variants

Exact live diagnostics and runtime failure variants:

- Expression let bind failure: `EvalError::LetPatternBindFailed { pattern: String, value: String }`, display text `pattern bind failed in let-expression: pattern {pattern} does not match value {value}`.
- Expression/core match fallback: `EvalError::NonExhaustiveMatch { value: String }`, display text `non-exhaustive match: no arm matched value {value}`.
- Workflow binder failure: `ExecError::PatternMatchFailed { pattern: String, value: Box<Value> }`, display text `pattern match failed: {pattern} does not match {value}`.
- Dynamic pattern engine errors are `PatternError::MatchFailed`, `PatternError::ListLengthMismatch`, `PatternError::FieldMissing`, and `PatternError::NotARecord`.

These runtime errors remain defensive for unchecked IR/host values. TASK-1003/TASK-1004/TASK-1008 must prove checked source no longer normally reaches them for binder cases owned by typeck.

## Selective Receive Behavior

Current selective receive is an explicit refutable filtering construct:

- Arms are tried in source/lowered order.
- Stream and control messages are matched by receive pattern first.
- Guards run after pattern matching with pattern bindings in guard context; a false guard falls through to later arms.
- Matching entries are removed from the mailbox before executing the selected arm body.
- Non-blocking receive with no matching entry returns `Value::Null`, unless a wildcard arm is selected as fallback.
- Timed receive returns wildcard fallback on timeout if present, otherwise `Value::Null`.
- Blocking receive waits and retries.
- Capability/policy availability is enforced before wildcard fallback for denied/missing stream sources.

TASK-1007 must preserve guard/order/no-match behavior and prevent shared irrefutability checks from reclassifying receive arms as total protocol receive.

## RED-Test Map

| Owner | Required RED tests |
|-------|--------------------|
| TASK-1002 | `irrefutable_variable_and_wildcard_accept_open_non_adt_scrutinees`; `irrefutable_single_variant_adt_accepts_nested_irrefutable_fields`; `irrefutable_nested_refutable_binder_reports_missing_witness`; `irrefutable_list_pattern_without_rest_is_refutable`; `irrefutable_literal_pattern_is_refutable_without_singleton`; `irrefutable_duplicate_binders_rejected`; `irrefutable_impossible_pattern_reports_type_mismatch`; `irrefutable_blocked_constructor_coverage_reports_blocked_reason`. |
| TASK-1003 | `pure_block_let_rejects_some_over_option`; `pure_block_let_rejects_nested_refutable_binders`; `pure_block_let_rejects_list_patterns`; `core_expr_let_rejects_refutable_host_ir`; `pure_block_let_accepts_variable_wildcard_and_single_variant`; `pure_block_let_duplicate_binders_rejected`. |
| TASK-1004 | `workflow_let_rejects_refutable_sum_literal_and_list_patterns`; `observe_binding_rejects_refutable_pattern`; `orient_binding_either_rejects_or_documents_lowering_defer`; `for_binder_rejects_refutable_item_pattern`; `yield_arms_reject_or_document_current_lowered_binder_semantics`; `core_spawn_pattern_rejects_refutable_instance_pattern`; `core_split_pattern_rejects_refutable_tuple_pattern`; `receive_stream_pattern_remains_selective_not_irrefutable`. |
| TASK-1005 | `match_wildcard_default_accepts_open_non_adt_scrutinee`; `match_missing_adt_constructor_reports_NonExhaustiveMatch_diagnostic`; `match_blocked_constructor_coverage_reports_blocked_reason`; `match_impossible_pattern_reports_type_error`; `match_nested_product_coverage_does_not_overgeneralize`; `match_list_patterns_have_conservative_diagnostics`. |
| TASK-1006 | `with_error_total_handler_reports_or_defers_closed_payload_missing_case`; `with_error_handler_pattern_type_error_is_structured`; `with_error_wildcard_accepts_open_payload`; `with_error_branch_type_mismatch_reports_handler_context`. |
| TASK-1007 | `if_let_parser_entrypoints_accept_raw_expression_and_real_function_context_or_pin_rejection`; `if_let_without_else_rejected`; `if_let_check_pattern_errors_are_propagated_not_silent`; `if_let_then_binding_scope_does_not_escape`; `if_let_shadowing_then_uses_inner_else_uses_outer`; `if_let_branch_type_mismatch_is_reported`; `if_let_duplicate_binders_rejected`; `if_let_irrefutable_pattern_emits_unreachable_else_warning`; `if_let_impossible_pattern_is_hard_error`; `selective_receive_guard_order_no_match_behavior_preserved`. |
| TASK-1008 | `runtime_defensive_expr_let_still_yields_LetPatternBindFailed_for_unchecked_ir`; `runtime_defensive_workflow_binder_still_yields_PatternMatchFailed_for_unchecked_ir`; `runtime_defensive_match_still_yields_NonExhaustiveMatch_for_unchecked_ir`; `checked_source_refutable_binders_fail_in_typeck_not_runtime`; `cli_and_lsp_surface_matching_diagnostics_from_typeck_when_available`. |

## Downstream Focused Commands

TASK-1002 through TASK-1008 verification blocks now use exact concrete future test commands. The commands intentionally name owning crates, integration-test targets, and exact RED-test map names so they are non-zero and focused once the owning tasks create the tests.
