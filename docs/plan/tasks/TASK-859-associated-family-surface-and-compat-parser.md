# TASK-859: Associated family surface and compatibility parser

## Status: 🟡 Ready

## Description

Add explicit computation-grade associated family projection syntax while preserving SPEC-035 compatibility parsing for existing `Base::Assoc` forms.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-858 completion

## Files / Ownership

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/parse_type_def.rs` if shared type-parser helpers are factored there
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-parser/src/error.rs` or parser diagnostic helpers if unsupported-shape diagnostics require new variants
- Create/modify tests: `crates/ash-parser/tests/task_859_associated_family_surface.rs`

## Requirements

### Functional Requirements

1. Add parser/surface support for `<Interface<Args...>>::Assoc` in type positions with source spans. Phase 115 accepts source-visible unqualified interface names in the explicit projection head; path-qualified type/interface names are deferred unless the audit proves an existing carrier can support them without ambiguity.
2. Add parser/surface support for typed interface/impl parameters such as `Xs: TypeList`, preserving annotation syntax and spans without semantic validation.
3. Replace or extend the name-only associated member carrier so interface bodies can distinguish ordinary `type Name` from raw `sealed type family Name: ResultDomain [decreases Param]` declarations.
4. Treat `: ResultDomain` as mandatory for sealed associated families in the MVP; omitted domains produce a parser/typecheck diagnostic instead of defaulting silently.
5. Preserve existing `Base::Assoc` and nominal-application compatibility parsing.
6. Keep parser output raw; no interface/member semantic resolution in parser.
7. Add explicit parser diagnostics for unsupported projection and family-declaration shapes.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write failing parser tests

- Add tests in `crates/ash-parser/tests/task_859_associated_family_surface.rs` for `<Iterator<List<A>>>::Item`, nested `<Append<Cons<H, T>, Ys>>::Out`, existing `T::Item`, typed params `Append<Xs: TypeList, Ys: TypeList>`, raw `sealed type family Item: Type`, raw `sealed type family Out: TypeList decreases Xs`, missing result-domain rejection, unsupported qualified projection heads, malformed `<...>::` forms, and malformed declaration forms.

### Step 2: Implement parser/surface changes

- Update surface type carriers only as needed.
- Update parsing/lowering without semantic family lookup.

### Step 3: Verify non-interference

- Re-run associated type, type function, ordinary type, and projection parser suites.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Requirements above are satisfied.
- [ ] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [ ] Negative leakage/non-interference behavior is covered for this task's surface.
- [ ] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [ ] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Completion evidence must be recorded by the implementing agent before marking this task complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - |
    cargo test -p ash-parser --test task_859_associated_family_surface -- --list | tee /tmp/task_859_associated_family_surface-list.txt
    grep -Eq 'associated_family|sealed_type_family|task_859' /tmp/task_859_associated_family_surface-list.txt
  - cargo test -p ash-parser --test task_859_associated_family_surface -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Raw surface syntax and parser tests for explicit family projections plus sealed family declarations.
