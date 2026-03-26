# Codex Comprehensive Review

Review date: 2026-03-26

Scope:
- Rust workspace under `crates/`
- Canonical specs `docs/spec/SPEC-001-IR.md` through `docs/spec/SPEC-022-WORKFLOW-TYPING.md`
- Project status artifacts in `README.md`, `docs/spec/README.md`, `docs/plan/PLAN-INDEX.md`, and prior audit notes

Verification run:
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`
- `cargo fmt --check`
- `cargo audit` (attempted, but blocked by read-only advisory DB path in this sandbox)

## Executive Summary

Overall health score: **58/100**

The codebase has broad test coverage and `cargo test --workspace` passes, but it is not in a release-ready state against its own project standards. The largest problems are not isolated style issues:

- `SPEC-022` workflow-obligation constructs are present in the AST and parser, but the interpreter still rejects them at runtime.
- Proxy/yield execution remains partially placeholder-driven, and the parser still lowers some proxy constructs to `Done`.
- `SPEC-018` runtime-verification support includes placeholder types instead of concrete runtime-side structures.
- `SPEC-010` embedding/builder APIs are documented as richer than the implementation; current builder methods are effectively no-ops and provider implementations are stubs.
- Workspace quality gates are red: `cargo clippy -D warnings` fails, `cargo fmt --check` fails, and `cargo doc` emits rustdoc warnings.

The repository therefore looks strongest as an active implementation branch with meaningful coverage, not as a converged reference implementation.

## Findings By Category

### Spec Compliance

#### Critical

- **SPEC-022 workflow obligations are not executable.**
  - Spec/API surface: `docs/spec/SPEC-022-WORKFLOW-TYPING.md`, `crates/ash-core/src/ast.rs:150`, `crates/ash-parser/src/parse_workflow.rs`, `crates/ash-parser/src/lower.rs:229`
  - Runtime behavior: `crates/ash-interp/src/execute.rs:856`
  - Details: `Workflow::Oblige` and `Workflow::CheckObligation` exist and are type-checked, but execution hard-fails with `"not yet implemented in interpreter"`. This is a direct contract break, not a polish gap.

- **Proxy/yield workflow support is not end-to-end despite surface and core representations existing.**
  - Surface/core types: `crates/ash-core/src/ast.rs:162`
  - Lowering gap: `crates/ash-parser/src/lower.rs:388`
  - Runtime gap: `crates/ash-interp/src/execute.rs:874`
  - Details: `SurfaceWorkflow::Yield` is still lowered to `CoreWorkflow::Done`, `YIELD` returns an execution error after suspending, and `PROXY_RESUME` is explicitly unimplemented. This also makes current proxy tests more optimistic than the actual runtime contract.

#### High

- **SPEC-018 runtime-verification model is represented by placeholders, not production-grade runtime structures.**
  - `crates/ash-typeck/src/runtime_verification.rs:1324`
  - Details: `MailboxRegistry`, `SourceScheduler`, `ApprovalQueue`, and `ProvenanceSink` are explicitly labeled placeholders. The spec describes concrete runtime inputs and enforcement surfaces, but the implementation still exposes stand-ins.

- **SPEC-010 embedding API is over-documented relative to implementation.**
  - Spec: `docs/spec/SPEC-010-EMBEDDING.md:38`, `docs/spec/SPEC-010-EMBEDDING.md:56`
  - Implementation: `crates/ash-engine/src/lib.rs:306`, `crates/ash-engine/src/lib.rs:318`, `crates/ash-engine/src/lib.rs:327`
  - Details: `with_http_capabilities` and `with_custom_provider` are documented but do not exist. Existing builder methods are no-ops. The engine builds, but provider registration/configuration is not actually modeled by the builder.

- **Built-in provider behavior is still stubbed.**
  - `crates/ash-engine/src/providers.rs:82`
  - `crates/ash-engine/src/providers.rs:95`
  - `crates/ash-engine/src/providers.rs:133`
  - `crates/ash-engine/src/providers.rs:146`
  - Details: stdio and filesystem provider methods mostly discard inputs and return `Value::Null`. That does not match the richer behavior implied by `SPEC-010`, `SPEC-014`, and `SPEC-016`.

- **Role-runtime semantics are only partially enforced.**
  - `crates/ash-interp/src/role_context.rs:86`
  - Details: `RoleContext::discharge` accepts any string and records it as discharged, even if the obligation was never declared on the role. That conflicts with the intent of `SPEC-019`, where role obligations are closed, mandatory obligations rather than arbitrary runtime tags.

#### Medium

- **Float handling is lossy and silent across boundaries.**
  - Parser/core lowering: `crates/ash-parser/src/lower.rs:531`
  - CLI input path: `crates/ash-cli/src/commands/run.rs:90`
  - Details: float literals are truncated to `Int` during lowering, while JSON numbers that are not integers become `Value::Null`. Silent coercion is dangerous and should be replaced with either first-class float support or an explicit rejection path.

- **Observable trace options are partially decorative.**
  - `crates/ash-cli/src/commands/trace.rs:32`
  - `crates/ash-cli/src/commands/trace.rs:72`
  - Details: `--lineage` and `--verify` are parsed, but `_args` is unused in execution. The CLI surface advertises behavior that the implementation does not honor.

- **Spec index documentation is stale and mislabels the spec set.**
  - `docs/spec/README.md:9`
  - Details: the index still maps `SPEC-002` to capability system, `SPEC-004` to policy framework, and `SPEC-005` to provenance/audit, which no longer matches the actual filenames or canonical documents in `docs/spec/`.

### Rust Style And API Quality

#### High

- **Workspace quality gates fail the project’s own standard.**
  - Policy source: `docs/plan/PLAN-INDEX.md:17`
  - `cargo clippy -D warnings` failures:
    - `crates/ash-core/tests/proxy_ast_tests.rs:175`
    - `crates/ash-parser/src/lower.rs:20`
    - `crates/ash-parser/src/parse_module.rs:16`
  - `cargo check`/`cargo doc` warnings also affect:
    - `crates/ash-typeck/src/requirements.rs:920`
    - `crates/ash-interp/src/execute.rs:916`
    - `crates/ash-parser/src/surface.rs:534`
  - Details: the repository currently does not satisfy “clippy passes with no warnings” or “cargo doc generates clean documentation”.

- **Formatting is not clean.**
  - `cargo fmt --check` failed across multiple files, including:
    - `crates/ash-interp/src/execute.rs`
    - `crates/ash-parser/tests/proxy_parser_tests.rs`
    - `crates/ash-core/tests/proxy_ast_tests.rs`

#### Medium

- **Feature flag configuration is internally inconsistent.**
  - `crates/ash-typeck/src/requirements.rs:920`
  - `crates/ash-typeck/Cargo.toml:1`
  - Details: code references `feature = "proptest"` but the crate declares no such feature. This triggers `unexpected_cfgs` and makes the intended test/config split ambiguous.

- **Public API docs have correctness issues even when coverage exists.**
  - Examples:
    - `crates/ash-parser/src/surface.rs:534` broken intra-doc link
    - `crates/ash-core/src/ast.rs:458` invalid rust code block
    - `crates/ash-typeck/src/kind.rs:4` invalid HTML tag rendering
    - `crates/ash-doc-tests/src/main.rs:6` invalid code block language

- **Documentation/API guides are partially stale.**
  - `README.md:45`
  - `docs/API.md:52`
  - Details: `README.md` points to `examples/multi_agent.ash`, which does not exist in the current tree, and `docs/API.md` contains incorrect sample code such as `pubuse provenance::*;`.

### Security Review

#### High

- **Manual `unsafe impl Send/Sync` for the Z3 context wrapper has weak justification and no proven synchronization boundary.**
  - `crates/ash-typeck/src/smt.rs:118`
  - Details: the comment says the type is used in a single-threaded manner, but the type is declared `Send + Sync` globally. That is exactly the kind of informal reasoning `unsafe` should avoid. If this type crosses thread boundaries accidentally, soundness depends on Z3 internals rather than Rust’s type system.

#### Medium

- **Silent numeric coercion can hide invalid or attacker-controlled input shape.**
  - `crates/ash-cli/src/commands/run.rs:95`
  - `crates/ash-parser/src/lower.rs:537`
  - Details: converting unsupported numbers to `Null` or truncating floats can change workflow meaning without a visible error.

- **Environment mutation uses `unsafe` in the CLI entrypoint.**
  - `crates/ash-cli/src/main.rs:76`
  - Details: this is likely required by Rust 2024’s environment APIs, but it should be isolated and documented because it broadens the unsafe surface beyond the SMT wrapper.

#### Low

- **`cargo audit` could not be completed in this sandbox.**
  - Command failure: advisory DB lock under read-only cargo home
  - Impact: dependency vulnerability status is **unverified**, not “clean”.

### Completeness

#### High

- **Project status documents overstate completion.**
  - `docs/plan/PLAN-INDEX.md:133`
  - `docs/plan/PLAN-INDEX.md:248`
  - Details: the file still declares early interpreter/CLI phases complete and describes a “working interpreter”, while Phase 18 is still in progress and core newer features are stubbed. This is governance drift, not just editorial noise.

- **Previous closeout audits no longer reflect the live workspace state.**
  - Example reference: `docs/audit/2026-03-20-final-convergence-audit.md`
  - Details: historical audits claimed full convergence and clean verification, but the current workspace fails clippy/fmt/doc hygiene and still contains runtime TODOs in areas those audits described as closed.

#### Medium

- **Testing breadth is strong, but coverage confidence is qualitative rather than measured.**
  - Evidence: substantial unit/property/integration test presence across parser, engine, interpreter, and workflow-contract files
  - Gap: no coverage report or threshold is enforced, so uncovered execution paths can still hide behind a passing test suite.

- **Proxy and workflow-contract test surfaces are ahead of runtime completeness.**
  - Examples:
    - `crates/ash-engine/tests/workflow_contracts_proptest.rs`
    - `crates/ash-interp/tests/proxy_execution_tests.rs`
  - Details: the repository has many tests for representations and partial flows, but some tested features still terminate in runtime “not yet implemented” errors.

### Project Status

#### Current Tooling State

- `cargo check --workspace`: **passes with warnings**
- `cargo test --workspace`: **passes**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **fails**
- `cargo fmt --check`: **fails**
- `cargo doc --workspace --no-deps`: **completes with rustdoc warnings**
- `cargo audit`: **not completed in this environment**

#### Outstanding `TODO` / placeholder inventory

- `crates/ash-interp/src/execute.rs:858` obligation tracking TODO
- `crates/ash-interp/src/execute.rs:867` obligation checking TODO
- `crates/ash-parser/src/lower.rs:388` proxy lowering placeholder
- `crates/ash-parser/src/lower.rs:538` float support TODO
- `crates/ash-engine/src/lib.rs:606` future tests marker
- `crates/ash-typeck/src/runtime_verification.rs:1324` verification placeholder types

## Recommendations

### 1. Make workflow obligations executable before expanding the contract surface

Current state:

```rust
Workflow::Oblige { name, .. } => {
    Err(ExecError::ExecutionFailed(format!(
        "OBLIGE '{name}' not yet implemented in interpreter"
    )))
}
```

Recommended direction:

```rust
Workflow::Oblige { name, .. } => {
    let mut next = ctx.extend();
    next.obligations.insert(name.clone())?;
    Ok(Value::Null)
}

