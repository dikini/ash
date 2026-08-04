//! TASK-2058 parser-to-core module-name compatibility evidence.
//!
//! `ash-core` cannot depend on the parser, so this parser-owned integration
//! test keeps the canonical child-module identifier contract aligned without
//! introducing a reverse dependency.

use ash_core::module_graph::ModuleKey;
use ash_parser::parse_surface_file;

/// The canonical parser keyword set from `parse_utils::is_keyword`.
const PARSER_KEYWORDS: &[&str] = &[
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
    "on",
    "handle",
    "maybe",
    "must",
    "attempt",
    "retry",
    "timeout",
    "done",
    "ret",
    "epistemic",
    "deliberative",
    "evaluative",
    "operational",
    "authority",
    "obligations",
    "when",
    "returns",
    "where",
    "law",
    "proof",
    "by_definition",
    "permit",
    "deny",
    "require_approval",
    "escalate",
    "fn",
    "panic",
    "match",
    "fail",
    "with_error",
    "requires",
    "ensures",
    "set",
    "send",
    "in",
    "not",
    "and",
    "or",
    "true",
    "false",
    "null",
];

#[test]
fn parser_and_module_key_reject_every_canonical_keyword_as_a_child_name() {
    let root = ModuleKey::root("crate").expect("test crate name must be valid");

    for keyword in PARSER_KEYWORDS {
        let source = format!("mod {keyword};");
        assert!(
            parse_surface_file(&source).is_err(),
            "parser accepted reserved module name {keyword:?}"
        );
        assert!(
            root.child(keyword).is_err(),
            "ModuleKey accepted reserved module name {keyword:?}"
        );
    }
}

#[test]
fn parser_and_module_key_accept_representative_canonical_child_names() {
    let root = ModuleKey::root("crate").expect("test crate name must be valid");

    for name in ["Thing", "_private", "with-error"] {
        let source = format!("mod {name};");
        assert!(
            parse_surface_file(&source).is_ok(),
            "parser rejected canonical module name {name:?}"
        );
        assert!(
            root.child(name).is_ok(),
            "ModuleKey rejected canonical module name {name:?}"
        );
    }
}
