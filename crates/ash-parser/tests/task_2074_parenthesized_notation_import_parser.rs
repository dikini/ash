//! TASK-2074 RED evidence for exact parenthesized notation-import syntax.

use ash_parser::surface::Visibility;
use ash_parser::token::Span;
use ash_parser::use_tree::{SimplePath, Use, UsePath};
use ash_parser::{NotationImportSelector, NotationPatternPart};

fn parse_use_complete(source: &str) -> Use {
    let mut input = ash_parser::input::new_input(source);
    let parsed = ash_parser::parse_use::parse_use(&mut input)
        .expect("the complete use declaration should parse");
    assert!(
        input.input.is_empty(),
        "the use parser left unconsumed source: {:?}",
        input.input
    );
    parsed
}

fn rejects_use(source: &str) {
    let mut input = ash_parser::input::new_input(source);
    assert!(
        ash_parser::parse_use::parse_use(&mut input).is_err(),
        "the unsupported use declaration unexpectedly parsed: {source}"
    );
}

fn span_for(source: &str, start: usize, end: usize) -> Span {
    let line = source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let column = source[..start]
        .rfind('\n')
        .map_or(start + 1, |newline| start - newline);
    Span::new(start, end, line, column)
}

fn notation_path(use_declaration: &Use) -> (&SimplePath, &NotationImportSelector) {
    let UsePath::Notation { module, selector } = &use_declaration.path else {
        panic!(
            "expected a notation-import path, got {:?}",
            use_declaration.path
        )
    };
    (module, selector)
}

fn assert_hole(part: &NotationPatternPart, span: Span) {
    assert_eq!(part, &NotationPatternPart::Hole { span });
}

fn assert_token(part: &NotationPatternPart, spelling: &str, span: Span) {
    assert_eq!(
        part,
        &NotationPatternPart::Token {
            spelling: spelling.into(),
            span,
        }
    );
}

#[test]
fn parses_symbolic_parenthesized_notation_selector_with_exact_ast_and_spans() {
    let source = "use crate::math::(<*>);";
    let parsed = parse_use_complete(source);

    assert_eq!(parsed.visibility, Visibility::Inherited);
    assert_eq!(parsed.alias, None);
    assert_eq!(parsed.span, Span::new(0, source.len(), 1, 1));

    let (module, selector) = notation_path(&parsed);
    assert_eq!(module.segments, vec!["crate".into(), "math".into()]);
    assert_eq!(selector.span, Span::new(18, 21, 1, 19));
    let [part] = selector.parts.as_ref() else {
        panic!("expected one normalized selector part")
    };
    assert_token(part, "<*>", Span::new(18, 21, 1, 19));
}

#[test]
fn parses_mixfix_selector_with_ordered_holes_tokens_and_exact_spans() {
    let source = "use crate::ranges::(_ between _ and _);";
    let parsed = parse_use_complete(source);
    let (module, selector) = notation_path(&parsed);

    assert_eq!(module.segments, vec!["crate".into(), "ranges".into()]);
    assert_eq!(selector.span, Span::new(20, 37, 1, 21));
    let [first, between, second, and, third] = selector.parts.as_ref() else {
        panic!("expected the five ordered mixfix selector parts")
    };
    assert_hole(first, Span::new(20, 21, 1, 21));
    assert_token(between, "between", Span::new(22, 29, 1, 23));
    assert_hole(second, Span::new(30, 31, 1, 31));
    assert_token(and, "and", Span::new(32, 35, 1, 33));
    assert_hole(third, Span::new(36, 37, 1, 37));
}

#[test]
fn selector_comments_and_whitespace_normalize_away_without_losing_diagnostic_source() {
    let source = "use m::(  _ /* gap */ between // bridge\n  _  );";
    let parsed = parse_use_complete(source);
    let (_, selector) = notation_path(&parsed);

    let first_start = source.find('_').expect("first hole is present");
    let between_start = source.find("between").expect("word token is present");
    let second_start = source.rfind('_').expect("second hole is present");
    assert_eq!(
        selector.span,
        span_for(source, first_start, second_start + 1)
    );
    let [first, between, second] = selector.parts.as_ref() else {
        panic!("comments and whitespace must not become semantic parts")
    };
    assert_hole(first, span_for(source, first_start, first_start + 1));
    assert_token(
        between,
        "between",
        span_for(source, between_start, between_start + "between".len()),
    );
    assert_hole(second, span_for(source, second_start, second_start + 1));
}