Workflow::CheckObligation { name, .. } => {
    Ok(Value::Bool(ctx.obligations.remove(name).is_ok()))
}
```

This aligns the runtime with `SPEC-022` and removes one of the largest current spec breaks.

### 2. Remove placeholder lowering for proxy constructs

Current state:

```rust
SurfaceWorkflow::Yield { .. } => CoreWorkflow::Done
```

Recommended direction:

```rust
SurfaceWorkflow::Yield { role, request, expected_response_type, continuation, span } => {
    CoreWorkflow::Yield {
        role: role.to_string(),
        request: Box::new(lower_expr(request)),
        expected_response_type: lower_type_expr(expected_response_type),
        continuation: Box::new(lower_workflow_body(continuation, provenance)),
        span: lower_span(*span),
    }
}
```

Do not advertise proxy execution as implemented until lowering and resume semantics both exist.

### 3. Replace the `unsafe` SMT threading contract with a safe ownership model

Current state:

```rust
unsafe impl Send for SmtContext {}
unsafe impl Sync for SmtContext {}
```

Recommended direction:

```rust
pub struct SmtContext {
    context: Box<Context>,
    _not_send_sync: std::marker::PhantomData<std::rc::Rc<()>>,
}
```

If cross-thread use is genuinely required, wrap access behind a dedicated worker thread or documented synchronization boundary rather than asserting `Send + Sync`.

### 4. Make numeric boundary failures explicit

Current state silently truncates or nulls unsupported numeric values. Prefer:

```rust
serde_json::Value::Number(n) => {
    if let Some(i) = n.as_i64() {
        Ok(Value::Int(i))
    } else {
        Err(anyhow::anyhow!("non-integer numbers are not supported"))
    }
}
```

The same principle should hold for source-level float literals: reject with a typed parse/type/lowering error unless true float support exists end to end.

### 5. Bring project metadata back in sync with reality

- Update `docs/spec/README.md` to match the actual spec files.
- Reconcile `docs/plan/PLAN-INDEX.md` with current incomplete/stubbed runtime areas.
- Refresh `README.md` and `docs/API.md` examples against the live CLI and public API.

## Prioritized Task List

1. **Critical**: Implement `Workflow::Oblige` and `Workflow::CheckObligation` execution semantics and add end-to-end runtime tests.
2. **Critical**: Replace proxy/yield placeholders with real lowering and resume behavior, or explicitly gate/remove the feature from public surfaces until complete.
3. **High**: Remove or redesign `unsafe impl Send/Sync` for `SmtContext`.
4. **High**: Make `EngineBuilder` capability configuration real, or shrink `SPEC-010` and API docs to the actually supported surface.
5. **High**: Eliminate all current clippy `-D warnings` failures and fix the `unexpected_cfgs` warning in `ash-typeck`.
6. **High**: Run `cargo fmt` and keep the workspace formatting-clean.
7. **High**: Resolve rustdoc warnings and broken docs/examples so `cargo doc` is clean.
8. **Medium**: Replace silent float coercions with explicit errors or full numeric support.
9. **Medium**: Turn `trace --lineage` / `--verify` into real behavior or remove the flags.
10. **Medium**: Update `docs/spec/README.md`, `README.md`, `docs/API.md`, and `docs/plan/PLAN-INDEX.md` so governance artifacts match the live implementation.
11. **Medium**: Add a real dependency-audit workflow that can run outside this sandbox and record the result in CI.

## Closing Assessment

The Ash codebase has meaningful implementation depth and a strong testing culture, but it currently overstates convergence. The biggest issues are not “more tests needed”; they are unfinished runtime paths sitting behind already-published AST/spec/API surfaces. The fastest path to materially improving quality is to stop widening the public contract and instead close the gap between declared features and executable semantics.
