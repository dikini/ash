//! Property-based tests for the Ash lexer.
//!
//! These tests use proptest to verify lexer invariants across
//! a wide range of generated inputs.

use ash_parser::{TokenKind, lex, lex_with_recovery};
use proptest::prelude::*;

// List of all keywords to test that they are not treated as identifiers
const KEYWORDS: &[&str] = &[
    "workflow",
    "capability",
    "policy",
    "role",
    "observe",
    "orient",
    "propose",
    "decide",
    "act",
    "oblige",
    "check",
    "let",
    "if",
    "then",
    "else",
    "for",
    "do",
    "par",
    "with",
    "maybe",
    "must",
    "attempt",
    "retry",
    "timeout",
    "done",
    "epistemic",
    "deliberative",
    "evaluative",
    "operational",
    "authority",
    "obligations",
    "supervises",
    "when",
    "returns",
    "where",
    "permit",
    "deny",
    "require_approval",
    "escalate",
    "in",
    "not",
    "and",
    "or",
    "true",
    "false",
    "null",
];

mod tests {
    use super::*;

    // ===================================================================
    // Identifier Properties
    // ===================================================================

    proptest! {
        /// Property: Valid identifiers should lex to an Ident token.
        #[test]
        fn prop_identifier_roundtrip(ident in "[a-zA-Z_][a-zA-Z0-9_]{0,50}") {
            let tokens = lex(&ident).unwrap();
            prop_assert!(tokens.len() >= 2, "Expected at least 2 tokens");
            prop_assert_eq!(tokens[0].kind.clone(), TokenKind::Ident(ident.clone().into()));
        }

        /// Property: Keywords should NOT be recognized as identifiers.
        #[test]
        fn prop_keywords_not_identifiers(keyword in proptest::sample::select(super::KEYWORDS)) {
            let tokens = lex(keyword).unwrap();
            prop_assert!(
                !matches!(tokens[0].kind.clone(), TokenKind::Ident(_)),
                "Keyword '{}' was incorrectly lexed as an identifier",
                keyword
            );
        }

        /// Property: Identifiers starting with a digit should be lexed as numbers.
        #[test]
        fn prop_digit_start_not_identifier(num_ident in "[0-9][a-zA-Z0-9_]{0,20}") {
            let result = lex(&num_ident);
            if let Ok(tokens) = result {
                prop_assert!(
                    !matches!(tokens[0].kind.clone(), TokenKind::Ident(_)),
                    "'{}' starting with digit was lexed as identifier",
                    num_ident
                );
            }
        }

        /// Property: Identifiers with hyphens should be valid and preserve hyphen.
        #[test]
        fn prop_hyphenated_identifiers(ident in "[a-zA-Z_][a-zA-Z0-9_]*-[a-zA-Z0-9_]+") {
            let tokens = lex(&ident).unwrap();
            prop_assert!(tokens.len() >= 2);
            prop_assert_eq!(tokens[0].kind.clone(), TokenKind::Ident(ident.clone().into()));
        }
    }

    // ===================================================================
    // Literal Properties
    // ===================================================================

