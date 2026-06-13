# TASK-1442: Staleness Checker Automation

## Status: ✅ Complete

## Description

Implement `tools/reference/check_staleness.py` with `--slice` support to automate reference staleness detection. The checker uses git diff against `verified_against.git_commit` to flag pages that need inspection.

## Owner

Phase 144 — Stream B (Staleness Checker)

## Specification References

- `docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA.md`
- `docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md`
- `docs/plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md`
- `docs/plan/PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md`

## Requirements

1. **Git diff scanner**:
   - Read `verified_against.git_commit` from page frontmatter
   - Run `git diff <commit>..HEAD -- <refresh_trigger paths>`
   - Detect if any refresh-triggered file has changed since verification

2. **Frontmatter parser**:
   - Parse YAML frontmatter from `.md` files in `reference/`
   - Extract: `id`, `verified_against.git_commit`, `verified_against.specs`, `verified_against.tasks`, `verified_against.code`, `verified_against.tests`, `refresh_trigger`
   - Handle missing frontmatter gracefully (flag as `needs-frontmatter`)

3. **Refresh-trigger matcher**:
   - Match `refresh_trigger` globs/paths against actual changed files in git diff
   - Support exact paths and glob patterns
   - Report which trigger(s) caused the staleness flag

4. **Output modes**:
   - **Human:** tabular output with page id, status (fresh/stale/needs-inspection), last verified commit, changed files
   - **JSON:** structured array for CI/tooling consumption
   - **Exit codes:** 0 = all fresh, 1 = stale/needs-inspection found, 2 = error

5. **Slice support**:
   - `--slice reference-slice-2`: check only pages with `slice: reference-slice-2` in frontmatter
   - `--slice reference-slice-3`: check only pages with `slice: reference-slice-3` in frontmatter
   - `--all`: check all reference pages (default)

6. **Status semantics**:
   - `fresh`: no refresh-triggered changes since `verified_against.git_commit`
   - `stale`: refresh-triggered files changed, page needs re-verification
   - `needs-inspection`: git diff inconclusive or frontmatter incomplete
   - `needs-frontmatter`: page lacks frontmatter entirely

## Acceptance Criteria

- [x] `python3 tools/reference/check_staleness.py` runs without error on current `reference/` corpus
- [x] All existing Slice 2 pages are correctly classified (fresh/stale/needs-inspection)
- [x] `--slice reference-slice-2` filters to only Slice 2 pages
- [x] `--slice reference-slice-3` filters to only Slice 3 pages (or reports no pages if none exist yet)
- [x] JSON output is valid and includes all required fields
- [x] Exit code 1 when stale pages found (suitable for CI gate)
- [x] Missing frontmatter is flagged, not silently ignored
- [x] `python3 tools/reference/validate.py` still passes (no regression)

## Verification

```bash
# Full corpus check
python3 tools/reference/check_staleness.py

# Slice 2 only
python3 tools/reference/check_staleness.py --slice reference-slice-2

# Slice 3 only (may be empty initially)
python3 tools/reference/check_staleness.py --slice reference-slice-3

# JSON output
python3 tools/reference/check_staleness.py --json

# Validate no regression
python3 tools/reference/validate.py
```

## Out of Scope

- Automatic re-verification or page rewriting
- Semantic staleness detection (content diff, not just metadata)
- Integration with CI/CD pipeline (documented but not wired)
- Webhook or scheduled execution

## Notes

- Use `pyyaml` for frontmatter parsing (check if already in repo)
- Use `subprocess` for git commands
- Keep the script self-contained (no external dependencies beyond stdlib + pyyaml)
- Document the script's limitations in `--help` output

## Dependencies

- Phase 130 (Reference Slice 2) — metadata model established
- Phase 139 (Reference Maintenance) — refresh procedures documented
