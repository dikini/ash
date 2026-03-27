# TASK-330: Documentation Consistency Audit

## Status: 🟡 Medium

## Problem

Documentation may contain inconsistencies with implementation after Phase 52 changes.

## Audit Checklist

### SPEC-005-CLI.md
- [ ] `--capability` flag documented as removed (not just missing)
- [ ] `--input` flag documented as removed (not just missing)
- [ ] No examples use removed flags
- [ ] Capability providers section is clear
- [ ] Input parameters section explains workaround

### SPEC-010-EMBEDDING.md
- [ ] HTTP section documents unimplemented status
- [ ] `with_http_capabilities()` documents error return
- [ ] No examples imply HTTP is available
- [ ] Custom provider workaround documented

### CHANGELOG.md
- [ ] Phase 52 tasks documented
- [ ] Breaking changes clearly marked
- [ ] Superseded tasks noted
- [ ] Task references correct

### Example Files
- [ ] All examples use `capabilities:` syntax (related to TASK-328)
- [ ] Examples don't use removed CLI flags

## Verification

```bash
# Check for removed flags in docs
grep -r "\-\-capability" docs/ --include="*.md"
grep -r "\-\-input" docs/ --include="*.md"
# Expected: Only in context of "removed" or "not supported"
```

## Completion Checklist

- [ ] SPEC-005 audited and consistent
- [ ] SPEC-010 audited and consistent
- [ ] CHANGELOG complete and accurate
- [ ] Any inconsistencies fixed
- [ ] Changes committed

**Estimated Hours:** 2
**Priority:** Medium (documentation quality)