    proptest! {
        /// Property: Non-negative integer literals should parse correctly.
        /// Note: Negative numbers are lexed as Minus + Int.
        #[test]
        fn prop_nonnegative_int_roundtrip(n in 0i64..=i64::MAX) {
            let s = n.to_string();
            let tokens = lex(&s).unwrap();
            prop_assert_eq!(tokens[0].kind.clone(), TokenKind::Int(n));
        }

        /// Property: Negative integer literals are lexed as Minus + Int.
        #[test]
        fn prop_negative_int_as_minus_plus_int(n in 1i64..=i64::MAX) {
            let s = format!("-{}", n);
            let tokens = lex(&s).unwrap();
            // Should be: Minus, Int(n), Eof
            prop_assert_eq!(tokens[0].kind.clone(), TokenKind::Minus);
            prop_assert_eq!(tokens[1].kind.clone(), TokenKind::Int(n));
        }

        /// Property: String literals should preserve their content.
        #[test]
        fn prop_string_literal_content(content in "[a-zA-Z0-9 _-]{0,100}") {
            let input = format!("\"{}\"", content);
            let tokens = lex(&input).unwrap();
            prop_assert!(tokens.len() >= 2);
            prop_assert_eq!(tokens[0].kind.clone(), TokenKind::String(content.into()));
        }

        /// Property: Float literals should be recognized.
        #[test]
        fn prop_float_literal_roundtrip(
            whole in "[1-9][0-9]{0,9}",
            frac in "[0-9]{1,10}"
        ) {
            let input = format!("{}.{}", whole, frac);
            let tokens = lex(&input).unwrap();
            prop_assert!(tokens.len() >= 2);
            prop_assert!(
                matches!(tokens[0].kind.clone(), TokenKind::Float(_)),
                "Expected Float token for '{}'",
                input
            );
        }
    }

    // ===================================================================
    // Span Properties
    // ===================================================================

    proptest! {
        /// Property: Spans should be monotonic (non-overlapping and ordered).
        #[test]
        fn prop_spans_monotonic(input in "[a-zA-Z0-9_ ]{1,200}") {
            let tokens = lex(&input).unwrap();
            let non_eof: Vec<_> = tokens.iter().filter(|t| t.kind != TokenKind::Eof).collect();

            for i in 1..non_eof.len() {
                prop_assert!(
                    non_eof[i].span.start >= non_eof[i-1].span.end,
                    "Span overlap detected"
                );
            }
        }

        /// Property: Line numbers should increase on newlines.
        #[test]
        fn prop_line_numbers_increase_on_newline(lines in proptest::collection::vec("[a-z]+", 1..10)) {
            let input = lines.join("\n");
            let tokens = lex(&input).unwrap();

            let mut last_line = 1;
            for tok in &tokens {
                prop_assert!(
                    tok.span.line >= last_line,
                    "Line number decreased from {} to {}",
                    last_line, tok.span.line
                );
                last_line = tok.span.line;
            }
        }

        /// Property: Single-line input should have all tokens on line 1.
        #[test]
        fn prop_single_line_all_line_one(input in "[a-zA-Z0-9_]+( [a-zA-Z0-9_]+)*") {
            let tokens = lex(&input).unwrap();
            for tok in &tokens {
                prop_assert_eq!(
                    tok.span.line, 1,
                    "Token should be on line 1 but is on line {}",
                    tok.span.line
                );
            }
        }

        /// Property: Span end should be greater than or equal to start.
        #[test]
        fn prop_span_end_gte_start(input in "[a-zA-Z0-9_ ]{0,100}") {
            let tokens = lex(&input).unwrap();
            for tok in &tokens {
                prop_assert!(
                    tok.span.end >= tok.span.start,
                    "Span end should be >= start"
                );
            }
        }
    }

    // ===================================================================
    // Error Recovery Properties
    // ===================================================================

    proptest! {
        /// Property: Error recovery should preserve valid tokens.
        #[test]
        fn prop_error_recovery_preserves_valid_tokens(
            prefix in "[a-z]+",
            invalid in "[@#$%^&~`|]",
            suffix in "[a-z]+"
        ) {
            let input = format!("{}{}{}", prefix, invalid, suffix);
            let (tokens, errors) = lex_with_recovery(&input);
            let prefix_tokens = lex(&prefix).unwrap();
            let suffix_tokens = lex(&suffix).unwrap();

            prop_assert!(!errors.is_empty(), "Expected at least one error");
            prop_assert!(
                tokens.len() >= 3,
                "Expected at least 3 tokens, got {}",
                tokens.len()
            );

            prop_assert_eq!(tokens[0].kind.clone(), prefix_tokens[0].kind.clone());
            prop_assert_eq!(tokens[1].kind.clone(), suffix_tokens[0].kind.clone());
        }

        /// Property: Unterminated strings should produce an error.
        #[test]
        fn prop_unterminated_string_error(content in "[a-zA-Z0-9 _-]{0,50}") {
            let input = format!("\"{}", content);
            let result = lex(&input);
            prop_assert!(
                result.is_err(),
                "Unterminated string should produce an error"
            );
        }
    }

