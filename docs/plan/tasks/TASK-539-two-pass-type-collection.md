# TASK-539: Pre-declare type names and upgrade in register_type

## Status: Draft (v3)

## Description

Add `TypeEnv::declare_type_name()` and modify `register_type()` to allow upgrading a placeholder entry. Then modify `Engine::check()` to pre-declare all imported type names before the full registration loop. This fixes sibling type cross-references at the actual failing layer.

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §3 (especially §3.2 placeholder upgrade)
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D1

## Root Cause

The failure occurs in `Engine::check()` (lib.rs:442-451) where imported types are registered one-by-one into `TypeEnv`. When `Message { role: Role }` is registered before `Role`, `resolve_type("Role")` fails because `Role` hasn't been inserted into `ast_types` yet.

Additionally, the planned predeclare-then-register flow requires `register_type()` to accept upgrading a placeholder. The current duplicate-rejection guard (type_env.rs:487-489) rejects ALL entries in `ast_types`, including placeholders. This must be changed.

## Requirements

1. `TypeEnv::declare_type_name(name)` inserts a placeholder into `ast_types` without full conversion.
2. `register_type()` allows replacing a placeholder (but still rejects non-placeholder duplicates).
3. `Engine::check()` pre-declares all imported type names before the register loop.
4. Sibling types register in any order.
5. All 11 SPEC-029 types register without error.

## Completion Checklist

- [ ] `declare_type_name` added to TypeEnv
- [ ] `register_type` modified for placeholder upgrade
- [ ] Engine::check pre-declares before register loop
- [ ] Forward reference test passes
- [ ] All 11 SPEC-029 types import without error
- [ ] Non-placeholder duplicate still errors
- [ ] Existing tests pass
