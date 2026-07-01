//! Completion support for Ash source files.
//!
//! MVP: returns keyword completions and top-level definition name completions.
//! Context-aware completion (type-position, expression-position, etc.) is deferred.

use ash_parser::surface::{Definition, ModuleFile};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, InsertTextFormat};

/// All Ash language keywords that make sense as completion candidates.
const KEYWORDS: &[(&str, &str)] = &[
    ("workflow", "workflow $1 { $0 }"),
    ("fn", "fn $1() -> $0 {  }"),
    ("capability", "capability $1: $0()"),
    ("policy", "policy $1 { $0 }"),
    ("role", "role $1 { $0 }"),
    ("proxy", "proxy $1 for $2 { $0 }"),
    ("interface", "interface $1 { $0 }"),
    ("impl", "impl $1 for $2 { $0 }"),
    ("prop", "prop $1<$2>;"),
    ("mod", "mod $1;"),
    ("observe", "observe $0"),
    ("orient", "orient $0"),
    ("act", "act $0"),
    ("decide", "decide { $1 } under $2 then { $0 }"),
    ("check", "check $0"),
    ("let", "let $1 = $0"),
    ("if", "if $1 then { $0 }"),
    ("for", "for $1 in $2 { $0 }"),
    ("with", "with $1 { $0 }"),
    ("maybe", "maybe $1 else { $0 }"),
    ("must", "must $0"),
    ("propose", "propose $0"),
    ("send", "send $0"),
    ("yield", "yield $0"),
    ("receive", "receive $0"),
    ("done", "done"),
    ("resume", "resume $0"),
    ("set", "set $1 = $0"),
    ("oblige", "oblige $0"),
    ("Int", "Int"),
    ("String", "String"),
    ("Bool", "Bool"),
    ("Unit", "Unit"),
    ("List", "List<$0>"),
    ("true", "true"),
    ("false", "false"),
];

fn keyword_completions() -> Vec<CompletionItem> {
    KEYWORDS
        .iter()
        .map(|(label, snippet)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        })
        .collect()
}

fn definition_name(current_token: Option<&str>, def: &Definition) -> Option<String> {
    match def {
        Definition::Notation(n) if Some(n.pattern.raw.as_ref()) == current_token => None,
        Definition::Notation(n) => Some(n.pattern.raw.as_ref().to_string()),
        Definition::Macro(m) if Some(m.name.as_ref()) == current_token => None,
        Definition::Macro(m) => Some(m.name.as_ref().to_string()),
        Definition::Function(f) if Some(f.name.as_ref()) == current_token => None, // skip self
        Definition::Function(f) => Some(f.name.as_ref().to_string()),
        Definition::Capability(c) if Some(c.name.as_ref()) == current_token => None,
        Definition::Capability(c) => Some(c.name.as_ref().to_string()),
        Definition::Policy(p) if Some(p.name.as_ref()) == current_token => None,
        Definition::Policy(p) => Some(p.name.as_ref().to_string()),
        Definition::Role(r) if Some(r.name.as_ref()) == current_token => None,
        Definition::Role(r) => Some(r.name.as_ref().to_string()),
        Definition::Proxy(p) if Some(p.name.as_ref()) == current_token => None,
        Definition::Proxy(p) => Some(p.name.as_ref().to_string()),
        Definition::Interface(i) if Some(i.name.as_ref()) == current_token => None,
        Definition::Interface(i) => Some(i.name.as_ref().to_string()),
        Definition::CapabilityInterface(i) if Some(i.name.as_ref()) == current_token => None,
        Definition::CapabilityInterface(i) => Some(i.name.as_ref().to_string()),
        Definition::CapabilityImplementation(i) if Some(i.name.as_ref()) == current_token => None,
        Definition::CapabilityImplementation(i) => Some(i.name.as_ref().to_string()),
        Definition::ResourceType(r) if Some(r.name.as_ref()) == current_token => None,
        Definition::ResourceType(r) => Some(r.name.as_ref().to_string()),
        Definition::Type(t) if Some(t.name.as_ref()) == current_token => None,
        Definition::Type(t) => Some(t.name.as_ref().to_string()),
        Definition::DataKind(d) if Some(d.name.as_ref()) == current_token => None,
        Definition::DataKind(d) => Some(d.name.as_ref().to_string()),
        Definition::TypeFn(t) if Some(t.name.as_ref()) == current_token => None,
        Definition::TypeFn(t) => Some(t.name.as_ref().to_string()),
        Definition::PropositionPredicate(p) if Some(p.name.as_ref()) == current_token => None,
        Definition::PropositionPredicate(p) => Some(p.name.as_ref().to_string()),
        Definition::Impl(_) | Definition::Law(_) => None, // impl blocks and laws don't have a useful completion name
        Definition::Proof(p) if Some(p.name.as_ref()) == current_token => None,
        Definition::Proof(p) => Some(p.name.as_ref().to_string()),
        Definition::BuiltinFn(b) if Some(b.name.as_ref()) == current_token => None,
        Definition::BuiltinFn(b) => Some(b.name.as_ref().to_string()),
        Definition::SealedDomain(d) if Some(d.name.as_ref()) == current_token => None,
        Definition::SealedDomain(d) => Some(d.name.as_ref().to_string()),
    }
}

