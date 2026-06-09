# Phase 136: Interface and Module Law Syntax — Implementation Plan

> **For Hermes:** Use ash-phase-implementation and subagent-driven-development skills to execute this plan task-by-task.

**Goal:** Implement `law` and `proof` syntax for Ash interfaces and modules, plus synthetic test generation and totality checking, across three stages.

**Architecture:** Parser extension for `law`/`proof` keywords → AST storage → Typechecker verification → Synthetic test runner integration → Optional totality checking for hand-written proofs.

**Tech Stack:** Rust (parser in `ash-parser`, typechecker in `ash-typeck`, test runner in `ash-engine` or `ash-cli`)

---

## Phase Overview

| Stage | Focus | Deliverable |
|---|---|---|
| Stage 1 | Parse and store `law`/`proof` syntax | Parser accepts syntax, typechecker verifies names |
| Stage 2 | Synthetic test generation | Runner generates tests from laws, `by test` delegation |
| Stage 3 | Totality checking | Compiler checks proof termination, `Prop` kind promotion |

**Design Note:** [DESIGN-NOTE-INTERFACE-LAWS.md](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

---

## Task Breakdown

### TASK-1360: Parser — `law` keyword in interfaces

**Objective:** Extend `ash-parser` to accept `law` declarations inside interface bodies.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs` — add `LawDef` struct and `laws` field to `InterfaceDef`
- Modify: `crates/ash-parser/src/parse_module.rs` — add `law` parsing inside interface bodies
- Modify: `crates/ash-parser/src/lexer.rs` — add `law` as recognized keyword
- Test: `crates/ash-parser/tests/law_syntax.rs`

**Step 1: Write failing test**

```rust
#[test]
fn parse_law_in_interface() {
    let source = r#"
        pub interface Semigroup<A> {
            append(A, A) -> A
            law associativity(a: A, b: A, c: A, eq: Eq<A>)
              : eq.equiv(append(append(a, b), c), append(a, append(b, c)))
        }
    "#;
    let result = parse_surface_file(source);
    assert!(result.is_ok(), "law syntax should parse: {:?}", result.err());
}
```

**Step 2: Run test to verify failure**

```bash
cargo test -p ash-parser parse_law_in_interface -- --nocapture
```
Expected: FAIL — "unexpected token: law"

**Step 3: Add `LawDef` AST node**

```rust
// In surface.rs
pub struct LawDef {
    pub name: String,
    pub params: Vec<Param>,
    pub constraints: Vec<Constraint>,
    pub proposition: Expr,
}

// LawDef and ProofDef are added as variants to the Definition enum
// See surface.rs: pub enum Definition { ..., Law(LawDef), Proof(ProofDef) }
```

**Step 4: Add parser rule**

```rust
// In grammar module
fn parse_interface_def(...) -> Result<InterfaceDef, ParseError> {
    // ... existing method parsing ...
    let laws = many0(parse_law_def)?;
    // ...
}

fn parse_law_def(...) -> Result<LawDef, ParseError> {
    expect_keyword("law")?;
    let name = parse_identifier()?;
    let params = parse_params()?;
    let constraints = parse_optional_where_clause()?;
    expect_token(Colon)?;
    let proposition = parse_expr()?;
    Ok(LawDef { name, params, constraints, proposition })
}
```

**Step 5: Run test to verify pass**

```bash
cargo test -p ash-parser parse_law_in_interface -- --nocapture
```
Expected: PASS

**Step 6: Commit**

```bash
git add crates/ash-parser/src/surface.rs crates/ash-parser/src/parse_module.rs crates/ash-parser/tests/law_syntax.rs
git commit -m "feat(parser): add law keyword parsing in interfaces (TASK-1360)"
```

**Step 7: Verification**

- [ ] Parser test passes
- [ ] No regressions in existing parser tests: `cargo test -p ash-parser`
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser --all-targets -- -D warnings` clean

---

### TASK-1361: Parser — `law` keyword at module scope

**Objective:** Extend parser to accept `law` declarations at module top level (outside interfaces).

**Files:**
- Modify: `crates/ash-parser/src/surface.rs` — add `Definition::Law(LawDef)` variant to `Definition` enum
- Modify: `crates/ash-parser/src/parse_module.rs` — add module-level `law` parsing in `parse_definitions`
- Modify: `crates/ash-parser/src/lexer.rs` — add `law` as recognized keyword
- Test: `crates/ash-parser/tests/law_module_scope.rs`

**Step 1: Write failing test**

```rust
#[test]
fn parse_module_law() {
    let source = r#"
        law join_preserves_absolute(base: PathBuf, child: String, eq: Eq<PathBuf>)
          : match is_absolute(base) {
              true => is_absolute(join(base, child)),
              false => true
          }
    "#;
    let result = parse_surface_file(source);
    assert!(result.is_ok());
}
```

**Step 2-5:** Add `Definition::Law(LawDef)` variant to `Definition` enum, add parser rule in `parse_definitions`, verify test passes.

**Step 6: Commit**

```bash
git commit -m "feat(parser): add module-scoped law parsing (TASK-1361)"
```

---

### TASK-1362: Parser — `proof` keyword in impl blocks

**Objective:** Extend parser to accept `proof` blocks inside `impl` declarations.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs` — add `ProofDef` struct and `proofs` field to `ImplDef`
- Modify: `crates/ash-parser/src/parse_module.rs` — add `proof` parsing inside impl bodies
- Modify: `crates/ash-parser/src/lexer.rs` — add `proof` and `by_definition` as recognized keywords
- Test: `crates/ash-parser/tests/proof_syntax.rs`

**Step 1: Write failing test**

```rust
#[test]
fn parse_proof_in_impl() {
    let source = r#"
        pub impl Semigroup<String> {
            append(a, b) = string::concat(a, b)
            proof associativity(a, b, c, eq) {
                by_definition
            }
        }
    "#;
    let result = parse_surface_file(source);
    assert!(result.is_ok());
}
```

**Step 3: Add `ProofDef` AST node**

```rust
pub struct ProofDef {
    pub name: String,
    pub params: Vec<Param>,
    pub constraints: Vec<Constraint>,
    pub body: ProofBody,
}

pub enum ProofBody {
    ByDefinition,
    ByTest { config: TestConfig },
    Block { expr: Expr },
}
```

**Step 6: Commit**

```bash
git commit -m "feat(parser): add proof block parsing in impls (TASK-1362)"
```

---

### TASK-1363: Parser — `proof` keyword at module scope

**Objective:** Extend parser to accept module-scoped `proof` blocks.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs` — add `Definition::Proof(ProofDef)` variant to `Definition` enum
- Modify: `crates/ash-parser/src/parse_module.rs` — add module-level `proof` parsing in `parse_definitions`
- Modify: `crates/ash-parser/src/lexer.rs` — add `proof` as recognized keyword
- Test: `crates/ash-parser/tests/proof_module_scope.rs`

**Step 6: Commit**

```bash
git commit -m "feat(parser): add module-scoped proof parsing (TASK-1363)"
```

---

### TASK-1364: Typechecker — verify law proposition names exist

**Objective:** Typechecker verifies that all names referenced in a law proposition exist and are well-typed.

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs` — add law registration
- Modify: `crates/ash-typeck/src/check.rs` — add law proposition checking
- Test: `crates/ash-typeck/tests/law_typecheck.rs`

**Step 1: Write failing test**

```rust
#[test]
fn law_references_unknown_function_is_error() {
    let mut env = TypeEnv::new();
    let module = parse(r#"
        law bad_law(x: Int)
          : unknown_function(x)
    "#);
    let err = env.register_module_laws(&module).unwrap_err();
    assert!(err.to_string().contains("unknown_function"));
}
```

**Step 3: Add law registration**

```rust
impl TypeEnv {
    pub fn register_interface_laws(&mut self, interface: &InterfaceDef) -> Result<(), TypeError> {
        for law in &interface.laws {
            // Verify all names in proposition exist in scope
            self.check_expr(&law.proposition)?;
        }
        Ok(())
    }
    
    pub fn register_module_laws(&mut self, module: &ModuleFile) -> Result<(), TypeError> {
        for law in &module.laws {
            self.check_expr(&law.proposition)?;
        }
        Ok(())
    }
}
```

**Step 6: Commit**

```bash
git commit -m "feat(typeck): verify law proposition names exist (TASK-1364)"
```

---

### TASK-1365: Typechecker — verify proof names match declared laws

**Objective:** Compiler rejects `proof unknown_law(...) { ... }` if no matching law exists.

**Files:**
- Modify: `crates/ash-typeck/src/check.rs`
- Test: `crates/ash-typeck/tests/proof_typecheck.rs`

**Step 1: Write failing test**

```rust
#[test]
fn proof_for_unknown_law_is_error() {
    let mut env = TypeEnv::new();
    let module = parse(r#"
        law real_law(x: Int) : true
        
        proof fake_law(x) {
            by_definition
        }
    "#);
    let err = env.register_module_proofs(&module).unwrap_err();
    assert!(err.to_string().contains("fake_law"));
    assert!(err.to_string().contains("no matching law"));
}
```

**Step 6: Commit**

```bash
git commit -m "feat(typeck): verify proof names match declared laws (TASK-1365)"
```

---

### TASK-1366: Typechecker — restrict law propositions to Pure functions

**Objective:** Law propositions must reference only `Pure` functions. `Act`/`Proc`/`Workflow` in law body = compile-time error.

**Files:**
- Modify: `crates/ash-typeck/src/check.rs` — add effect level checking for law propositions
- Test: `crates/ash-typeck/tests/law_purity.rs`

**Step 1: Write failing test**

```rust
#[test]
fn law_with_act_function_is_error() {
    let mut env = TypeEnv::with_builtin_types();
    let module = parse(r#"
        law bad_law(ma: Act<Int>)
          : bind(ma, |x| -> unit(x))
    "#);
    let err = env.register_module_laws(&module).unwrap_err();
    assert!(err.to_string().contains("Pure"));
    assert!(err.to_string().contains("Act"));
}
```

**Step 6: Commit**

```bash
git commit -m "feat(typeck): restrict law propositions to Pure functions (TASK-1366)"
```

---

### TASK-1367: Typechecker — proof body totality check (Stage 3 prep)

**Objective:** Add infrastructure for totality checking. Stage 3 will fill in the actual checks.

**Files:**
- Modify: `crates/ash-typeck/src/check.rs` — add `check_proof_totality` stub
- Test: `crates/ash-typeck/tests/proof_totality.rs`

**Step 1: Write test (stub passes)**

```rust
#[test]
fn proof_totality_stub_accepts_by_definition() {
    let mut env = TypeEnv::new();
    let proof = parse_proof(r#"
        proof associativity(a, b, c, eq) {
            by_definition
        }
    "#);
    // Stage 3: this will actually check totality
    // Stage 1: stub accepts all proof bodies
    env.check_proof_totality(&proof).expect("stub accepts all");
}
```

**Step 6: Commit**

```bash
git commit -m "feat(typeck): add proof totality check stub (TASK-1367)"
```

---

### TASK-1368: Synthetic tests — extract law nodes from AST

**Objective:** Test runner can iterate over `law` declarations in parsed modules.

**Files:**
- Modify: `crates/ash-cli/src/test_runner/mod.rs` or `crates/ash-cli/src/test.rs`
- Test: `crates/ash-engine/tests/law_extraction.rs`

**Step 1: Write failing test**

```rust
#[test]
fn extract_laws_from_module() {
    let module = parse(r#"
        law associativity(a: Int, b: Int, c: Int, eq: Eq<Int>)
          : eq.equiv(add(add(a, b), c), add(a, add(b, c)))
    "#);
    let laws = extract_laws(&module);
    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].name, "associativity");
}
```

**Step 6: Commit**

```bash
git commit -m "feat(test-runner): extract law nodes from AST (TASK-1368)"
```

---

### TASK-1369: Synthetic tests — generate small-world tests from laws

**Objective:** For each law without a `proof` block, generate small-world tests using the SPEC-077 runner framework.

**Files:**
- Modify: `crates/ash-cli/src/test_runner/mod.rs`
- Test: `crates/ash-engine/tests/law_synthetic_tests.rs`

**Step 1: Write failing test**

```rust
#[test]
fn generate_synthetic_test_for_unproven_law() {
    let module = parse(r#"
        law always_true(x: Int) : x == x
    "#);
    let tests = generate_law_tests(&module);
    assert_eq!(tests.len(), 1);
    // Test should pass for all small-world values
    let result = run_synthetic_test(&tests[0]);
    assert!(result.passed);
}
```

**Step 6: Commit**

```bash
git commit -m "feat(test-runner): generate synthetic tests from laws (TASK-1369)"
```

---

### TASK-1370: Synthetic tests — `by test` delegation syntax

**Objective:** Support `proof ... { by test "test_name" }` syntax for explicit synthetic test delegation.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs` — `ProofBody::ByTest`
- Modify: `crates/ash-cli/src/test_runner/mod.rs`
- Test: `crates/ash-engine/tests/by_test_delegation.rs`

**Step 6: Commit**

```bash
git commit -m "feat(test-runner): support by test delegation syntax (TASK-1370)"
```

---

### TASK-1371: CLI — `--skip-law-tests` and `--skip-law-test=<name>`

**Objective:** Add opt-out flags for law testing.

**Files:**
- Modify: `crates/ash-cli/src/commands/test.rs`
- Modify: `crates/ash-cli/src/test_runner/executor.rs`
- Test: `crates/ash-cli/tests/test_command.rs`

**Step 6: Commit**

```bash
git commit -m "feat(cli): add --skip-law-tests and --skip-law-test=<name> support (TASK-1371)"
```

---

### TASK-1372: Cache — `.ash/law-cache.toml` implementation

**Objective:** Implement dedicated law test result cache separate from `ash.lock`.

**Files:**
- Create: `crates/ash-engine/src/law_cache.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Test: `crates/ash-engine/tests/law_cache.rs`

**Step 6: Commit**

```bash
git commit -m "feat(cache): implement .ash/law-cache.toml (TASK-1372)"
```

---

### TASK-1373: Integration — end-to-end law syntax in std::algebra

**Objective:** Add `law` declarations to at least one `std::algebra` interface and verify full pipeline works.

**Files:**
- Modify: `std/src/algebra/semigroup.ash`
- Modify: `std/src/algebra/monoid.ash`
- Test: `crates/ash-engine/tests/task_1021_std_algebra_namespace_and_interfaces.rs`
- Test: `crates/ash-cli/src/test_runner/synthesized.rs`

**Step 1: Add law to Semigroup**

```ash
pub interface Semigroup<A> {
    append(A, A) -> A

    law associativity(a: A, b: A, c: A, eq: Eq<A>)
      : eq.equiv(append(append(a, b), c), append(a, append(b, c)))
}
```

**Status:** Complete. `Semigroup` now declares `associativity`, `Monoid` declares `left_identity` and `right_identity`, and real stdlib parse/check plus runner extraction coverage is in place.

**Step 6: Commit**

```bash
git commit -m "feat(stdlib): add law declarations to std::algebra (TASK-1373)"
```

---

### TASK-1374: Integration — module-scoped law in std::io::path

**Objective:** Add module-scoped `law` to `std::io::path` and verify.

**Files:**
- Modify: `std/src/io/path.ash`
- Test: `crates/ash-parser/tests/stdlib_parsing.rs`
- Test: `crates/ash-engine/tests/task_1374_stdlib_path_law.rs`
- Test: `crates/ash-cli/tests/test_command.rs`

**Status:** Complete. `std::io::path` now declares module law `join_preserves_absolute` over pure helper `preserves_absolute_after_join`, with parser, engine check, CLI `ash check`, and synthesized law-row coverage against the real stdlib file.

**Step 6: Commit**

```bash
git commit -m "feat(stdlib): add module-scoped law to std::io::path (TASK-1374)"
```

---

### TASK-1375: Stage 3 — proof totality checking

**Objective:** Implement actual totality checking for proof bodies.

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-cli/src/commands/check.rs`
- Test: `crates/ash-typeck/tests/task_1375a_proof_fuel.rs`
- Test: `crates/ash-cli/tests/test_command.rs`

**Status:** In progress via split subtasks. TASK-1375a is complete: proof expression bodies now consume configurable traversal fuel, default proof fuel is 1000, the direct proof checker returns an `Untested(FuelExhausted)` result rather than a type error on fuel exhaustion, and `ash check --proof-fuel <N>` threads the configured budget into program typechecking. Program registration currently treats untested proof checks as non-errors and does not persist or report them through the CLI. TASK-1375b is complete: AST-level proof-body `Expr::Match` nodes are checked with the conservative match coverage engine, rejecting missing constructors unless a `_` catch-all or complete coverage is present. TASK-1375c still owns circular-proof detection.

**Step 1: Write failing test**

```rust
#[test]
fn non_total_proof_is_rejected() {
    let mut env = TypeEnv::new();
    let proof = parse_proof(r#"
        proof bad_proof(n: Int) {
            -- Non-total: infinite recursion on negative numbers
            if n < 0 { bad_proof(n - 1) } else { by_definition }
        }
    "#);
    let err = env.check_proof_totality(&proof).unwrap_err();
    assert!(err.to_string().contains("non-total"));
}
```

**Step 6: Commit**

```bash
git commit -m "feat(typeck): implement proof totality checking (TASK-1375)"
```

---

### TASK-1376: Stage 3 — `Prop` kind promotion

**Objective:** Promote `Prop` from convention to distinct kind.

**Files:**
- Modify: `crates/ash-typeck/src/kind.rs`
- Modify: `crates/ash-typeck/src/check.rs`
- Test: `crates/ash-typeck/tests/prop_kind.rs`

**Step 6: Commit**

```bash
git commit -m "feat(typeck): promote Prop to distinct kind (TASK-1376)"
```

---

### TASK-1377: Closeout — docs, status, CHANGELOG

**Objective:** Update all status surfaces, write docs, verify full gates.

**Files:**
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Create: `docs/plan/tasks/TASK-1360.md` through `TASK-1376.md`
- Modify: `docs/design/DESIGN-NOTE-INTERFACE-LAWS.md` — mark Stage 1/2/3 complete

**Step 1: Create task files**

For each TASK-1360 through TASK-1376, create a task file in `docs/plan/tasks/`.

**Step 2: Update PLAN-INDEX**

Add Phase 136 row with all tasks.

**Step 3: Update CHANGELOG**

```markdown
### Added
- [Phase 136](docs/plan/PLAN-136-INTERFACE-LAW-SYNTAX.md): Implemented `law` and `proof` syntax for interfaces and modules (TASK-1360 through TASK-1376).
  - Parser accepts `law` declarations in interfaces and at module scope.
  - Parser accepts `proof` blocks in impls and at module scope.
  - Typechecker verifies law proposition names and restricts to Pure functions.
  - Synthetic test generation from unproven laws.
  - `by test` delegation for tower carrier laws.
  - `--skip-law-tests` and `--skip-law-test=<name>` opt-out.
  - `.ash/law-cache.toml` for test result caching.
  - Proof totality checking (Stage 3).
  - `Prop` promoted to distinct kind (Stage 3).
```

**Step 4: Run full gates**

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
```

**Step 5: Commit**

```bash
git add docs/plan/tasks/TASK-136*.md docs/plan/tasks/TASK-137*.md docs/plan/PLAN-INDEX.md CHANGELOG.md
git commit -m "docs: Phase 136 closeout — law syntax implementation (TASK-1377)"
```

---

## Verification Commands

Run before marking phase complete:

```bash
# Parser tests
cargo test -p ash-parser

# Typechecker tests
cargo test -p ash-typeck

# Engine/CLI tests
cargo test -p ash-engine
cargo test -p ash-cli

# Full workspace
cargo test --workspace

# Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format
cargo fmt --check

# Docs
cargo doc --workspace --no-deps
```

---

## Task Dependencies

```
TASK-1359 (prerequisite: Eq interface in std::algebra)
    |
    v
TASK-1360 (parser: law in interfaces)
    |
    v
TASK-1361 (parser: module law)
    |
    v
TASK-1362 (parser: proof in impls)
    |
    v
TASK-1363 (parser: module proof)
    |
    v
TASK-1364 (typeck: law name checking)
    |
    v
TASK-1365 (typeck: proof name checking)
    |
    v
TASK-1366 (typeck: law purity restriction)
    |
    v
TASK-1367 (typeck: totality stub)
    |
    v
TASK-1368 (runner: law extraction)
    |
    v
TASK-1369 (runner: synthetic test generation)
    |
    v
TASK-1370 (runner: by test delegation)
    |
    v
TASK-1371 (CLI: opt-out flags)
    |
    v
TASK-1372 (cache: law-cache.toml)
    |
    v
TASK-1373 (integration: std::algebra laws)
    |
    v
TASK-1374 (integration: module law example)
    |
    v
TASK-1375 (Stage 3: totality checking)
    |
    v
TASK-1376 (Stage 3: Prop kind)
    |
    v
TASK-1377 (closeout)
```

---

## Risk Mitigation

| Risk | Mitigation |
|---|---|
| Parser changes break existing syntax | Extensive regression tests on all `std/src/algebra/*.ash` files |
| Typechecker performance degradation | Law checking is opt-in per-module; no overhead for modules without laws |
| Synthetic tests are slow | `--skip-law-tests` flag; caching in `.ash/law-cache.toml` |
| Proof totality checking is undecidable | Fuel-based approach with configurable limits; timeout = `untested` |

---

## Acceptance Criteria

- [ ] All 19 tasks implemented and committed (TASK-1359 through TASK-1377)
- [ ] Parser accepts all law/proof syntax without regressions
- [ ] Typechecker verifies names, purity, and proof-law matching
- [ ] Synthetic tests generate and execute for unproven laws
- [ ] At least one `std::algebra` interface has live law declarations
- [ ] At least one module has live module-scoped law declaration
- [ ] Full workspace gates pass
- [ ] CHANGELOG.md updated
- [ ] PLAN-INDEX.md updated
- [ ] Task files created for all tasks
