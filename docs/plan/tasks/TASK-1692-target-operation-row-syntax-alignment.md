# TASK-1692: Align target operation row syntax

**Status:** ✅ Complete
**Owner:** Documentation consistency

## Description

Align target Ash syntax notes and specs so current/legacy capability-authored operations are
written as direct effect operation row items, not as `cap`-prefixed row items. In the target
language, capabilities are subsumed by effects; row syntax names the operation identity
directly, and providers/admission discharge those operation requirements.

## Requirements

1. Replace target surface examples such as `{cap fs.read}` with `{fs.read}`.
2. Replace target alias/group examples such as `{cap fs.read, cap log.write}` with direct
   operation items.
3. Update target taxonomy prose from "capability effect" row items to operation effects where
   the text describes row facts.
4. Preserve out-of-scope current syntax and runtime capability parameter forms such as
   `cap Args`.
5. Clarify that target Ash has effects/operations/providers, not a distinct capability
   language feature.
6. Record the target decision that runtime-provided builtins are ordinary `builtin(...)`
   calls inside trusted stdlib handler implementations only, with `extern fn` out of scope.

## Verification

- `rg -n "\{[^\}\n]*\bcap\b|\bcap\s+(fs|db|log|http|email|llm|tool|approve_transfer)\b|cap items in (effect )?row|row includes cap|inferred row includes cap|specifically cap" docs/notes docs/spec/SPEC-095b-TARGET-GRAMMAR.md docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md docs/spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md docs/spec/SPEC-097-TYPE-SYSTEM-CHANGES.md -S`
  - Pass: no stale target row-syntax matches were found.
- `rg -n 'keep capability|retain capability|domain-specific authoring form|target language has capabilities|Capability effects|capability effects|capability effect' docs/notes/NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md docs/notes/NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md docs/notes/NOTE-018-BOUNDARY-DISCIPLINE.md docs/notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md docs/spec/SPEC-095b-TARGET-GRAMMAR.md docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md docs/spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md docs/spec/SPEC-097-TYPE-SYSTEM-CHANGES.md -S`
  - Pass: no stale target-capability-as-language-feature wording was found.
- `rg -n 'Keep narrow compiler-known escape hatch|Prefer stdlib/effect-owned externs|effect-owned externs|extern placement is split|domain-specific authoring form|keep capability|retain capability|target language has capabilities|Capability effects|capability effects|capability effect' docs/notes/NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md docs/notes/NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md docs/notes/NOTE-018-BOUNDARY-DISCIPLINE.md docs/notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md docs/spec/SPEC-095b-TARGET-GRAMMAR.md docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md docs/spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md docs/spec/SPEC-097-TYPE-SYSTEM-CHANGES.md -S`
  - Pass: no stale builtin-as-top-level-escape-hatch or extern-in-current-target wording was found.
- `git diff --check`
  - Pass: no whitespace errors.
- `rm -f /tmp/task1692-check && git diff --no-index --check /dev/null docs/plan/tasks/TASK-1692-target-operation-row-syntax-alignment.md > /tmp/task1692-check 2>&1 || true; if [ -s /tmp/task1692-check ]; then cat /tmp/task1692-check; exit 1; fi`
  - Pass after removing trailing whitespace from the new untracked task file.

## Completion Notes

- Updated target grammar, target effect-system/type-system specs, duplicated non-`b` target
  specs, and target-oriented notes to use direct operation row items such as `{fs.read}` and
  to describe capabilities as subsumed by effects in the target language.
- Removed stale target-language alternatives that would retain `capability` as sugar or a
  domain-specific authoring form.
- Recorded the agreed builtin boundary: `effect` declarations stay pure operation
  interfaces whose members use ordinary `fn` signatures, `handler` is the preferred surface
  term for operation interpreters while `provider` is a synonym, there is no special target
  `builtin fn` declaration syntax, trusted stdlib handler/provider methods call
  `builtin(symbol, args...)` using a typed runtime primitive symbol/key, and user libraries
  cannot introduce new runtime primitive bindings.
- Left current compatibility syntax and Core/runtime capability text outside the scoped
  target-syntax cleanup.
