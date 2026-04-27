    # TASK-729: Add type-checker environments for capability interface operation signatures.

    ## Status: 📝 Planned

    ## Task Type

    Type System

    ## Description

    Add type-checker environments for capability interface operation signatures.

    ## Specification Reference

    - SPEC-052
- SPEC-003

    ## Dependencies

    - ✅ TASK-727: prerequisite task
- ✅ TASK-728: prerequisite task

    ## Requirements

    ### Functional Requirements

    1. Register visible capability interfaces in TypeEnv or a dedicated adjacent environment.
2. Validate operation parameter and return types.
3. Reject duplicate or unsupported operation definitions.
4. Expose lookup APIs for capability call and implementation conformance checking.

    ### Property Requirements (proptest)

    ```rust
    // Add property-based tests for parser round-trips, conformance invariants,
    // authority non-widening, resource identity preservation, or split/join
    // behavior where this task introduces executable semantics.
    // Docs-only tasks must instead include a corpus consistency sweep.
    ```

    ## TDD Steps

    ### Step 1: Write Tests or Corpus Checks (Red)

    For implementation tasks, add failing tests before code changes. For docs/planning tasks, add or run corpus checks that fail or report missing references before updating docs.

    ### Step 2: Implement or Write Docs (Green)

    Make the minimal focused change required by this task while preserving the Ash semantic tower:

    ```text
    Pure < Effectful / Act < Proc < Workflow
    ```

    ### Step 3: Integration (Green)

    Wire only the layer owned by this task. Do not silently implement downstream runtime behavior from later tasks.

    ### Step 4: Verification

    Required verification for this task class:

    - Parser/type/runtime tasks: focused crate tests plus affected integration tests.
    - Docs/planning tasks: `git diff --check` plus cross-reference sweep for changed docs.
    - All code tasks: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` before completion.

    ## Verification Steps

    - [ ] Requirements above are satisfied.
    - [ ] New tests or docs checks cover the task-owned behavior.
    - [ ] Existing public behavior remains compatible unless the spec explicitly says otherwise.
    - [ ] CHANGELOG.md is updated for implementation/tooling/docs-policy changes.
    - [ ] PLAN-INDEX.md status is updated only when the task is actually complete.

    ## Dependencies for Next Task

    This task outputs:

    - Type checking can resolve interface operation signatures.

    ## Notes

    - Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
    - Do not allow ambient capability/resource lookup to bypass explicit admission.
    - Do not manufacture external authority from Ash-defined code.
    - Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
