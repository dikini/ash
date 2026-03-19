# Audit: Rust Codebase Review Findings

## Scope

Executed the cluster-based Rust review checklist in [docs/audit/2026-03-19-rust-codebase-review-checklist.md](2026-03-19-rust-codebase-review-checklist.md) against the current codebase.

This review used the earlier audits as context:
- [docs/audit/2026-03-19-spec-001-018-consistency-review.md](2026-03-19-spec-001-018-consistency-review.md)
- [docs/audit/2026-03-19-task-consistency-review-non-lean.md](2026-03-19-task-consistency-review-non-lean.md)

Constraints of this audit:
- Reporting only
- No Rust source files changed
- Focus on distinguishing code bugs from spec/task drift

Review date: 2026-03-19

## Summary

The codebase is not uniformly drifted.

There are usable baseline anchors in the core AST, surface workflow vocabulary, and some ADT runtime paths. The largest implementation risks remain concentrated in the same clusters identified earlier:
- policies,
- REPL/CLI,
- streams and runtime verification,
- ADTs.

The main pattern is not random breakage. It is layered divergence: parser, lowering, type checking, runtime verification, and interpreter code often implement partially different models of the same feature.

## Overall Classification

- **Stable enough to anchor review:** core workflow vocabulary, effect lattice usage in workflow effect computation, basic ADT runtime pattern matching
- **Primarily task/spec drift reflected in code:** policy modeling, REPL command surface, runtime verification structure
- **Likely code bugs or incomplete integrations:** unreachable `receive` pipeline, disabled obligation enforcement in the verification aggregator, placeholder policy lowering, placeholder REPL type display

---

## 1. Baseline Anchors

### Findings

#### Medium — `Check` is not shape-stable across parser and core AST
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs#L35-L42)
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L197-L204)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs#L239-L287)

Core AST models `Workflow::Check` as an obligation-only operation, while surface/parser support both obligation and policy targets.

#### Medium — `Decide` policy requirement drifts across layers
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs#L30-L34)
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L186-L194)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs#L205-L237)

Core AST requires a concrete policy name, but surface/parser make policy optional.

#### Medium — `receive` exists in surface AST but is not wired into the scoped parser entrypoint
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L334-L343)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs#L103-L124)

This is a foundational integration gap that affects the stream review later.

#### Medium — workflow effects use the four-level lattice, but capability definitions still expose a different effect vocabulary
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L30-L38)
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L848-L985)
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs#L148-L154)

This is mostly model drift, not a single bug.

### Stable areas
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs#L17-L46)
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L157-L220)

The main workflow-stage vocabulary is still recognizable and coherent.

---

## 2. Policy Cluster

### Findings

