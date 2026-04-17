# Diagnostic Infrastructure — Follow-Up Notes

## Span::default() Tech Debt (SPEC-039 dependency)

**Date:** 2026-04-17
**Origin:** Phase 85 review (NM-2)
**Blocks:** Phase 87 (LSP) — real spans needed for jump-to-source

### Problem

~50 construction sites in `ash-typeck` pass `Span::default()` instead of real source locations.
These fall into two categories:

1.  **AST span gaps** — `TypeDef`, `InterfaceDef`, `ImplDef` in the surface AST lack `Span`
    fields. All errors emitted during type/interface/impl registration in `type_env.rs`
    (DuplicateType, MissingInterface, OverlappingImpls, etc.) have no span source.
2.  **API span gaps** — Helper functions in `solver.rs`, `check_expr.rs`, `check_pattern.rs`
    do not thread a `Span` parameter through. The `UnifyError → TypeError` conversion in
    `From<UnifyError>` also has no span available.

### Affected files

-   `crates/ash-typeck/src/type_env.rs` (~30 sites)
-   `crates/ash-typeck/src/solver.rs` (~6 sites)
-   `crates/ash-typeck/src/check_expr.rs` (~10 sites)
-   `crates/ash-typeck/src/check_pattern.rs` (~10 sites)

### Resolution path

1.  SPEC-039 (Phase 84, TASK-570/571) resolved `Expr::Variable` and `Pattern::Variable`
    spans. Remaining gaps need a follow-up task to add `Span` to `TypeDef`, `InterfaceDef`,
    `ImplDef`, and other AST nodes.
2.  Thread `Span` through `UnifyError` or provide a context span at unification call sites.
3.  Replace `Span::default()` with real spans incrementally as each AST gap closes.

### Verification

```bash
grep -rn 'Span::default()' crates/ash-typeck/src/ --include='*.rs' | grep -v test | grep -v '#\['
```
