# TASK-008: Token Definitions

## Status: 🟢 Complete

## Description

Define the complete set of tokens for the Ash lexer with spans and error types.

## Specification Reference

- SPEC-002: Surface Language - Section 2. Lexical Structure

## Requirements

### Token Categories

1. Keywords: workflow, capability, policy, role, observe, orient, propose, decide, act, oblige, check, let, if, then, else, for, do, par, with, maybe, must, done, etc.
2. Literals: Int, String, Bool, Null
3. Operators: +, -, *, /, =, !=, <, >, <=, >=, and, or, not, in
4. Delimiters: parentheses, braces, brackets, comma, semicolon, colon, dot

### Types to Implement

- `Token` enum with all variants
- `Span` for source locations
- `Spanned<T>` wrapper
- `LexError` for errors

## TDD Steps

1. Write unit tests for token types
2. Implement Token enum
3. Implement Span and Spanned
4. Implement LexError with Display
5. Add property tests

## Completion Checklist

- [ ] All keywords defined
- [ ] All operators defined
- [ ] All delimiters defined
- [ ] Span tracking implemented
- [ ] Error types with Display
- [ ] Unit tests for each token type
- [ ] Property tests for identifiers
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

3 hours

## Dependencies

None