#### High — parser/type-level policy support is disconnected from workflow lowering
- [crates/ash-parser/src/parse_policy.rs](../crates/ash-parser/src/parse_policy.rs#L49-L120)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs#L239-L287)
- [crates/ash-parser/src/lower.rs](../crates/ash-parser/src/lower.rs#L356-L380)
- [crates/ash-parser/src/lower.rs](../crates/ash-parser/src/lower.rs#L120-L129)

The parser has dedicated `PolicyExpr` parsing, but workflow `check` handling only parses policy instances. Lowering then converts policy expressions to debug strings and lowers policy checks to a dummy obligation.

#### High — codebase contains multiple incompatible policy representations
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L51-L76)
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L508-L540)
- [crates/ash-typeck/src/policy_check.rs](../crates/ash-typeck/src/policy_check.rs#L33-L84)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L700-L715)

The code simultaneously models:
- declarative policy definitions,
- first-class compositional `PolicyExpr` values,
- static runtime-verification policies with decision enums.

These are not unified by a single execution model.

#### Medium — policy method typing is internally inconsistent even within the type checker
- [crates/ash-typeck/src/policy_check.rs](../crates/ash-typeck/src/policy_check.rs#L105-L149)

Built-in methods like `retry` and `timeout` are documented as count/duration-style methods, but their registered argument types are still `PolicyType::Policy`.

#### Medium — SMT conflict detection exists, but it is structurally separate from policy expression checking
- [crates/ash-typeck/src/lib.rs](../crates/ash-typeck/src/lib.rs#L34-L39)
- [crates/ash-typeck/src/smt.rs](../crates/ash-typeck/src/smt.rs#L1-L36)
- [crates/ash-typeck/src/policy_check.rs](../crates/ash-typeck/src/policy_check.rs#L1-L33)

SMT infrastructure is present, but the policy-expression checker is a separate path. The implementation does not present one clearly unified policy pipeline.

### Classification
Mostly **implementation drift caused by unresolved policy-model drift**, plus one real code issue: policy checks are lowered through placeholders rather than a meaningful core representation.

---

## 3. REPL and CLI Cluster

### Findings

#### High — there are two separate REPL implementations with different command surfaces
- [crates/ash-repl/src/lib.rs](../crates/ash-repl/src/lib.rs#L338-L377)
- [crates/ash-cli/src/commands/repl.rs](../crates/ash-cli/src/commands/repl.rs#L80-L159)

`ash-repl` supports `:help`, `:quit`, `:type`, `:ast`, and `:clear`. The CLI REPL command supports `:help`, `:quit`, `:exit`, `:type`, `:bindings`, and `:clear`, but not `:ast`.

#### High — REPL behavior depends on which entrypoint is used
- [crates/ash-repl/src/main.rs](../crates/ash-repl/src/main.rs#L1-L11)
- [crates/ash-cli/src/commands/repl.rs](../crates/ash-cli/src/commands/repl.rs#L1-L40)

The standalone REPL binary runs `ash_repl::Repl`, while the CLI implements its own REPL loop instead of delegating.

#### Medium — `ash-repl` type inspection is still placeholder-level
- [crates/ash-repl/src/lib.rs](../crates/ash-repl/src/lib.rs#L385-L397)

`show_type()` prints `Type: (inferred from context)` instead of actual inferred types.

#### Medium — history/config behavior diverges across REPL implementations
- [crates/ash-repl/src/lib.rs](../crates/ash-repl/src/lib.rs#L163-L181)
- [crates/ash-repl/src/lib.rs](../crates/ash-repl/src/lib.rs#L252-L258)
- [crates/ash-cli/src/commands/repl.rs](../crates/ash-cli/src/commands/repl.rs#L40-L74)

`ash-repl` uses a project directory-backed history path and automatic save/load. CLI REPL uses an explicit `--history` path and separate persistence behavior.

#### Medium — engine providers still encode provider-level effects in a coarse way
- [crates/ash-engine/src/providers.rs](../crates/ash-engine/src/providers.rs#L39-L49)
- [crates/ash-engine/src/providers.rs](../crates/ash-engine/src/providers.rs#L66-L76)
- [crates/ash-engine/src/providers.rs](../crates/ash-engine/src/providers.rs#L110-L120)

This preserves the same effect-granularity drift noted in the spec audit.

### Classification
A mix of **task/spec drift reflected in code** and **real duplication debt**. The clearest code-level issue is that there is no single authoritative REPL implementation.

---

## 4. Streams and Runtime Verification Cluster

### Findings

#### High — `receive` is not an end-to-end feature
- [crates/ash-parser/src/parse_receive.rs](../crates/ash-parser/src/parse_receive.rs#L65-L95)
- [crates/ash-parser/src/parse_receive.rs](../crates/ash-parser/src/parse_receive.rs#L129-L195)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs#L103-L122)
- [crates/ash-parser/src/lower.rs](../crates/ash-parser/src/lower.rs#L183-L189)

`receive` parsing exists in isolation, but the main workflow parser does not dispatch to it, and lowering reduces surface `Receive` to `CoreWorkflow::Done`.

#### High — control-stream handling is structurally inconsistent
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs#L335-L389)
- [crates/ash-core/src/stream.rs](../crates/ash-core/src/stream.rs#L134-L169)
- [crates/ash-interp/src/execute_stream.rs](../crates/ash-interp/src/execute_stream.rs#L47-L61)
- [crates/ash-interp/src/execute_stream.rs](../crates/ash-interp/src/execute_stream.rs#L161-L179)

Surface syntax and runtime stream structures do not model control arms the same way.

#### High — compile-time declaration checking and runtime checking disagree on `receive`
- [crates/ash-typeck/src/capability_check.rs](../crates/ash-typeck/src/capability_check.rs#L416-L423)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L341-L353)
- [crates/ash-interp/src/execute_observe.rs](../crates/ash-interp/src/execute_observe.rs#L67-L80)
- [crates/ash-interp/src/execute_set.rs](../crates/ash-interp/src/execute_set.rs#L47-L63)
- [crates/ash-interp/src/exec_send.rs](../crates/ash-interp/src/exec_send.rs#L48-L64)

Declaration verification skips `Workflow::Receive`, while runtime verification expects declared receives, and interpreter execution does not consistently route through runtime verification anyway.

#### High — the runtime verification model is fragmented
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L14-L69)
- [crates/ash-typeck/src/capability_typecheck.rs](../crates/ash-typeck/src/capability_typecheck.rs#L10-L53)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L1057-L1097)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L953-L1040)

Capability schema state, runtime context, policy state, and operation verification are not centered around one coherent model.

#### High — obligation enforcement is effectively absent in the main aggregate path
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L610-L680)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L1138-L1155)

Obligation checking is modeled, but the aggregate verifier does not use the full role/obligation checks in a way that matches the documented intent.

#### Medium — deny and transform semantics drift from the documented model
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L700-L715)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs#L1000-L1030)

Per-operation deny returns `OperationResult::Denied`, while transform support is exposed elsewhere but not represented in the static policy decision type.

### Classification
This cluster contains the strongest **actual code issues**, not just documentation drift.

---

## 5. ADT Cluster

### Findings

#### High — ADT type modeling in code follows the later `TypeExpr`-based task direction, not the earlier SPEC-020 core-shape wording
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs#L406-L459)
- [crates/ash-parser/src/parse_type_def.rs](../crates/ash-parser/src/parse_type_def.rs#L20-L71)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L40-L66)

The implementation is internally more consistent than the task chain, but it has clearly chosen the `TypeExpr` route rather than the older direct-`Type` model described in SPEC-020.

#### High — runtime value shape does not match SPEC-020’s `typ` / `variant` model
- [crates/ash-core/src/value.rs](../crates/ash-core/src/value.rs#L48-L66)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L311-L319)

Runtime variants only store constructor `name` plus fields. They do not store the enclosing type name as SPEC-020 describes.

#### Medium — pattern typing still uses a record-tag approximation for variants
- [crates/ash-typeck/src/check_pattern.rs](../crates/ash-typeck/src/check_pattern.rs#L152-L234)
- [crates/ash-typeck/src/check_pattern.rs](../crates/ash-typeck/src/check_pattern.rs#L13)

Interpreter pattern matching works directly on `Value::Variant`, but the type checker still recognizes variant patterns by treating expected types as tagged records with `__variant`.

#### Medium — exhaustiveness exists as a separate module, but the type checker model still does not fully converge with runtime matching
- [crates/ash-typeck/src/exhaustiveness.rs](../crates/ash-typeck/src/exhaustiveness.rs#L1-L120)
- [crates/ash-interp/src/pattern.rs](../crates/ash-interp/src/pattern.rs#L133-L180)

There is one dedicated exhaustiveness module, which is good, but its type model is still separate from the runtime-value representation.

#### Medium — control-link work is implemented as runtime values, but remains conceptually bolted onto the ADT wave
- [crates/ash-interp/src/eval.rs](../crates/ash-interp/src/eval.rs#L150-L186)
- [crates/ash-core/src/value.rs](../crates/ash-core/src/value.rs#L56-L62)

The spawn/split path exists and is coherent locally, but it still sits adjacent to the ADT review rather than feeling like a fully separate subsystem.

#### Medium — Option/Result standard library is useful but narrower than SPEC-020
- [std/src/option.ash](../std/src/option.ash#L1-L67)
- [std/src/result.ash](../std/src/result.ash#L1-L84)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L422-L462)

The current libraries provide core helpers, but not the full helper surface described in SPEC-020.

### Stable areas
- [crates/ash-interp/src/pattern.rs](../crates/ash-interp/src/pattern.rs#L133-L180)

Runtime variant-pattern matching is locally coherent.

### Classification
Mostly **code that chose one side of the documentation drift**, plus some remaining type-check/runtime mismatches. Less broken than the streams/runtime-verification cluster, but still not fully aligned with its own docs.

---

## Priority for Follow-Up Rust Review

### Highest priority
1. Streams and runtime verification
2. Policy handling and lowering
3. REPL duplication
4. ADT type/runtime alignment

### Lower priority
5. Baseline anchors and foundational naming cleanup

## Conclusion

The Rust codebase is reviewable, but it should not be judged against a single document source.

The most accurate interpretation is:
- some code follows stable core intent,
- some code follows later task decisions rather than spec text,
- some code exposes genuine end-to-end integration gaps.

The streams/runtime-verification cluster is the clearest source of likely implementation bugs. The policy and REPL clusters are more heavily affected by unresolved model drift. The ADT cluster is partially coherent in code, but it has clearly diverged from parts of the written plan/spec trail.

No Rust source files were modified as part of this audit.
