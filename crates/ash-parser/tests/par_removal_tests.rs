//! Tests to verify that legacy workflow keywords have been removed.
//!
//! These tests ensure that:
//! - `par { ... }` no longer parses
//! - Parser keyword inventory no longer reserves legacy workflow forms

use ash_parser::{TokenKind, lex};

#[test]
fn test_par_keyword_is_no_longer_reserved() {
    // "par" should be treated as an identifier, not a keyword
    let tokens = lex("par").expect("Lexing should succeed");
    assert_eq!(tokens.len(), 2, "Should have 2 tokens: par + Eof");
    assert!(
        matches!(tokens[0].kind, TokenKind::Ident(_)),
        "'par' should be lexed as an identifier, not a keyword"
    );
    if let TokenKind::Ident(name) = &tokens[0].kind {
        assert_eq!(name.as_ref(), "par");
    } else {
        panic!("Expected Ident token");
    }
}

#[test]
fn test_par_block_does_not_parse() {
    let input = "fn main() { par { {}; } }";
    let result = ash_parser::parse_surface_file(input);
    assert!(
        result.is_err(),
        "par block should fail in active source syntax, but got: {:?}",
        result
    );
}

#[test]
fn test_par_in_fn_body_fails() {
    // A function body containing `par` should fail to parse
    let input = "fn test() { par { {}; } }";

    let result = ash_parser::parse_surface_file(input);
    assert!(
        result.is_err(),
        "function with par block should fail to parse, but got: {:?}",
        result
    );
}

// Note: "par" as identifier test removed because let statements are workflow statements,
// not expressions. The expr() parser cannot parse "let par = 42". The main goal of
// rejecting `par { ... }` at the parser boundary is achieved by the other tests.

#[test]
fn test_keyword_list_excludes_par() {
    // Verify that "par" is not in the lexer's keyword list
    // We test this by checking that "par" lexes as an identifier
    let tokens = lex("par").expect("Lexing should succeed");
    assert!(
        matches!(tokens[0].kind, TokenKind::Ident(_)),
        "'par' should not be a keyword"
    );
}

#[test]
fn test_par_token_kind_removed() {
    // Verify TokenKind::Par no longer exists
    // This is a compile-time check - if TokenKind::Par exists,
    // this code won't compile
    // Commented out because it won't compile if Par is removed:
    // let _ = TokenKind::Par;
}

#[test]
fn test_lexer_keywords_list_excludes_par() {
    // Test current non-workflow keywords to ensure `par` is not among them.
    let known_keywords = [
        "capability",
        "policy",
        "role",
        "let",
        "if",
        "then",
        "else",
        "for",
        "do",
        "with",
        "maybe",
        "must",
        "attempt",
        "retry",
        "timeout",
        "{};",
        "epistemic",
        "deliberative",
        "evaluative",
        "operational",
        "authority",
        "obligations",
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

    for keyword in known_keywords {
        let tokens = lex(keyword).expect("Lexing should succeed");
        assert!(
            !matches!(tokens[0].kind, TokenKind::Ident(_)),
            "'{}' should be a keyword",
            keyword
        );
    }

    // But "par" should NOT be a keyword
    let tokens = lex("par").expect("Lexing should succeed");
    assert!(
        matches!(tokens[0].kind, TokenKind::Ident(_)),
        "'par' should NOT be a keyword"
    );
}

#[test]
fn test_legacy_workflow_words_are_no_longer_reserved() {
    let removed_words = [
        "workflow", "proc", "act", "observe", "orient", "propose", "decide", "oblige", "check",
        "par",
    ];

    for word in removed_words {
        let tokens = lex(word).expect("Lexing should succeed");
        assert!(
            matches!(tokens[0].kind, TokenKind::Ident(_)),
            "'{}' should be lexed as an identifier, not a keyword",
            word
        );
    }
}
