# Design Note: Interface Laws — Syntax, Semantics, and Curry-Howard Roadmap

**Status:** Draft  
**Scope:** Language Grammar, Type System, Synthetic Testing, Future Verification  
**Related:** SPEC-078, SPEC-079, TASK-1026, TASK-1029, DESIGN-031, DESIGN-034  

---

## 1. Problem Statement

Ash now has source-visible algebraic interfaces (`Semigroup`, `Monoid`, `Functor`, `Applicative`, `Monad`, `Comonad`) under `std::algebra`. These interfaces carry semantic obligations — monad laws, functor laws, semigroup associativity — that are currently documented only in handoff prose ([TASK-1026](../plan/audits/TASK-1026-algebra-law-test-handoff.md), [TASK-1036](../plan/audits/TASK-1036-comonad-law-test-handoff.md)).

There is no syntax for declaring these laws inside `.ash` source. This means:

- Law names and statements can drift out of sync with method names.
- Synthetic test generation (TASK-1029) must reconstruct law structure from external prose rather than the AST.
- Future verification (SMT, Lean, type-directed proof search) has no machine-readable propositions to work from.

What is needed is a declaration form that says: "this interface method combination must satisfy the following proposition for all valid inputs, modulo an explicit equivalence relation."

---

## 2. Design Decisions

The following decisions are frozen for this design note:

| # | Decision | Rationale |
|---|---|---|
| D1 | **Different equivalence relations stay different.** No overloading of `==` in law bodies. Structural equality for `Pure` types, observational equivalence for `Act`/`Proc`/`Workflow`, custom equivalence passed as constraints. | Prevents silent semantic changes when a type moves between effect levels. |
| D2 | **Proofs are not first-class values** unless a specific implementation requires it. Law inhabitation stays local to `impl` blocks. | Avoids committing to full dependent types before the core language is stable. |
| D3 | **No automatic law inheritance.** If `Monad` requires `Applicative`, `impl Monad<Option>` does not auto-inherit `Applicative` proof obligations. May reuse linked law findings if helpful. | Keeps proof/test locality explicit; inheritance graphs can be added later without breaking existing code. |
| D4 | **External proof assistant interop (Lean, Coq) is deferred.** May be used in the future, but no syntax is reserved for it now. | Avoids premature commitment to a specific prover's term language. |
| D5 | **Long-term direction: Curry-Howard correspondence.** Laws are uninhabited dependent types; proofs are total terminating programs; synthetic tests are proof search. | Provides a unified theoretical framework without requiring immediate implementation. |
| D6 | **Module laws are exportable.** They cross module boundaries via imports, just like types and functions. | Module laws document cross-module invariants and enable downstream synthetic testing. |
| D7 | **Private functions may appear in proofs.** A `proof` block has access to the full module scope, including private items. | Proofs are not runtime code; they are static evidence that may need to reason about implementation details. |
| D8 | **Proofs are not exported.** Only proof *results* (valid / broken / untested) are visible externally. | Proof terms may reference private internals; exporting them would leak implementation details. External consumers see only the verification status. |

---

## 3. Proposed Syntax

### 3.1 Law Declaration in Interfaces

A `law` is a type-level proposition with no runtime representation. It references interface methods (bound in scope), an explicit equivalence relation, and universally quantified parameters.

```ash
pub interface Semigroup<A> {
    append(a: A, b: A) -> A

    -- `eq` is an explicit equivalence relation constraint.
    -- The law body is a proposition, not executable code.
    law associativity(a: A, b: A, c: A, eq: Eq<A>)
      : eq.equiv(append(append(a, b), c), append(a, append(b, c)))
}
```

Syntax:

```text
law <Name>(<params>) [where <constraints>] : <proposition>
```

- `Name` is an identifier, scoped to the interface.
- `params` are universally quantified. They may include type variables (inferred from the interface signature) and value variables.
- `constraints` are optional interface bounds (e.g., `where A: Ord`).
- `proposition` is an expression of type `Prop` (uninhabited, no runtime value). In Stages 1–2, `Prop` is a convention (marker type). In Stage 3, it becomes a distinct kind with dedicated typing rules.
- The proposition may reference any function in scope, including imported items, just like an ordinary function body.
- Callable types may be written `(A) -> B` or `A -> B`; both are valid Ash syntax for single-argument functions.


### 3.1.1 AST Design Note

