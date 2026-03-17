# TASK-010: Lexer Property Tests

## Status: 🟢 Complete

## Description

Comprehensive property-based tests for the lexer using proptest to ensure correctness across all possible inputs.

## Specification Reference

- SPEC-002: Surface Language - Section 2. Lexical Structure

## Requirements

### Property Tests to Implement

1. **Identifier Properties**
   - Valid identifiers start with letter/underscore, followed by alphanumerics/underscores/hyphens
   - Keywords are not identifiers
   - Case sensitivity preserved

2. **Literal Properties**
   - Integers parse to exact values
   - Floats parse within epsilon
   - Strings preserve content (with escape handling)
   - Bools parse correctly
   - Null parses correctly

3. **Operator/Delimiter Properties**
   - Each operator produces correct token
   - Multi-character operators parsed as single token
   - Delimiters nest correctly

4. **Comment Properties**
   - Line comments skip to end of line
   - Block comments skip to closing delimiter
   - Nested block comments handled (if supported)

5. **Span Properties**
   - Byte offsets are monotonic
   - Line numbers increment correctly
   - Column resets on newline

6. **Recovery Properties**
   - Invalid characters don't stop lexing
   - Errors have correct spans
   - Multiple errors can be collected

### Fuzzing-Style Tests

- Random character sequences don't panic
- Very long inputs handled correctly
- Unicode handling (even if not fully supported)

## TDD Steps

### Step 1: Create Property Test Module

Create `crates/ash-parser/tests/lexer_props.rs`:

```rust
use proptest::prelude::*;
use ash_parser::lexer::*;

// Strategy for valid identifiers
fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_-]{0,50}"
}

// Strategy for valid strings (simple, no escapes)
fn arb_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{0,100}"
}

// Strategy for integers
fn arb_int() -> impl Strategy<Value = i64> {
    any::<i64>()
}
```

### Step 2: Implement Identifier Properties

```rust
proptest! {
    #[test]
    fn prop_identifier_roundtrip(ident in arb_identifier()) {
        // Check that valid identifiers lex correctly
        let tokens = lex(&ident).unwrap();
        prop_assert_eq!(tokens.len(), 1);
        prop_assert_eq!(tokens[0].kind, TokenKind::Ident(ident.clone().into()));
    }
    
    #[test]
    fn prop_keywords_not_identifiers(
        keyword in prop::sample::select(vec![
            "workflow", "observe", "act", "if", "then", "else"
        ])
    ) {
        let tokens = lex(keyword).unwrap();
        prop_assert!(!matches!(tokens[0].kind, TokenKind::Ident(_)));
    }
}
```

### Step 3: Implement Literal Properties

```rust
proptest! {
    #[test]
    fn prop_int_literal_roundtrip(n in arb_int()) {
        let s = n.to_string();
        let tokens = lex(&s).unwrap();
        prop_assert_eq!(tokens.len(), 1);
        prop_assert_eq!(tokens[0].kind, TokenKind::Int(n));
    }
    
    #[test]
    fn prop_string_literal_content(content in arb_string()) {
        let input = format!("\"{}\"", content);
        let tokens = lex(&input).unwrap();
        prop_assert_eq!(tokens.len(), 1);
        prop_assert_eq!(
            tokens[0].kind, 
            TokenKind::String(content.clone().into())
        );
    }
    
    #[test]
    fn prop_bool_literal_roundtrip(b in prop::bool::ANY) {
        let s = b.to_string();
        let tokens = lex(&s).unwrap();
        prop_assert_eq!(tokens.len(), 1);
        prop_assert_eq!(tokens[0].kind, TokenKind::Bool(b));
    }
}
```

### Step 4: Implement Span Properties

```rust
proptest! {
    #[test]
    fn prop_spans_monotonic(tokens in prop::collection::vec(arb_token(), 1..100)) {
        let input = tokens_to_string(&tokens);
        let lexed = lex(&input).unwrap();
        
        for i in 1..lexed.len() {
            prop_assert!(lexed[i].span.start >= lexed[i-1].span.end);
        }
    }
    
    #[test]
    fn prop_line_numbers_increase_on_newline(
        lines in prop::collection::vec("[a-z]+", 1..10)
    ) {
        let input = lines.join("\n");
        let tokens = lex(&input).unwrap();
        
        let mut last_line = 1;
        for tok in &tokens {
            prop_assert!(tok.span.line >= last_line);
            last_line = tok.span.line;
        }
    }
}
```

### Step 5: Implement Error Recovery Properties

```rust
proptest! {
    #[test]
    fn prop_error_recovery_preserves_valid_tokens(
        prefix in "[a-z]+",
        invalid in "[^a-zA-Z0-9_\s]",
        suffix in "[a-z]+"
    ) {
        let input = format!("{}{}{}", prefix, invalid, suffix);
        let (tokens, errors) = lex_with_recovery(&input);
        
        // Should have error for invalid character
        prop_assert!(!errors.is_empty());
        
        // Should still lex valid parts
        prop_assert!(tokens.len() >= 2);
        prop_assert_eq!(tokens[0].kind, TokenKind::Ident(prefix.into()));
        prop_assert_eq!(tokens[tokens.len()-1].kind, TokenKind::Ident(suffix.into()));
    }
}
```

### Step 6: Implement Stress Tests

```rust
proptest! {
    #[test]
    fn prop_no_panic_on_random_input(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let input = String::from_utf8_lossy(&bytes);
        let _ = lex_with_recovery(&input);  // Should not panic
    }
    
    #[test]
    fn prop_long_input_scales_linearly(
        token in arb_token(),
        count in 100usize..10000
    ) {
        let input = std::iter::repeat(token_to_string(&token))
            .take(count)
            .collect::<Vec<_>>()
            .join(" ");
        
        let start = std::time::Instant::now();
        let tokens = lex(&input).unwrap();
        let elapsed = start.elapsed();
        
        // Should complete in reasonable time (linear scaling)
        prop_assert_eq!(tokens.len(), count);
        prop_assert!(elapsed.as_millis() < count as u128 * 10);  // 10ms per token max
    }
}
```

## Completion Checklist

- [ ] 10+ property tests covering all token types
- [ ] Span property tests (monotonic, line/col tracking)
- [ ] Error recovery property tests
- [ ] Fuzzing-style stress tests
- [ ] Edge case tests (empty input, very long tokens, etc.)
- [ ] All tests pass consistently
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Coverage**: Do properties cover all token types?
2. **Shrinking**: Do test failures shrink to minimal cases?
3. **Performance**: Do tests complete in reasonable time?

## Estimated Effort

4 hours

## Dependencies

- TASK-009: Lexer implementation

## Blocked By

- TASK-009: Lexer implementation

## Blocks

- None (testing task)
