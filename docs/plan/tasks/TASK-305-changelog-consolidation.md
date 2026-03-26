# TASK-305: CHANGELOG.md Consolidation

## Status: 📝 Planned

## Description

Consolidate CHANGELOG.md entries from the 4 Phase 48 worktrees into a single coherent changelog section. Each worktree added its own entries, resulting in potential duplication or inconsistent formatting.

## Worktree CHANGELOG Entries to Consolidate

### 48.1-runtime (TASK-289, 290, 291)
- Engine provider wiring
- Obligation tracking integration
- Expression typing soundness fixes

### 48.2-cli (TASK-292, 293, 294)
- CLI --input improvements
- SPEC-005 compliance (exit codes, error classes)
- REPL workflow storage

### 48.4-types (TASK-295, 296)
- ADT qualified names (:: separator)
- pub(super) visibility fix

### 48.5-repl (TASK-297, 298)
- Multiline error detection
- JSON output schema (SPEC-005)

## Consolidation Requirements

1. **Single Unreleased Section**: Group all Phase 48 changes under `[Unreleased]`
2. **Categorization**: Use Common Changelog categories:
   - `### Added` - New features
   - `### Changed` - Changes to existing functionality
   - `### Fixed` - Bug fixes
   - `### Deprecated` - Soon-to-be removed features
3. **Consistency**: Uniform formatting, task references, and descriptions
4. **Chronological**: Order by completion date or logical grouping

## Proposed Structure

```markdown
## [Unreleased]

### Added

- **Phase 48: Phase 46 Code Review Findings**
  - TASK-289: Engine capability provider wiring to runtime (SPEC-010)
  - TASK-290: Workflow obligation checking in type checker (SPEC-022)
  - TASK-294: REPL workflow definition storage (SPEC-011)
  - TASK-297: REPL multiline error detection improvements (SPEC-011)

### Changed

- **Phase 48: SPEC Compliance**
  - TASK-293: CLI error classes now distinct with proper exit codes per SPEC-021
    - 2 = Parse error, 3 = Type error, 4 = I/O error, 5 = Execution, 6 = Capability
  - TASK-295: ADT names now use `::` separator for qualification (SPEC-003)
  - TASK-298: JSON output schema unified with diagnostics array (SPEC-005)

### Fixed

- **Phase 48: Critical Fixes**
  - TASK-291: Unsound expression typing for unary/binary operators (SPEC-003)
  - TASK-296: `pub(super)` visibility now correctly restricts to parent module (SPEC-009)
```

## Files to Modify

- `CHANGELOG.md` - Consolidate and restructure

## Completion Checklist

- [ ] Review CHANGELOG in all 4 worktrees
- [ ] Consolidate into single Unreleased section
- [ ] Ensure all 10 Phase 48 tasks represented
- [ ] Consistent formatting and task references
- [ ] Proper categorization (Added/Changed/Fixed)
- [ ] No duplicate entries
- [ ] Spell check and proofread
