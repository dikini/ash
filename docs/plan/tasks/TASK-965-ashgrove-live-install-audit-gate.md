# TASK-965: Ashgrove live install audit gate

## Status: ✅ Complete

## Description

Audit live CLI, build, release, stdlib, daemon, XDG, metadata, tarball, and git seams before implementation and bind downstream tasks to exact files/tests.

## Specification Reference

- SPEC-073 §5-§17
- PLAN-122 §6

## Dependencies

- TASK-964 completion.

## Requirements

### Functional Requirements

1. Create `docs/plan/audits/TASK-965-ashgrove-live-install-audit-gate.md`.
2. Decide the `ashgrove` implementation home, with `crates/ashgrove` as the preferred first-slice default unless audit evidence selects another path.
3. Map source install, binary tarball production/install, update, cleanup, remove, lock, fetch, and vendor implementation seams to exact files.
4. Freeze the exact public standard-tool list; daemon control remains `ash daemon ...` unless a compatibility `ashd` shim is explicitly added.
5. Audit current stdlib discovery and bind the installed-stdlib-root refactor to exact `ash-engine`/`ash-cli` files and tests.
6. Audit daemon state/control-plane changes needed for live toolchain removal protection.
7. Select TOML/XDG/git/tar/HTTP-download/SemVer dependencies or command prerequisites and record rationale.
8. Choose the first-slice toolchain-id scheme and deterministic same-version collision behavior.
9. Replace fail-closed placeholder verification in implementation tasks TASK-966 through TASK-973 with focused non-zero commands.

### Non-goals

- Do not implement behavior in this audit task.
- Do not proceed from guessed file paths.
- Do not mark downstream implementation tasks ready while placeholder verification remains.

## Work Steps

1. Inspect the exact live files named by the task or audit output.
2. Write focused RED tests or docs assertions before changing behavior.
3. Implement or document the minimal target behavior.
4. Run focused verification.
5. Update status surfaces and `CHANGELOG.md` if files beyond tests are changed.
6. Request independent review before marking complete.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - test -f docs/plan/audits/TASK-965-ashgrove-live-install-audit-gate.md
  - python3 - <<'PY'
from pathlib import Path
for n in range(966, 974):
    p = next(Path('docs/plan/tasks').glob(f'TASK-{n}-*.md'))
    text = p.read_text()
    assert 'false # TASK-965' not in text, str(p)
    assert 'placeholder' not in text.lower(), str(p)
    assert 'cargo test' in text or 'scripts/check-' in text or 'python3 -' in text, str(p)
PY
checklist:
  - [x] Create `docs/plan/audits/TASK-965-ashgrove-live-install-audit-gate.md`.
  - [x] Decide the `ashgrove` implementation home, with `crates/ashgrove` as the preferred first-slice default unless audit evidence selects another path.
  - [x] Map source install, binary tarball production/install, update, cleanup, remove, lock, fetch, and vendor implementation seams to exact files.
  - [x] Freeze the exact public standard-tool list; daemon control remains `ash daemon ...` unless a compatibility `ashd` shim is explicitly added.
  - [x] Audit current stdlib discovery and bind the installed-stdlib-root refactor to exact `ash-engine`/`ash-cli` files and tests.
  - [x] Audit daemon state/control-plane changes needed for live toolchain removal protection.
  - [x] Select TOML/XDG/git/tar/HTTP-download/SemVer dependencies or command prerequisites and record rationale.
  - [x] Choose the first-slice toolchain-id scheme and deterministic same-version collision behavior.
  - [x] Replace fail-closed placeholder verification in implementation tasks TASK-966 through TASK-973 with focused non-zero commands.
```


## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Dependencies for Next Task

This task contributes to PLAN-122 and SPEC-073 completion. Later tasks must preserve the alpha rules that toolchains are immutable, stdlib is bundled with the selected toolchain, lower-case `ash.toml` is the project manifest, and git dependencies resolve to exact commits in `ash.lock`.


## Notes

Area: audit/substrate. This is the hard gate that converts the packet from policy-level to implementation-bound handoff.
