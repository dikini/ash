//! Hover support for Ash source files.
//!
//! Current MVP behavior:
//! - Keyword documentation for core language keywords.
//! - Top-level signature hover for workflows, functions, capabilities, interfaces,
//!   interface methods, impl methods, proxies, roles, policies, and modules.
//!
//! TODO(TASK-569): add expression-level type hover once `ash-typeck` exposes
//! positional inferred-type data instead of an empty `inferred_types` map.

#![allow(
    clippy::map_unwrap_or,
    clippy::collapsible_if,
    clippy::option_if_let_else,
    clippy::manual_let_else,
    clippy::needless_pass_by_value
)]

use ash_parser::module::ModuleDecl;
use ash_parser::surface::{
    Definition, FnDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig, ModuleFile, Type,
    WorkflowDef,
};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};

use crate::position::{offset_from_line_col, token_at_offset};

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Name(name) => name.to_string(),
        Type::List(inner) => format!("List<{}>", type_to_string(inner)),
        Type::Tuple(items) => {
            let inner = items
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({inner})")
        }
        Type::Record(fields) => {
            let inner = fields
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, type_to_string(ty)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{inner}}}")
        }
        Type::Capability(name) => format!("Capability<{name}>"),
        Type::Constructor { name, args } => {
            if args.is_empty() {
                name.to_string()
            } else {
                let args = args
                    .iter()
                    .map(type_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name}<{args}>")
            }
        }
        Type::Associated { base, name } => format!("{}::{}", type_to_string(base), name),
        Type::Fn(params, ret) => {
            let params = params
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({params}) -> {}", type_to_string(ret))
        }
    }
}

fn markdown(code: String, detail: Option<String>) -> Hover {
    let mut value = format!("```ash\n{code}\n```");
    if let Some(detail) = detail {
        value.push_str("\n\n");
        value.push_str(&detail);
    }
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: None,
    }
}

fn keyword_hover(token: &str) -> Option<Hover> {
    let doc = match token {
        "workflow" => (
            "workflow <name> { ... }",
            "Declare the file entry workflow.",
        ),
        "fn" => ("fn <name>(...) -> T { ... }", "Declare a pure function."),
        "capability" => (
            "capability <name>: <effect>(...)",
            "Declare a capability interface.",
        ),
        "policy" => (
            "policy <name> { ... }",
            "Declare a policy type / policy schema.",
        ),
        "role" => (
            "role <name> { ... }",
            "Declare a role and its exposed capabilities/obligations.",
        ),
        "proxy" => (
            "proxy <name> for <role> { ... }",
            "Declare a proxy workflow for a role.",
        ),
        "interface" => (
            "interface <name> { ... }",
            "Declare an interface with methods and associated types.",
        ),
        "impl" => (
            "impl <Interface> for ... { ... }",
            "Declare an interface implementation.",
        ),
        "mod" => ("mod <name>; / mod <name> { ... }", "Declare a module."),
        "observe" => (
            "observe <capability> ...",
            "Observation phase of an Ash workflow.",
        ),
        "orient" => (
            "orient <expr> ...",
            "Evaluate / analyze an expression in workflow context.",
        ),
        "propose" => (
            "propose <action> ...",
            "Propose an action in workflow context.",
        ),
        "decide" => (
            "decide { expr } under policy then ...",
            "Branch under a policy decision.",
        ),
        "check" => (
            "check <target> ...",
            "Check an obligation or policy instance.",
        ),
        "act" => ("act <action> ...", "Execute an action in workflow context."),
        "let" => (
            "let <pattern> = <expr> ...",
            "Bind a value in workflow context.",
        ),
        "if" => (
            "if <cond> then ... else ...",
            "Conditional workflow or expression form.",
        ),
        "for" => (
            "for <pat> in <expr> ...",
            "Iterate over a collection in workflow context.",
        ),
        "with" => (
            "with <capability> ...",
            "Run a workflow with a scoped capability.",
        ),
        "maybe" => (
            "maybe ... else ...",
            "Try a primary workflow and fall back on failure.",
        ),
        "must" => ("must ...", "Require that the nested workflow succeed."),
        "done" => ("done", "Successful workflow termination."),
        _ => return None,
    };

    Some(markdown(doc.0.to_string(), Some(doc.1.to_string())))
}

