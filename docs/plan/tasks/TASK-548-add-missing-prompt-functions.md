# TASK-548: Add missing SPEC-029 prompt functions

**Plan Reference:** PLAN-027 (LLM Stdlb Usability Remediation)
**Spec Reference:** SPEC-029 §4.2.1, §4.2.2, §4.2.4, §4.3.2
**Status:** Done
**Depends on:** TASK-546 (record constructor support)

## Description

Add four SPEC-029 prompt functions missing from `prompt.ash` and fix the `has_tool_calls` signature to match the spec.

## Requirements

### New functions

| Function | Spec Section | Signature |
|----------|-------------|-----------|
| `append_response` | §4.2.1 | `(messages: List<Message>, response: ChatResponse) -> List<Message>` |
| `append_tool_result` | §4.2.2 | `(messages: List<Message>, call_id: String, content: String) -> List<Message>` |
| `is_final` | §4.2.4 | `(response: ChatResponse) -> Bool` |
| `render_template` | §4.3.2 | `(template: String, vars: Map<String, String>) -> String` |

### Signature fix

- `has_tool_calls`: changed from `(msg: Message)` to `(response: ChatResponse)` per SPEC-029 §4.2.3

### Re-exports

All four new functions must be re-exported in `mod.ash`.

## TDD Steps

1. Red: Write test asserting each new function is present in prompt.ash exports.
2. Red: Write test asserting `has_tool_calls` parameter type matches SPEC-029.
3. Green: Implement each function in prompt.ash using record constructors.
4. Green: Fix `has_tool_calls` parameter type.
5. Verify: `ash check std/src/llm/prompt.ash` passes. `count_pub_fn_snippets` reaches 27.

## Files

- Modify: `std/src/llm/prompt.ash`
- Modify: `std/src/llm/mod.ash`
- Modify: `crates/ash-engine/tests/llm_stdlib_e2e_tests.rs` (pub fn count assertions)
- Modify: `crates/ash-engine/tests/llm_stdlib_tests.rs`

## Completion Checklist

- [x] `append_response` added with ChatResponse match destructuring
- [x] `append_tool_result` added using existing `tool_result` constructor
- [x] `is_final` added checking finish_reason == "stop" || "length"
- [x] `render_template` added as stub with correct two-parameter signature
- [x] `has_tool_calls` signature fixed: Message -> ChatResponse
- [x] `mod.ash` re-exports updated for all four new functions
- [x] `ChatResponse` added to prompt.ash type imports
- [x] pub fn count assertions updated (23 -> 27 total, 12 -> 15 parseable)
- [x] Full `cargo test --package ash-engine` passes (0 failures)
- [x] CHANGELOG.md updated
- [x] PLAN-INDEX.md updated (TASK-548 -> Done)

## Known Gaps

- `render_template` is a stub (returns template unchanged). Full implementation requires `string::replace` runtime support. Signature uses `Map<String, String>` (alias for `List<(String, String)>` defined in `std/src/map.ash`).
- 12 of 27 pub fns still fail to parse due to parser limitations (closures, nested match, if/then/else). Tracked via `#[ignore]` target test.
- `get_tool_calls` still takes `Message` instead of `ChatResponse` per SPEC-029 §4.2.5. Pre-existing, not TASK-548 scope.