const fn definition_kind(def: &Definition) -> CompletionItemKind {
    match def {
        Definition::Function(_)
        | Definition::BuiltinFn(_)
        | Definition::TypeFn(_)
        | Definition::PropositionPredicate(_) => CompletionItemKind::FUNCTION,
        Definition::Macro(_) => CompletionItemKind::SNIPPET,
        Definition::Capability(_) | Definition::Role(_) | Definition::Proxy(_) => {
            CompletionItemKind::CLASS
        }
        Definition::Policy(_) | Definition::ResourceType(_) | Definition::Type(_) => {
            CompletionItemKind::STRUCT
        }
        Definition::Interface(_) | Definition::CapabilityInterface(_) => {
            CompletionItemKind::INTERFACE
        }
        Definition::CapabilityImplementation(_) | Definition::Impl(_) => CompletionItemKind::CLASS,
        Definition::SealedDomain(_) | Definition::DataKind(_) => CompletionItemKind::ENUM,
        Definition::Law(_) | Definition::Proof(_) => CompletionItemKind::PROPERTY,
        Definition::Notation(_) => CompletionItemKind::OPERATOR,
    }
}

fn collect_definitions(
    current_token: Option<&str>,
    definitions: &[Definition],
    items: &mut Vec<CompletionItem>,
) {
    for def in definitions {
        if let Some(name) = definition_name(current_token, def) {
            let detail = if matches!(def, Definition::Macro(_)) {
                Some("syntax-phase macro".to_string())
            } else {
                None
            };
            items.push(CompletionItem {
                label: name,
                kind: Some(definition_kind(def)),
                detail,
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..CompletionItem::default()
            });
        }
        // Offer interface methods as completions.
        if let Definition::Interface(iface) = def {
            for method in &iface.methods {
                let mname = method.name.as_ref();
                if Some(mname) != current_token {
                    items.push(CompletionItem {
                        label: format!("{}.{}", iface.name, mname),
                        kind: Some(CompletionItemKind::METHOD),
                        insert_text: Some(mname.to_string()),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        ..CompletionItem::default()
                    });
                }
            }
        }
    }
}

/// Compute completion items at the given cursor position.
///
/// Returns keyword completions plus the names of all top-level definitions in
/// the module (excluding the token under the cursor, if any).
#[must_use]
pub fn completions(module: &ModuleFile, source: &str, line: u32, col: u32) -> CompletionResponse {
    // Determine the token under the cursor (if any) so we can exclude it.
    let current_token = crate::position::offset_from_line_col(source, line, col)
        .and_then(|off| crate::position::token_at_offset(source, off))
        .map(str::to_string);

    let mut items = keyword_completions();

    // Workflow entry name.
    if let Some(ref wf) = module.workflow {
        let name = wf.name.as_ref();
        if current_token.as_deref() != Some(name) {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::EVENT),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..CompletionItem::default()
            });
        }
    }

    // Module declarations.
    for decl in &module.module_decls {
        let dname = decl.name.as_ref();
        if current_token.as_deref() != Some(dname) {
            items.push(CompletionItem {
                label: dname.to_string(),
                kind: Some(CompletionItemKind::MODULE),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..CompletionItem::default()
            });
        }
        if let Some(defs) = decl.definitions() {
            collect_definitions(current_token.as_deref(), defs, &mut items);
        }
    }

    // Top-level definitions.
    collect_definitions(current_token.as_deref(), &module.definitions, &mut items);

    CompletionResponse::Array(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::parse_surface_file;

    #[test]
    fn test_keyword_completions_present() {
        let source = "workflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        let CompletionResponse::Array(items) = completions(&module, source, 0, 0) else {
            panic!("expected array response");
        };
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"fn"), "fn keyword should be present");
        assert!(
            labels.contains(&"workflow"),
            "workflow keyword should be present"
        );
        assert!(labels.contains(&"let"), "let keyword should be present");
    }

    #[test]
    fn test_definition_names_in_completions() {
        let source =
            "fn helper() -> Int { 1 }\ncapability sensor: epistemic()\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        let CompletionResponse::Array(items) = completions(&module, source, 0, 0) else {
            panic!("expected array response");
        };
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels.contains(&"helper"),
            "function name should be present"
        );
        assert!(
            labels.contains(&"sensor"),
            "capability name should be present"
        );
        assert!(labels.contains(&"main"), "workflow name should be present");
    }

    #[test]
    fn test_current_token_excluded() {
        let source = "fn helper() -> Int { 1 }\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        // Cursor on "helper" (col 3) — should NOT include "helper" as a completion
        let CompletionResponse::Array(items) = completions(&module, source, 0, 3) else {
            panic!("expected array response");
        };
        assert!(
            !items
                .iter()
                .map(|i| i.label.as_str())
                .any(|x| x == "helper"),
            "current token should be excluded from completions"
        );
    }

    #[test]
    fn test_interface_methods_in_completions() {
        let source = "interface Store { get(String) -> String }\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        let CompletionResponse::Array(items) = completions(&module, source, 0, 0) else {
            panic!("expected array response");
        };
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels.iter().any(|l| l.contains("Store.get")),
            "interface method should be present"
        );
    }

    #[test]
    fn test_macro_completion_is_syntax_phase_not_function() {
        let source = "macro id(x) => x;\nfn helper() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        let CompletionResponse::Array(items) = completions(&module, source, 0, 0) else {
            panic!("expected array response");
        };

        let macro_item = items
            .iter()
            .find(|item| item.label == "id")
            .expect("macro completion");
        assert_eq!(macro_item.kind, Some(CompletionItemKind::SNIPPET));
        assert_eq!(macro_item.detail.as_deref(), Some("syntax-phase macro"));

        let fn_item = items
            .iter()
            .find(|item| item.label == "helper")
            .expect("function completion");
        assert_eq!(fn_item.kind, Some(CompletionItemKind::FUNCTION));
        assert_ne!(fn_item.detail.as_deref(), Some("syntax-phase macro"));
    }
}
