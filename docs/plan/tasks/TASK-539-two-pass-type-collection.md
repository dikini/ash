# TASK-539: Pre-declare type names in TypeEnv

## Status: Draft (v2)

## Description

Add `TypeEnv::declare_type_name()` and modify `Engine::check()` to pre-declare all imported type names before the full registration loop. This fixes sibling type cross-references at the actual failing layer: `TypeEnv::register_type()` → `convert_type_def()` → `resolve_type()`.

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §3
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D1

## Root Cause

The failure occurs in `Engine::check()` (lib.rs:442-451) where imported types are registered one-by-one into `TypeEnv`. When `Message { role: Role }` is registered before `Role`, `resolve_type("Role")` fails because `Role` hasn't been inserted into `ast_types` yet.

The module loader parses types correctly. The fix targets the registration path.

## Requirements

1. `TypeEnv::declare_type_name(name)` inserts into `ast_types` without full conversion.
2. `Engine::check()` pre-declares all imported type names before the register loop.
3. Sibling types register in any order.
4. All 11 SPEC-029 types register without error.

## Completion Checklist

- [ ] `declare_type_name` added to TypeEnv
- [ ] Engine::check pre-declares before register loop
- [ ] Forward reference test passes
- [ ] All 11 SPEC-029 types import without error
- [ ] Existing tests pass