The Ash parser uses two patterns for declarations:

1. **Interface/impl-scoped items** are stored as fields on `InterfaceDef` and `ImplDef`:
   - `InterfaceDef` has `methods: Vec<InterfaceMethodSig>` and `associated_types: Vec<AssociatedTypeDecl>`
   - `ImplDef` has `methods: Vec<ImplMethodDef>` and `associated_type_bindings: Vec<AssociatedTypeBinding>`

2. **Module-scoped items** are variants of the `Definition` enum.

`law` and `proof` follow both patterns:

- `law` inside `interface { ... }` → added as `laws: Vec<LawDef>` on `InterfaceDef`
- `proof` inside `impl { ... }` → added as `proofs: Vec<ProofDef>` on `ImplDef`
- `law`/`proof` at module scope → added as `Definition::Law(LawDef)` / `Definition::Proof(ProofDef)` variants

**Rationale:** This matches Ash's existing parser architecture. Interface bodies are parsed with specialized fields, not a generic `Vec<Definition>`. Module bodies use the `Definition` enum for flat, extensible item lists.


### 3.2 Monad Laws with Explicit Equivalence

```ash
pub interface Monad<M : * -> *> where M: Applicative {
    unit(A) -> M<A>
    bind(M<A>, (A) -> M<B>) -> M<B>

    law left_identity<A, B>(A, (A) -> M<B>, Eq<M<B>>)
      : equiv(bind(unit(a), f), f(a))

    law right_identity<A>(m: M<A>, eq: Eq<M<A>>)
      : eq.equiv(bind(m, unit), m)

    law associativity<A, B, C>(
        m: M<A>,
        f: (A) -> M<B>,
        g: (B) -> M<C>,
        eq: Eq<M<C>>
    ) : eq.equiv(bind(bind(m, f), g), bind(m, |x| -> bind(f(x), g)))
}
```

Note: `eq` is passed explicitly. For `Pure` carriers like `Option` or `List`, `Eq` is structural equality. For tower carriers like `Act`, `Proc`, or `Workflow`, a different equivalence relation (e.g., `BoundedEquiv`) is required.

### 3.3 Tower Carrier with Observational Equivalence

```ash
pub interface ActMonad where Self: Monad<Act> {
    -- For Act, we need bounded observational equivalence, not structural equality.
    law act_associativity<A, B, C>(
        ma: Act<A>,
        f: (A) -> Act<B>,
        g: (B) -> Act<C>,
        equiv: BoundedEquiv<Act<C>>
    ) : equiv.equiv(bind(bind(ma, f), g), bind(ma, |x| -> bind(f(x), g)))
}
```

### 3.4 Proof Terms in Impl Blocks (Optional)

When implementing a law-bearing interface, an `impl` may optionally provide a `proof` block. If omitted, the law remains unproven and synthetic tests apply.

```ash
pub impl Semigroup<String> {
    append(a, b) = string::concat(a, b)

    -- Optional proof block. Must be total and terminating.
    proof associativity(a, b, c, eq) {
        -- For String, concatenation associativity is definitional.
        by_definition  -- new keyword, requires lexer update
    }
}
```

```ash
pub impl Monad<Option> {
    unit(value) = Some { value }
    bind(ma, f) = match ma {
        Some { value } => f(value),
        None => None,
    }

    -- No proof block provided → synthetic test obligation.
}
```

Proof syntax:

```text
proof <Name>(<params>) [where <constraints>] {
    <total-expression>
}
```

- The proof body must be a total, terminating expression.
- `by_definition` is a shorthand for definitional equality (compiler verifies).
- `by test { ... }` explicitly delegates to the synthetic test runner.

### 3.5 Proof by Synthetic Test (Explicit Delegation)

```ash
impl Monad<Act<A>> {
    -- For tower carriers, synthetic tests are the default.
    proof act_associativity(ma, f, g, equiv) by test {
        generator: bounded_act_generator(),
        equivalence: equiv,
        schedule: deterministic_schedule(),
        max_steps: 100,
    }
}
```

---

## 4. Equivalence Relation Types

```ash
-- Structural equality for Pure types
pub interface Eq<A> {
    equiv(a: A, b: A) -> Bool
}

```ash
pub interface BoundedEquiv<C : * -> *> {
    equiv(C<A>, C<A>) -> EquivResult
}