#[test]
fn only_a_bare_underscore_is_a_hole() {
    let source = "use m::(_ _name __);";
    let parsed = parse_use_complete(source);
    let (_, selector) = notation_path(&parsed);
    let [bare, underscore_name, double_underscore] = selector.parts.as_ref() else {
        panic!("expected three normalized selector parts")
    };

    assert_hole(bare, span_for(source, 8, 9));
    assert_token(underscore_name, "_name", span_for(source, 10, 15));
    assert_token(double_underscore, "__", span_for(source, 16, 18));
}

#[test]
fn as_is_an_ordinary_word_inside_a_mixfix_selector() {
    let source = "use m::(_ as _);";
    let parsed = parse_use_complete(source);
    let (_, selector) = notation_path(&parsed);
    let [left, as_token, right] = selector.parts.as_ref() else {
        panic!("expected hole, `as` token, hole")
    };

    assert_hole(left, span_for(source, 8, 9));
    assert_token(as_token, "as", span_for(source, 10, 12));
    assert_hole(right, span_for(source, 13, 14));
}

#[test]
fn parenthesized_star_is_an_exact_operator_selector_not_a_glob() {
    let source = "use m::(*);";
    let parsed = parse_use_complete(source);
    let (_, selector) = notation_path(&parsed);
    let [star] = selector.parts.as_ref() else {
        panic!("expected one exact operator token")
    };
    assert_token(star, "*", span_for(source, 8, 9));
}

#[test]
fn empty_or_unclosed_notation_selectors_reject() {
    rejects_use("use m::();");
    rejects_use("use m::(_ between _;");
}

#[test]
fn notation_selectors_reject_invalid_comma_separators() {
    rejects_use("use m::(, _);");
    rejects_use("use m::(_,,_);");
    rejects_use("use m::(_ between _,);");
}

#[test]
fn trailing_whole_import_alias_rejects_but_does_not_reserve_as_inside_selector() {
    rejects_use("use m::(<*>) as apply;");
    rejects_use("use m::(_ as _) as cast;");

    let parsed = parse_use_complete("use m::(_ as _);");
    assert!(matches!(parsed.path, UsePath::Notation { .. }));
}

#[test]
fn every_visible_notation_use_rejects_until_reexport_has_a_contract() {
    for source in [
        "pub use m::(<*>);",
        "pub(crate) use m::(<*>);",
        "pub(super) use m::(<*>);",
        "pub(self) use m::(<*>);",
        "pub(in crate::syntax) use m::(<*>);",
    ] {
        rejects_use(source);
    }
}

#[test]
fn ordinary_simple_and_whole_alias_imports_remain_unchanged() {
    let simple = parse_use_complete("use crate::math::value;");
    assert_eq!(simple.visibility, Visibility::Inherited);
    assert_eq!(simple.alias, None);
    assert_eq!(
        simple.path,
        UsePath::Simple(SimplePath {
            segments: vec!["crate".into(), "math".into(), "value".into()],
        })
    );

    let aliased = parse_use_complete("use crate::math::value as local;");
    assert_eq!(aliased.alias.as_deref(), Some("local"));
    assert_eq!(
        aliased.path,
        UsePath::Simple(SimplePath {
            segments: vec!["crate".into(), "math".into(), "value".into()],
        })
    );
}

#[test]
fn ordinary_glob_and_nested_member_alias_imports_remain_unchanged() {
    let glob = parse_use_complete("use crate::math::*;");
    assert_eq!(
        glob.path,
        UsePath::Glob(SimplePath {
            segments: vec!["crate".into(), "math".into()],
        })
    );

    let nested = parse_use_complete("use crate::math::{left, right as local_right};");
    let UsePath::Nested(module, members) = nested.path else {
        panic!("expected the ordinary nested import form")
    };
    assert_eq!(module.segments, vec!["crate".into(), "math".into()]);
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].name.as_ref(), "left");
    assert_eq!(members[0].alias, None);
    assert_eq!(members[1].name.as_ref(), "right");
    assert_eq!(members[1].alias.as_deref(), Some("local_right"));
}
