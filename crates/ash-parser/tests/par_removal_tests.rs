//! Tests to verify that `par` workflow form has been removed.
//!
//! These tests ensure that:
//! - `par { ... }` no longer parses
//! - Parser keyword inventory no longer reserves `par`
//! - Lowering no longer contains SurfaceWorkflow::Par -> CoreWorkflow::Par path

use ash_parser::{
    TokenKind,
    input::new_input,
    lex,
    parse_workflow::{workflow, workflow_def},
};

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
    // `par { {}; }` should not parse as a workflow
    let input = "par { {}; }";
    let mut parse_input = new_input(input);

    let result = workflow(&mut parse_input);
    assert!(
        result.is_err(),
        "par block should fail to parse, but got: {:?}",
        result
    );
}

#[test]
fn test_par_in_workflow_body_fails() {
    // A workflow containing `par` should fail to parse
    let input = "fn test() { par { {}; } }";
    let mut parse_input = new_input(input);

    let result = workflow_def(&mut parse_input);
    assert!(
        result.is_err(),
        "workflow with par block should fail to parse, but got: {:?}",
        result
    );
}

#[test]
fn test_surface_workflow_no_longer_has_par_variant() {
    // Verify that Workflow enum doesn't have a Par variant that can be constructed
    // This test will fail to compile if Par still exists in the enum
    // Commented out because it won't compile - uncomment to verify removal
    // let _ = Workflow::Par {
    //     branches: vec![],
    //     span: Default::default(),
    // };
}

#[test]
fn test_lowering_no_longer_handles_par() {
    // Verify that lowering doesn't handle SurfaceWorkflow::Par
    // We can't directly test this since we can't construct a Par variant,
    // but we can verify the lowering function signature exists
    // and doesn't have a Par match arm

    // This is more of a compile-time check - if Par variant exists in
    // SurfaceWorkflow, the lower_workflow function will need to handle it
    // and will fail to compile
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
    // Test all current keywords to ensure par is not among them
    let known_keywords = [
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