pub enum EquivResult {
    Equal,
    NotEqual { counterexample: Trace },
    Timeout,  -- bounded execution exceeded
}
```

**Note:** `BoundedEquiv` returns `EquivResult`, not `Bool`. Law propositions are typed as `Prop`, which accepts any expression that reduces to a truth value. The compiler treats `EquivResult::Equal` as truthy and `NotEqual`/`Timeout` as falsy within law contexts. Alternatively, a law may explicitly match on the result:

```ash
law act_associativity<A, B, C>(..., equiv: BoundedEquiv<Act<C>>)
  : match equiv.equiv(bind(bind(ma, f), g), bind(ma, |x| -> bind(f(x), g))) {
      Equal => true,
      NotEqual { counterexample: _ } => false,
      Timeout => false
  }
```

---

## 5. Compiler and Tooling Behavior

### Stage 1: Parse and Store (Now)

- The parser accepts `law` declarations inside interface bodies.
- The typechecker verifies that all names referenced in the proposition exist and are well-typed.
- `law` nodes are stored in the AST with no runtime lowering.
- `proof` blocks inside `impl` are parsed and stored; no totality checking.

### Stage 2: Synthetic Test Generation (TASK-1029)

- The test runner extracts `law` nodes from the AST.
- For each `impl` without a `proof` block, generate small-world tests:
  - Law parameters become generators.
  - The law proposition becomes an assertion.
  - The interface instance becomes the test subject.
- Law failures report: interface name, instance key, law name, seed, minimized counterexample.
- **Synthetic tests are development-time only.** They do not run during production builds unless explicitly requested.
- **Opt-out:** Developers may skip synthetic tests via `--skip-law-tests` (skips all laws) or `--skip-law-test=<name>` (skips specific law). Per-law opt-out via attributes is deferred to Phase 140+.
- **Evidence caching:** Test results may be cached in a dedicated law cache file (e.g., `.ash/law-cache.toml`) with seed, timestamp, and source hash. Production builds trust pre-verified results without re-execution. The cache is separate from `ash.lock` (which is for dependency resolution only).

### Stage 3: Totality Checking (Future)

- The compiler checks `proof` blocks for termination.
- Rejects non-total proofs with a diagnostic.
- `by_definition` is verified by normalization.

### Stage 4: External Prover Integration (Deferred)

- Emit law types to Lean, Coq, or SMT.
- Import verified proofs back as `proof` blocks.
- Syntax: `by lean { ... }` or `by z3 { ... }` (not reserved now).

---

## 6. Curry-Howard Correspondence

This syntax is designed to map cleanly onto the Curry-Howard correspondence as Ash evolves:

| Ash Concept | CH Mapping |
|---|---|
| `interface` | Signature / type theory context |
| `law` | Dependent function type `(a: A) -> (b: B) -> Eq<C> -> Prop` |
| `proof` | Lambda term inhabiting the law type |
| `by_definition` | Reflexivity combinator |
| Synthetic test | Proof search / falsification |
| `Prop` | Universe of uninhabited types (propositions) |

The key insight is that **laws are not special** — they are types that happen to have no runtime representation. This gives a unified framework where:

- Interfaces are signatures.
- Impls are implementations.
- Laws are theorems.
- Proofs are programs.
- Tests are proof search.

---

## 7. Module-Scoped Laws

Modules have boundaries, invariants, and cross-cutting concerns that are not tied to any single interface. Module-scoped laws apply the same principles (D1–D5) at the module level.

### 7.1 Motivation

Consider `std::io::path`. It defines `from_string`, `join`, `parent`, `file_name`, and `is_absolute`. These functions have module-level invariants:

- `join(from_string("/"), "a")` should equal `from_string("/a")`.
- `parent(join(p, "a"))` should equal `Some(p)` for non-root `p`.
- `is_absolute(from_string(s))` should equal `string::starts_with(s, "/")`.

These are not interface laws — they are **module coherence laws** that relate multiple functions within a module. They are useful for:

- Documentation: what guarantees does the module provide as a whole?
- Synthetic testing: generate tests that exercise function combinations.
- Refactoring safety: changing `join` requires checking all module laws.

### 7.2 Syntax

Module laws are declared at module scope, outside any `interface` or `impl`:

```ash
-- std/src/io/path.ash

pub type PathBuf = PathBuf { inner: String };

