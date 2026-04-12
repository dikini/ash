# TASK-528: Create std/src/llm/prompt.ash -- Renderers

## Status: ✅ Complete

## Description

Implement the pure renderer functions that produce string representations: `render_conversation()` and `render_template()`. These are Tier 1 pure `fn` definitions.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D2: Three-Tier Layering)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS4.3: Renderers)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-527](TASK-527-create-prompt-inspectors.md)

## Requirements

1. `fn render_conversation(messages: List<Message>) -> String` -- each message gets a role prefix line.
2. `fn render_template(template: String, vars: Map<String, String>) -> String` -- replaces `{{key}}` placeholders.
3. Postconditions: role prefixes are uppercase, messages in order, deterministic, unresolved placeholders left as-is.

## Guidance

`render_conversation` produces lines like "SYSTEM: content\nUSER: content\n". `render_template` does simple string replacement of `{{key}}` patterns.

## Likely Files

- Modify: `std/src/llm/prompt.ash` (add renderers section)

## TDD Steps

### Red

1. Write test: `render_conversation([system("a"), user("b")])` produces "SYSTEM: a\nUSER: b\n".
2. Write test: empty conversation produces empty string.
3. Write test: determinism: same input produces same output.
4. Write test: `render_template("Hello {{name}}", {"name": "Ash"})` produces "Hello Ash".
5. Write test: missing key leaves placeholder unreplaced.
6. Write test: multiple placeholders all replaced.

### Green

Implement both renderer functions.

## Completion Checklist

- [ ] `render_conversation()` implemented with correct formatting
- [ ] `render_template()` implemented with placeholder replacement
- [ ] All are pure `fn`
- [ ] Tests pass