fn fn_hover(def: &FnDef) -> Hover {
    let params = def
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, type_to_string(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let ret = def
        .return_type
        .as_ref()
        .map(type_to_string)
        .unwrap_or_else(|| "Unit".to_string());
    markdown(
        format!("fn {}({params}) -> {ret}", def.name),
        Some("Pure function".to_string()),
    )
}

fn workflow_hover(def: &WorkflowDef) -> Hover {
    let params = def
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, type_to_string(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let ret = def
        .declared_return_type
        .as_ref()
        .map(type_to_string)
        .unwrap_or_else(|| "Unit".to_string());
    let effect = format!("{:?}", def.body.effect());
    markdown(
        format!("workflow {}({params}) -> {ret}", def.name),
        Some(format!("Workflow effect: `{effect}`")),
    )
}

fn interface_hover(def: &InterfaceDef) -> Hover {
    markdown(
        format!("interface {}", def.name),
        Some(format!("Methods: {}", def.methods.len())),
    )
}

fn interface_method_hover(method: &InterfaceMethodSig, interface_name: &str) -> Hover {
    let params = method
        .params
        .iter()
        .map(type_to_string)
        .collect::<Vec<_>>()
        .join(", ");
    markdown(
        format!(
            "fn {}.{}({params}) -> {}",
            interface_name,
            method.name,
            type_to_string(&method.return_type)
        ),
        Some("Interface method signature".to_string()),
    )
}
fn impl_method_hover(method: &ImplMethodDef, interface_name: &str) -> Hover {
    let params = method
        .params
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    markdown(
        format!("impl {}::{}({params})", interface_name, method.name),
        Some("Implementation method".to_string()),
    )
}

fn definition_hover(definition: &Definition) -> Hover {
    match definition {
        Definition::Capability(def) => {
            let params = def
                .params
                .iter()
                .map(|param| format!("{}: {}", param.name, type_to_string(&param.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = def
                .return_type
                .as_ref()
                .map(type_to_string)
                .unwrap_or_else(|| "Unit".to_string());
            markdown(
                format!(
                    "capability {}: {:?}({params}) -> {ret}",
                    def.name, def.effect
                ),
                Some("Capability declaration".to_string()),
            )
        }
        Definition::Policy(def) => markdown(
            format!("policy {}", def.name),
            Some(format!("Fields: {}", def.fields.len())),
        ),
        Definition::Role(def) => markdown(
            format!("role {}", def.name),
            Some(format!(
                "Capabilities: {}, obligations: {}",
                def.capabilities.len(),
                def.obligations.len()
            )),
        ),
        Definition::Proxy(def) => markdown(
            format!("proxy {} for {}", def.name, def.role),
            Some("Proxy workflow".to_string()),
        ),
        Definition::Interface(def) => interface_hover(def),
        Definition::CapabilityInterface(def) => markdown(
            format!("capability interface {}", def.name),
            Some(format!("Operations: {}", def.operations.len())),
        ),
        Definition::CapabilityImplementation(def) => markdown(
            format!("capability impl {} for {}", def.name, def.interface),
            Some(format!(
                "Dependencies: {}, operations: {}",
                def.dependencies.len(),
                def.operations.len()
            )),
        ),
        Definition::ResourceType(def) => markdown(
            format!("resource type {}", def.name),
            Some(format!("Fields: {}", def.fields.len())),
        ),
        Definition::Impl(def) => markdown(
            format!("impl {}", def.interface),
            Some(format!("Methods: {}", def.methods.len())),
        ),
        Definition::Function(def) => fn_hover(def),
        Definition::BuiltinFn(def) => {
            let params = def
                .params
                .iter()
                .map(|param| format!("{}: {}", param.name, type_to_string(&param.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = type_to_string(&def.return_type);
            markdown(
                format!("builtin fn {}({params}) -> {ret}", def.name),
                Some("Builtin function (runtime-provided)".to_string()),
            )
        }
    }
}

fn module_hover(module: &ModuleDecl) -> Hover {
    markdown(
        format!("mod {}", module.name),
        Some(match &module.source {
            ash_parser::module::ModuleSource::File => "File-backed module".to_string(),
            ash_parser::module::ModuleSource::Inline(defs) => {
                format!("Inline module with {} definition(s)", defs.len())
            }
        }),
    )
}

fn top_level_hover(token: &str, module: &ModuleFile) -> Option<Hover> {
    if let Some(workflow) = &module.workflow {
        if workflow.name.as_ref() == token {
            return Some(workflow_hover(workflow));
        }
    }

    for module_decl in &module.module_decls {
        if module_decl.name.as_ref() == token {
            return Some(module_hover(module_decl));
        }
        if let Some(defs) = module_decl.definitions() {
            for definition in defs {
                match definition {
                    Definition::Interface(def) => {
                        if def.name.as_ref() == token {
                            return Some(interface_hover(def));
                        }
                        if let Some(method) = def.methods.iter().find(|m| m.name.as_ref() == token)
                        {
                            return Some(interface_method_hover(method, def.name.as_ref()));
                        }
                    }
                    Definition::Impl(def) => {
                        if let Some(method) = def.methods.iter().find(|m| m.name.as_ref() == token)
                        {
                            return Some(impl_method_hover(method, def.interface.as_ref()));
                        }
                    }
                    _ => {
                        let name_matches = match definition {
                            Definition::Capability(def) => def.name.as_ref() == token,
                            Definition::CapabilityInterface(def) => def.name.as_ref() == token,
                            Definition::CapabilityImplementation(def) => def.name.as_ref() == token,
                            Definition::ResourceType(def) => def.name.as_ref() == token,
                            Definition::Policy(def) => def.name.as_ref() == token,
                            Definition::Role(def) => def.name.as_ref() == token,
                            Definition::Proxy(def) => def.name.as_ref() == token,
                            Definition::Function(def) => def.name.as_ref() == token,
                            Definition::Interface(_) | Definition::Impl(_) => false,
                            Definition::BuiltinFn(b) => b.name.as_ref() == token,
                        };
                        if name_matches {
                            return Some(definition_hover(definition));
                        }
                    }
                }
            }
        }
    }

    for definition in &module.definitions {
        match definition {
            Definition::Interface(def) => {
                if def.name.as_ref() == token {
                    return Some(interface_hover(def));
                }
                if let Some(method) = def.methods.iter().find(|m| m.name.as_ref() == token) {
                    return Some(interface_method_hover(method, def.name.as_ref()));
                }
            }
            Definition::Impl(def) => {
                if let Some(method) = def.methods.iter().find(|m| m.name.as_ref() == token) {
                    return Some(impl_method_hover(method, def.interface.as_ref()));
                }
            }
            _ => {
                let name_matches = match definition {
                    Definition::Capability(def) => def.name.as_ref() == token,
                    Definition::CapabilityInterface(def) => def.name.as_ref() == token,
                    Definition::CapabilityImplementation(def) => def.name.as_ref() == token,
                    Definition::ResourceType(def) => def.name.as_ref() == token,
                    Definition::Policy(def) => def.name.as_ref() == token,
                    Definition::Role(def) => def.name.as_ref() == token,
                    Definition::Proxy(def) => def.name.as_ref() == token,
                    Definition::Function(def) => def.name.as_ref() == token,
                    Definition::Interface(_) | Definition::Impl(_) => false,
                    Definition::BuiltinFn(b) => b.name.as_ref() == token,
                };
                if name_matches {
                    return Some(definition_hover(definition));
                }
            }
        }
    }

    None
}

#[must_use]
pub fn hover_at(module: &ModuleFile, source: &str, line: u32, col: u32) -> Option<Hover> {
    let offset = offset_from_line_col(source, line, col)?;
    let token = token_at_offset(source, offset)?;
    keyword_hover(token).or_else(|| top_level_hover(token, module))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::parse_surface_file;

    #[test]
    fn test_keyword_hover() {
        let source = "workflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        let hover = hover_at(&module, source, 0, 1).expect("hover exists");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markdown hover");
        };
        assert!(markup.value.contains("workflow <name>"));
    }

    #[test]
    fn test_workflow_hover() {
        let source = "workflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        let hover = hover_at(&module, source, 0, 10).expect("hover exists");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markdown hover");
        };
        assert!(markup.value.contains("workflow main() -> Unit"));
        assert!(markup.value.contains("Workflow effect"));
    }

    #[test]
    fn test_function_hover() {
        let source = "fn helper(x: Int) -> String { x }\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        let hover = hover_at(&module, source, 0, 4).expect("hover exists");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markdown hover");
        };
        assert!(markup.value.contains("fn helper(x: Int) -> String"));
    }

    #[test]
    fn test_no_hover_for_whitespace() {
        let source = "workflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        assert!(hover_at(&module, source, 0, 8).is_none());
    }
}