pub fn from_string(s: String) -> PathBuf { ... }
pub fn join(base: PathBuf, child: String) -> PathBuf { ... }
pub fn parent(path: PathBuf) -> Option<PathBuf> { ... }
pub fn is_absolute(path: PathBuf) -> Bool { ... }

-- Module law: join preserves absoluteness
law join_preserves_absolute(base: PathBuf, child: String, eq: Eq<PathBuf>)
  : match is_absolute(base) {
      true => is_absolute(join(base, child)),
      false => true
  }

-- Module law: parent of join is the original path (for non-root)
law parent_of_join(base: PathBuf, child: String, eq: Eq<Option<PathBuf>>)
  : eq.equiv(parent(join(base, child)), Some(base))

-- Module law: from_string produces consistent results
law from_string_consistent(s: String, eq: Eq<PathBuf>)
  : eq.equiv(from_string(s), from_string(s))
```

Syntax:

```text
law <Name>(<params>) [where <constraints>] : <proposition>
```

- Same syntax as interface laws, but scoped to the module.
- May reference any function in scope, including imported items, just like an ordinary function body.

### 7.3 Proof Terms for Module Laws

Module laws are proven at module scope, not inside an `impl`. A `proof` block has access to the full module scope, including private functions and types (D7).

```ash
-- In the same module, or in a test module
proof join_preserves_absolute(base, child, eq) {
    by_definition
}

proof parent_of_join(base, child, eq) {
    -- Pattern match on base to show parent(join(base, child)) == Some(base)
    match base {
        PathBuf { inner: b } =>
            -- Requires reasoning about string::concat and string::starts_with
            -- Private helper functions are visible here.
            by_definition  -- new keyword, requires lexer update
    }
}
```

If no `proof` is provided, synthetic tests apply.

### 7.4 Proof Visibility and Export

Proof terms are **not exported** (D8). They may reference private implementation details, and exporting them would leak those internals. What crosses module boundaries is the **law declaration** and the **proof result**:

```ash
-- Exported from std::io::path:
--   law join_preserves_absolute(...)  -- the proposition
--   proof result: valid               -- the verification status

-- NOT exported:
--   The body of the proof block
--   Any private functions referenced in the proof
```

Proof results are one of:

| Result | Meaning |
|---|---|
| `valid` | A `proof` block was provided and accepted by the compiler. |
| `tested` | No `proof` block; synthetic tests passed. |
| `broken` | Synthetic tests found a counterexample. |
| `untested` | No `proof` block and synthetic tests have not run. |

Downstream modules that import a law see its result. They do not see the proof term. This allows modules to claim "this law holds" without revealing how they know it.

### 7.5 Differences from Interface Laws

| Aspect | Interface Law | Module Law |
|---|---|---|
| Scope | Inside `interface { ... }` | At module top level |
| References | Interface methods, parameters | Any module-visible function or value, including private items in proofs (D7) |
| Proof site | Inside `impl` block | At module scope or in test module |
| Proof export | N/A — proofs live in `impl` | Proof terms are not exported; only results cross boundaries (D8) |
| Law export | Via the interface | Via module import, same as types and functions (D6) |
| Inheritance | None (D3) | None — each module is independent |
| Synthetic test | Per `impl` instance | Per module |

### 7.6 Use Cases

**Algebraic coherence:**

```ash
-- std/src/algebra/monad.ash

law monad_implies_applicative_unit<M, A>(v: A, eq: Eq<M<A>>)
  where M: Monad
  : eq.equiv(Monad::unit(v), Applicative::pure(v))
```

**API consistency:**

```ash
-- std/src/option.ash

law unwrap_or_is_none(opt: Option<A>, default: A, eq: Eq<A>)
  : match is_none(opt) {
      true => eq.equiv(unwrap_or(opt, default), default),
      false => true
  }
```

**Security invariants:**

```ash
-- std/src/act.ash

law guard_idempotent<A>(p: String, ma: Act<A>, equiv: BoundedEquiv<Act<A>>)
  : equiv.equiv(guard(p, guard(p, ma)), guard(p, ma))
