//! AST Visualization - Graphviz DOT output for workflows
//!
//! This module generates Graphviz DOT format representations of
//! workflow ASTs for debugging and documentation.
//!
//! # Example
//! ```ignore
//! use ash_core::visualize::ToDot;
//! use ash_core::ast::Workflow;
//!
//! let workflow = Workflow::Done;
//! println!("{}", workflow.to_dot());
//! ```

use crate::ast::{Expr, Pattern, Workflow};
use crate::effect::Effect;
use std::fmt::Write;

/// Trait for types that can be visualized as DOT graphs
pub trait ToDot {
    /// Generate DOT format representation
    fn to_dot(&self) -> String;
}

impl ToDot for Workflow {
    fn to_dot(&self) -> String {
        let mut generator = DotGenerator::new();
        generator.generate(self)
    }
}

/// DOT graph generator for workflows
pub struct DotGenerator {
    output: String,
    node_counter: usize,
}

impl DotGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            node_counter: 0,
        }
    }

    pub fn generate(&mut self, workflow: &Workflow) -> String {
        self.output.clear();
        self.node_counter = 0;

        writeln!(self.output, "digraph Workflow {{").unwrap();
        writeln!(self.output, "  rankdir=TB;").unwrap();
        writeln!(
            self.output,
            "  node [shape=box, style=\"rounded,filled\", fontname=\"Helvetica\"];"
        )
        .unwrap();
        writeln!(self.output, "  edge [fontname=\"Helvetica\"];").unwrap();
        writeln!(self.output).unwrap();

        let root_id = self.visit_workflow(workflow);
        writeln!(
            self.output,
            "  root [label=\"Workflow\", shape=ellipse, fillcolor=\"lightblue\"];"
        )
        .unwrap();
        writeln!(self.output, "  root -> node_{};", root_id).unwrap();

        writeln!(self.output, "}}").unwrap();
        self.output.clone()
    }

    fn next_id(&mut self) -> usize {
        let id = self.node_counter;
        self.node_counter += 1;
        id
    }

    fn visit_workflow(&mut self, workflow: &Workflow) -> usize {
        match workflow {
            Workflow::Observe {
                capability,
                pattern,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"OBSERVE\\n{}\", fillcolor=\"{}\"];",
                    id,
                    escape_dot(&capability.name),
                    effect_color(&Effect::Epistemic)
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"{}\"];",
                    id,
                    cont_id,
                    escape_dot(&pattern_to_string(pattern))
                )
                .unwrap();
                id
            }
            Workflow::Orient { expr, continuation } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                let expr_id = self.visit_expr(expr);
                writeln!(
                    self.output,
                    "  node_{} [label=\"ORIENT\", fillcolor=\"{}\"];",
                    id,
                    effect_color(&Effect::Deliberative)
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"expr\"];",
                    id, expr_id
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Propose {
                action,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"PROPOSE\\n{}\", fillcolor=\"{}\"];",
                    id,
                    escape_dot(&action.name),
                    effect_color(&Effect::Deliberative)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Decide {
                expr,
                policy,
                continuation,
            } => {
                let id = self.next_id();
                let expr_id = self.visit_expr(expr);
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"DECIDE\\npolicy: {}\", shape=diamond, fillcolor=\"{}\"];",
                    id,
                    escape_dot(policy),
                    effect_color(&Effect::Evaluative)
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"guard\"];",
                    id, expr_id
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Check {
                obligation,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"CHECK\\n{:.20}...\", fillcolor=\"lightsalmon\"];",
                    id,
                    escape_dot(&format!("{:?}", obligation))
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Act {
                action,
                guard,
                provenance: _,
            } => {
                let id = self.next_id();
                writeln!(
                    self.output,
                    "  node_{} [label=\"ACT\\n{}\", fillcolor=\"{}\"];",
                    id,
                    escape_dot(&action.name),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                // Optionally show guard
                let _ = guard;
                id
            }
            Workflow::Oblig { role, workflow } => {
                let id = self.next_id();
                let wf_id = self.visit_workflow(workflow);
                writeln!(
                    self.output,
                    "  node_{} [label=\"OBLIG\\n{}\", shape=component, fillcolor=\"lightcoral\"];",
                    id,
                    escape_dot(&role.name)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, wf_id).unwrap();
                id
            }
            Workflow::Let {
                pattern,
                expr,
                continuation,
            } => {
                let id = self.next_id();
                let expr_id = self.visit_expr(expr);
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"LET {}\", shape=note, fillcolor=\"lightcyan\"];",
                    id,
                    escape_dot(&pattern_to_string(pattern))
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"=\"];",
                    id, expr_id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"in\"];",
                    id, cont_id
                )
                .unwrap();
                id
            }
            Workflow::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let id = self.next_id();
                let cond_id = self.visit_expr(condition);
                let then_id = self.visit_workflow(then_branch);
                let else_id = self.visit_workflow(else_branch);
                writeln!(
                    self.output,
                    "  node_{} [label=\"IF\", shape=diamond, fillcolor=\"lightyellow\"];",
                    id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"cond\"];",
                    id, cond_id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"then\"];",
                    id, then_id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"else\"];",
                    id, else_id
                )
                .unwrap();
                id
            }
            Workflow::Seq { first, second } => {
                let id = self.next_id();
                let first_id = self.visit_workflow(first);
                let second_id = self.visit_workflow(second);
                writeln!(
                    self.output,
                    "  node_{} [label=\"SEQ\", shape=box, fillcolor=\"white\"];",
                    id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"1\"];",
                    id, first_id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"2\"];",
                    id, second_id
                )
                .unwrap();
                id
            }
            Workflow::Par { workflows } => {
                let id = self.next_id();
                writeln!(
                    self.output,
                    "  node_{} [label=\"PAR\\n({} branches)\", shape=parallelogram, fillcolor=\"lightyellow\"];",
                    id,
                    workflows.len()
                ).unwrap();
                for (i, w) in workflows.iter().enumerate() {
                    let branch_id = self.visit_workflow(w);
                    writeln!(
                        self.output,
                        "  node_{} -> node_{} [label=\"{}\"];",
                        id,
                        branch_id,
                        i + 1
                    )
                    .unwrap();
                }
                id
            }
            Workflow::ForEach {
                pattern,
                collection,
                body,
            } => {
                let id = self.next_id();
                let coll_id = self.visit_expr(collection);
                let body_id = self.visit_workflow(body);
                writeln!(
                    self.output,
                    "  node_{} [label=\"FOREACH {}\", shape=trapezium, fillcolor=\"lightsteelblue\"];",
                    id,
                    escape_dot(&pattern_to_string(pattern))
                ).unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"in\"];",
                    id, coll_id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"do\"];",
                    id, body_id
                )
                .unwrap();
                id
            }
            Workflow::Ret { expr } => {
                let id = self.next_id();
                let expr_id = self.visit_expr(expr);
                writeln!(
                    self.output,
                    "  node_{} [label=\"RET\", shape=oval, fillcolor=\"palegreen\"];",
                    id
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, expr_id).unwrap();
                id
            }
            Workflow::With {
                capability,
                workflow,
            } => {
                let id = self.next_id();
                let wf_id = self.visit_workflow(workflow);
                writeln!(
                    self.output,
                    "  node_{} [label=\"WITH {}\", shape=house, fillcolor=\"lightgray\"];",
                    id,
                    escape_dot(&capability.name)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, wf_id).unwrap();
                id
            }
            Workflow::Maybe { primary, fallback } => {
                let id = self.next_id();
                let primary_id = self.visit_workflow(primary);
                let fallback_id = self.visit_workflow(fallback);
                writeln!(
                    self.output,
                    "  node_{} [label=\"MAYBE\", shape=ellipse, fillcolor=\"lightpink\"];",
                    id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"try\"];",
                    id, primary_id
                )
                .unwrap();
                writeln!(
                    self.output,
                    "  node_{} -> node_{} [label=\"else\"];",
                    id, fallback_id
                )
                .unwrap();
                id
            }
            Workflow::Must { workflow } => {
                let id = self.next_id();
                let wf_id = self.visit_workflow(workflow);
                writeln!(
                    self.output,
                    "  node_{} [label=\"MUST\", shape=doubleoctagon, fillcolor=\"lightcoral\"];",
                    id
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, wf_id).unwrap();
                id
            }
            Workflow::Set {
                capability,
                channel,
                value: _,
            } => {
                let id = self.next_id();
                writeln!(
                    self.output,
                    "  node_{} [label=\"SET\\n{}:{}\", fillcolor=\"{}\"];",
                    id,
                    escape_dot(capability),
                    escape_dot(channel),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                id
            }
            Workflow::Send {
                capability,
                channel,
                value: _,
            } => {
                let id = self.next_id();
                writeln!(
                    self.output,
                    "  node_{} [label=\"SEND\\n{}:{}\", fillcolor=\"{}\"];",
                    id,
                    escape_dot(capability),
                    escape_dot(channel),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                id
            }
            Workflow::Spawn {
                workflow_type,
                init: _,
                binding,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"SPAWN\\n{} as {}, fillcolor=\"{}\"];",
                    id,
                    escape_dot(workflow_type),
                    escape_dot(binding),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Split {
                instance,
                addr_binding,
                control_binding,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"SPLIT\\n{} -> ({}, {}), fillcolor=\"{}\"];",
                    id,
                    escape_dot(instance),
                    escape_dot(addr_binding),
                    escape_dot(control_binding),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Kill {
                target,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"KILL\\n{}, fillcolor=\"{}\"];",
                    id,
                    escape_dot(target),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Pause {
                target,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"PAUSE\\n{}, fillcolor=\"{}\"];",
                    id,
                    escape_dot(target),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Resume {
                target,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"RESUME\\n{}, fillcolor=\"{}\"];",
                    id,
                    escape_dot(target),
                    effect_color(&Effect::Operational)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::CheckHealth {
                target,
                continuation,
            } => {
                let id = self.next_id();
                let cont_id = self.visit_workflow(continuation);
                writeln!(
                    self.output,
                    "  node_{} [label=\"CHECK_HEALTH\\n{}, fillcolor=\"{}\"];",
                    id,
                    escape_dot(target),
                    effect_color(&Effect::Epistemic)
                )
                .unwrap();
                writeln!(self.output, "  node_{} -> node_{};", id, cont_id).unwrap();
                id
            }
            Workflow::Done => {
                let id = self.next_id();
                writeln!(
                    self.output,
                    "  node_{} [label=\"DONE\", shape=oval, fillcolor=\"palegreen\"];",
                    id
                )
                .unwrap();
                id
            }
        }
    }

    fn visit_expr(&mut self, _expr: &Expr) -> usize {
        // Simplified - create placeholder for expressions
        let id = self.next_id();
        writeln!(
            self.output,
            "  node_{} [label=\"expr\", shape=ellipse, fillcolor=\"white\"];",
            id
        )
        .unwrap();
        id
    }
}

impl Default for DotGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn effect_color(effect: &Effect) -> &'static str {
    match effect {
        Effect::Epistemic => "lightgreen",
        Effect::Deliberative => "lightyellow",
        Effect::Evaluative => "lightsalmon",
        Effect::Operational => "lightcoral",
    }
}

fn pattern_to_string(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Variable(name) => name.clone(),
        Pattern::Tuple(patterns) => {
            let inner: Vec<_> = patterns.iter().map(pattern_to_string).collect();
            format!("({})", inner.join(", "))
        }
        Pattern::Record(fields) => {
            let inner: Vec<_> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, pattern_to_string(v)))
                .collect();
            format!("{{ {} }}", inner.join(", "))
        }
        Pattern::Wildcard => "_".to_string(),
        Pattern::Literal(value) => format!("{:?}", value),
        Pattern::List(patterns, rest) => {
            let inner: Vec<_> = patterns.iter().map(pattern_to_string).collect();
            match rest {
                Some(name) => format!("[{}, ..{}]", inner.join(", "), name),
                None => format!("[{}]", inner.join(", ")),
            }
        }
        Pattern::Variant { name, fields } => {
            if let Some(fields) = fields {
                let inner: Vec<_> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, pattern_to_string(v)))
                    .collect();
                format!("{} {{ {} }}", name, inner.join(", "))
            } else {
                name.clone()
            }
        }
    }
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Action, Capability, Pattern};

    #[test]
    fn test_done_to_dot() {
        let workflow = Workflow::Done;
        let dot = workflow.to_dot();
        assert!(dot.contains("DONE"));
        assert!(dot.contains("palegreen"));
    }

    #[test]
    fn test_act_to_dot() {
        let workflow = Workflow::Act {
            action: Action {
                name: "notify".to_string(),
                arguments: vec![],
            },
            guard: crate::ast::Guard::Always,
            provenance: crate::Provenance::new(),
        };
        let dot = workflow.to_dot();
        assert!(dot.contains("ACT"));
        assert!(dot.contains("notify"));
        assert!(dot.contains("lightcoral")); // Operational color
    }

    #[test]
    fn test_observe_to_dot() {
        let workflow = Workflow::Observe {
            capability: Capability {
                name: "sensor".to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            pattern: Pattern::Variable("data".to_string()),
            continuation: Box::new(Workflow::Done),
        };
        let dot = workflow.to_dot();
        assert!(dot.contains("OBSERVE"));
        assert!(dot.contains("sensor"));
        assert!(dot.contains("lightgreen")); // Epistemic color
    }

    #[test]
    fn test_dot_is_valid_format() {
        let workflow = Workflow::Done;
        let dot = workflow.to_dot();
        assert!(dot.starts_with("digraph Workflow {"));
        assert!(dot.ends_with("}\n"));
    }

    #[test]
    fn test_seq_to_dot() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Done),
            second: Box::new(Workflow::Done),
        };
        let dot = workflow.to_dot();
        assert!(dot.contains("SEQ"));
    }
}