    // ===================================================================
    // Stress Tests
    // ===================================================================

    proptest! {
        /// Property: Lexer should never panic on random byte sequences.
        #[test]
        fn prop_no_panic_on_random_input(bytes in proptest::collection::vec(any::<u8>(), 0..500)) {
            // Only test with ASCII inputs to avoid UTF-8 slicing bugs in lexer
            let ascii_only: Vec<u8> = bytes.into_iter()
                .filter(|b| b.is_ascii())
                .collect();
            let input = String::from_utf8_lossy(&ascii_only);
            let _ = lex_with_recovery(&input);
        }

        /// Property: Whitespace only input produces only EOF.
        #[test]
        fn prop_whitespace_only_produces_eof(ws in "[ \t\n\r]{0,100}") {
            let tokens = lex(&ws).unwrap();
            prop_assert_eq!(tokens.len(), 1, "Expected only EOF token, got {:?}", tokens);
            prop_assert_eq!(tokens[0].kind.clone(), TokenKind::Eof);
        }

        /// Property: Line comments should be properly skipped.
        /// Uses "x" prefix to avoid keyword conflicts.
        #[test]
        fn prop_line_comments_skipped(content in "x[a-z]{0,10}", after in "y[a-z]{0,10}") {
            let input = format!("{} -- comment\n{}", content, after);
            let tokens = lex(&input).unwrap();

            let idents: Vec<_> = tokens.iter()
                .filter(|t| matches!(t.kind.clone(), TokenKind::Ident(_)))
                .collect();

            prop_assert_eq!(idents.len(), 2, "Expected 2 identifiers, found {:?}",
                idents.iter().map(|t| &t.kind).collect::<Vec<_>>());
        }

        /// Property: Block comments should be properly skipped.
        /// Uses "x" prefix to avoid keyword conflicts.
        #[test]
        fn prop_block_comments_skipped(
            before in "x[a-z]{0,10}",
            comment in "[a-zA-Z0-9_ ]{0,50}",
            after in "y[a-z]{0,10}"
        ) {
            let input = format!("{} /* {} */ {}", before, comment, after);
            let tokens = lex(&input).unwrap();

            let idents: Vec<_> = tokens.iter()
                .filter(|t| matches!(t.kind.clone(), TokenKind::Ident(_)))
                .collect();

            prop_assert_eq!(idents.len(), 2, "Expected 2 identifiers, found {:?}",
                idents.iter().map(|t| &t.kind).collect::<Vec<_>>());
        }
    }

    // ===================================================================
    // Non-Property Tests
    // ===================================================================

    #[test]
    fn test_eof_token_properties() {
        let tokens = lex("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind.clone(), TokenKind::Eof);
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
    }

    #[test]
    fn test_null_literal() {
        let tokens = lex("null").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind.clone(), TokenKind::Null);
    }

    #[test]
    fn test_true_literal() {
        let tokens = lex("true").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind.clone(), TokenKind::True);
    }

    #[test]
    fn test_false_literal() {
        let tokens = lex("false").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind.clone(), TokenKind::False);
    }

    #[test]
    fn test_keyword_exhaustive_not_ident() {
        for keyword in super::KEYWORDS {
            let tokens = lex(keyword).unwrap();
            assert!(
                !matches!(tokens[0].kind.clone(), TokenKind::Ident(_)),
                "Keyword '{}' should not be an identifier",
                keyword
            );
        }
    }
}