```

---

## 8. Open Questions (Deferred)

The following questions are intentionally left open. They do not block Stage 1 syntax:

| Question | Status |
|---|---|
| Should `Prop` be a distinct kind or just a convention? | **Resolved: Distinct kind in Stage 3.** Convention (marker type) for Stages 1–2; promoted to a distinct universe `Kind::Prop` when hand-written proofs land. This enables totality checking, proof irrelevance, and static separation from runtime values. |
| What is the exact `BoundedEquiv` contract for `Proc` and `Workflow`? | Deferred to tower carrier semantics work. |
| Should the compiler generate `Eq` instances automatically for `Pure` types? | **Resolved: Out of scope.** Derivation, macros, and auto-generation are separate concerns. This design's features *use* `Eq` instances; how they are produced (hand-written, derived, or generated) does not affect law syntax or semantics. |
| Should module laws be exportable/importable across module boundaries? | **Resolved: Yes (D6).** Module laws are exported via the same mechanism as types and functions. |
| Can module laws reference private functions? | **Resolved: Yes (D7).** Proofs may reference private items. Law propositions should reference public API. |
| Should proof terms be exported? | **Resolved: No (D8).** Only proof results (valid/tested/broken/untested) cross module boundaries. |
| Should proof names be statically checked? | **Resolved: Yes.** The compiler rejects `proof unknown_law(...) { ... }` at Stage 1. Proof names must match declared laws in the current module or interface. |
| Can a module law have multiple proofs? | **Resolved: One proof per law.** A module law may have exactly one `proof` block. Multiple proofs for the same law are a compile-time error. Interface laws may have one proof per `impl` instance. |
| Must law propositions reference only Pure functions? | **Resolved: Yes, with exception.** Law propositions are `Prop`-typed and must reference only `Pure` functions. Referencing `Act`, `Proc`, or `Workflow`-returning functions in a law body is a compile-time error. Proofs are similarly restricted to pure, total, terminating expressions. **Exception:** Tower carrier laws (`Act`, `Proc`, `Workflow`) cannot be proven by pure proof terms. They must use `by test` (synthetic testing) because their propositions reference effectful computations. The `by test` delegation is not a proof term; it is an external verification strategy. |
| Should circular proofs be detected? | **Resolved: Yes, in Stage 3.** Circular proof dependencies (proof A references law B, proof B references law A) are detected during totality checking and rejected with a diagnostic. |
| Should law checking be incremental? | **Resolved: Yes.** The compiler tracks law dependencies. Changing a law or a function referenced by a law invalidates only the affected proofs, not the entire module. |
| How are cached test results invalidated? | **Resolved: Source hash.** The law cache (`.ash/law-cache.toml`) entries include a hash of the law's source text and the source text of all functions it references. If any referenced source changes, the cached result is invalidated and tests re-run. For coarse-grained invalidation, a module-level source hash may be used. |
| Do law changes affect semantic versioning? | **Resolved: Yes, with nuance.** Adding a law is a **minor** version bump (new obligation, backward-compatible for callers). Removing or changing a law is a **major** version bump (breaking change). Changing a law's proof result (e.g., from `untested` to `valid`) is not a breaking change. |
| Are laws transitive across packages? | **Resolved: No.** Laws are local to their declaring module. Importing module B into module A does not make B's laws into A's obligations. Downstream modules may reference B's functions in their own laws, but B's law status does not propagate. |

---

### Deferred to Later Phases

The following features are intentionally out of scope for Phase 136 and will be addressed in future work:

| Feature | Reason | Planned Phase |
|---|---|---|
| Attribute syntax (`#[no_test]`, `#[trace]`) | Ash has no attribute parser. Use CLI flags instead. | Phase 140+ |
| `by test { ... }` inline configuration | Complex parser extension. Use `by_test` keyword + `.ash/law-config.toml`. | Phase 140+ |
| `Eq` interface in stdlib | Forward reference. Will be added as part of integration tasks. | Phase 136 |
| Full `BoundedEquiv` implementation | Requires tower carrier semantic design (SPEC-002). | Phase 140+ |
| Cross-module law inheritance | D3 decision: no automatic inheritance. | Not planned |
| External prover integration (Lean/Coq) | D4 decision: deferred until needed. | Phase 150+ |


## 9. Relation to Existing Work

| Artifact | Relation |
|---|---|
| [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md) | Provides the `std::algebra` interfaces that will first use `law` syntax. |
| [TASK-1026](../plan/audits/TASK-1026-algebra-law-test-handoff.md) | Handoff prose becomes the initial law declarations in `.ash` source. |
| [TASK-1029](../plan/tasks/TASK-1029-generated-algebra-law-tests.md) | Consumes `law` AST nodes to generate synthetic tests. |
| [DESIGN-031](DESIGN-031-GENERALIZED-DO-NOTATION.md) | `bind` laws are exactly the monad laws; `do` blocks are proof terms for sequential composition. |
| [DESIGN-034](DESIGN-034-TOTAL-TYPE-COMPUTATION.md) | Totality checking for `proof` blocks builds on the same normalization infrastructure. |

---

## 10. Acceptance Criteria

Before this design note is promoted to normative status:

- [ ] At least one `std::algebra` interface (`Semigroup`, `Monoid`, `Functor`, `Applicative`, `Monad`, or `Comonad`) contains `law` declarations in live `.ash` source.
- [ ] The parser accepts `law` and `proof` syntax without error.
- [ ] The typechecker verifies that law propositions reference existing methods and types.
- [ ] A synthetic test is generated from at least one law declaration and executed by the SPEC-077 runner.
- [ ] At least one module (e.g., `std::io::path` or `std::option`) contains a module-scoped `law` declaration.
- [ ] A CHANGELOG.md entry records the syntax addition.

---

## 11. Proof Execution Traces and Observability

Proofs are executable code — they run during type checking. During proof development, observing execution traces is valuable for both human developers and AI agents. This section sketches a minimal, opt-in observability layer without committing to a full debugger.

### 11.1 Motivation

A proof may fail for subtle reasons:

```ash
proof parent_of_join(base, child, eq) {
    match base {
        PathBuf { inner: b } =>
            -- Why does this not reduce to `Some(base)`?
            by_definition  -- new keyword, requires lexer update
    }
}
```

The compiler reports: "proof does not reduce to `()`". But *where* in the reduction chain did it diverge? A trace of the normalization steps would show:

1. `parent(join(base, child))` → `parent(PathBuf { inner: concat(b, "/", child) })`
2. `parent(PathBuf { inner: ... })` → `Some(PathBuf { inner: ... })`
3. `Some(PathBuf { inner: ... })` ≠ `Some(base)` because `...` is not `b`

This is not a runtime debugger — it is a **normalization trace**. The proof is pure, total, and terminating (or rejected). The trace is a sequence of definitional reductions.

### 11.2 Trace Output Format

When proof tracing is enabled, the compiler emits a structured trace:

```json
{
  "law": "std::io::path::parent_of_join",
  "status": "broken",
  "steps": [
    { "expr": "parent(join(base, child))", "reduces_to": "parent(PathBuf { inner: concat(b, \"/\", child) })" },
    { "expr": "parent(PathBuf { inner: concat(b, \"/\", child) })", "reduces_to": "Some(PathBuf { inner: concat(b, \"/\", child) })" },
    { "expr": "eq.equiv(Some(PathBuf { inner: concat(b, \"/\", child) }), Some(base))", "reduces_to": "false" }
  ],
  "stuck_at": "eq.equiv(Some(PathBuf { inner: concat(b, \"/\", child) }), Some(base))",
  "reason": "concat(b, \"/\", child) is not definitionally equal to b"
}
```

This trace is:
- **Deterministic**: Same proof, same trace.
- **Finite**: Proofs are terminating; traces are bounded.
- **Inspectable**: Can be consumed by LSP, CLI, or AI agents.

### 11.3 Activation and Scope

Proof tracing is **opt-in** and **per-proof**:

```ash
-- Enable tracing for this proof only
--trace-proofs flag
proof parent_of_join(base, child, eq) {
    by_definition
}
```

Or via compiler flag:

```bash
ash check --trace-proofs=std::io::path::parent_of_join
```

Tracing is never enabled by default. It is a development aid, not a production feature.

Trace depth is bounded by a default maximum (e.g., 50 steps). This prevents runaway traces for complex proofs. The limit is configurable:

```ash
-- Per-proof override
--trace-proofs --trace-max-depth=100
proof complex_law(...) { ... }
```

Or globally:

```bash
ash check --trace-proofs --trace-max-depth=100
```

When the depth limit is reached, the trace records:

```json
{ "expr": "...", "reduces_to": "...", "note": "depth limit reached; further steps abbreviated" }
```

### 11.4 LSP Integration

The LSP server can expose proof traces as:

- **Hover information**: Show the last N reduction steps on hover over a `proof` block.
- **Code lens**: "Show trace" action above `proof` declarations.
- **Diagnostics**: Attach trace excerpts to "proof does not reduce" errors.

### 11.5 AI Agent Integration

An AI agent (or Codex sub-agent) can:

1. Request a trace for a failing proof.
2. Inspect the stuck expression.
3. Suggest a fix: "The proof fails because `parent` does not strip the child component. Consider using a private helper `strip_last_component` or reformulating the law."

This turns proof development into an interactive, observable process rather than a black-box "correct/incorrect" judgment.

### 11.6 Bloat Avoidance

To keep this feature minimal:

- **No step-through debugger**: Traces are post-hoc, not interactive.
- **No variable watches**: The trace shows expressions, not mutable state.
- **No breakpoints**: Proofs are total; there is no "pause at step 5."
- **No serialization of large values**: Traces abbreviate large structures (e.g., `List<Int>[...]` for a list of 100 elements).
- **Depth limits**: Default maximum (e.g., 50 steps) with per-proof and global override.

The trace is a **normalization log**, not a general-purpose debugger. It is sufficient for proof development and nothing more.

### 11.7 Open Questions

| Question | Status |
|---|---|
| Should traces include source spans for each reduction step? | Deferred; useful for LSP but not required for initial trace output. |
| Can traces be replayed or diffed across proof versions? | Deferred; interesting for regression testing but out of scope. |
| Should synthetic test failures also produce traces? | Deferred; synthetic tests are external to the proof system. |
| Should traces be emitted for successful proofs, or only failures? | **Resolved: On request.** Failure traces are automatic. Success traces are opt-in via `--trace-proofs flag` or `--trace-proofs`. |
| What is the maximum trace depth before abbreviation kicks in? | **Resolved: Default depth with optional override.** A sensible default (e.g., 50 steps) is applied. Users may override per-proof or globally. |

---


---

## 12. Effect-Related Laws and Capability Integration *(Highly Speculative — Future Design Note)*

**Status:** Intent only. Not planned for Stages 1–3. Will be specified in a separate design note when prerequisite features (capabilities, provenance, policy syntax) are stable.

Future Ash effect systems — capabilities, policies, and the effect lattice (`Pure < Act < Proc < Workflow`) — will need their own law syntax. This section records design intent for effect-aware laws without specifying syntax.

### 12.1 Motivation

The current law design handles algebraic invariants (equality, equivalence, totality). Effect-related laws handle behavioral invariants:

- **Capability safety:** A function that claims to require `capability Fs` must actually use `Fs` (no overclaiming) and must not use capabilities it does not declare (no underclaiming).
- **Effect monotonicity:** A `Pure` function must not call `Act` functions; an `Act` function must not call `Proc` functions.
- **Policy exhaustiveness:** A policy match must cover all possible capability/role combinations.
- **Governance preservation:** A workflow's provenance chain must be non-empty for all `Operational` effects.

### 12.2 Design Principles

Effect-related laws use the same `law` syntax but with effect-aware propositions:

```ash
-- Hypothetical future syntax
law fs_capability_sound(file_path: String, content: String)
  : requires(write_file, Fs)
    ensures(write_file(file_path, content), Provenance::non_empty)
```

Key differences from algebraic laws:

| Aspect | Algebraic Law | Effect Law |
|---|---|---|
| Proposition type | `Prop` | `Prop` (same kind) |
| Parameters | Values + equivalence relations | Values + capability contexts + effect traces |
| Proof target | Definitional equality | Capability trace validity |
| Synthetic test | Small-world generators | Capability boundary fuzzing |
| Failure mode | Counterexample value | Counterexample trace + capability violation |

### 12.3 Integration with Ash Effect System

Effect laws integrate with existing Ash infrastructure:

- **Capability declarations:** Laws may reference `capability` names from `capability` blocks.
- **Effect lattice:** Laws may use effect-level comparisons (`effect_level(f) <= Act`).
- **Provenance:** Laws may inspect `Provenance` traces for `Operational` effects.
- **Policy blocks:** Laws may verify policy exhaustiveness and role coverage.

### 12.4 Deferred Details

The exact syntax for effect propositions (`requires`, `ensures`, `effect_level`, `Provenance::non_empty`) is not specified here. It depends on:

- Stabilization of the capability syntax (SPEC-002, SPEC-031)
- Stabilization of the provenance trace format (SPEC-047)
- Implementation of effect-level reflection in the typechecker

This section records intent only. A future design note will specify effect-law syntax when the prerequisite features are stable.
