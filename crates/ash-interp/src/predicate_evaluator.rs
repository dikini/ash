//! Authority-free runtime evaluator for Core contract predicates.
//!
//! This module evaluates [`PredicateNode`] trees over captured boundary
//! environments and snapshots. It is intentionally authority-free: it performs
//! arithmetic, comparisons, boolean operations, and binder/snapshot lookup only.
//! It does not dispatch operations, install handlers, or admit rows.
//!
//! Evaluation results are distinguished as:
//!
//! - `PredicateResult::False` - the predicate is well-formed but evaluates to false,
//!   which becomes a [`ContractViolation`](ash_core::core_ash_contract::ContractDiagnostic).
//! - `PredicateResult::Fault(f)` - the evaluator encountered an unexpected value
//!   shape, missing binder, or other internal fault, which becomes a
//!   [`ContractPredicateFault`](ash_core::core_ash_contract::PredicateFaultDiagnostic).
//! - `PredicateResult::True` - the predicate evaluated to true.

use ash_core::{
    Value,
    core_ash::{CoreName, CoreType},
    core_ash_contract::{
        PredicateBinderKind, PredicateBinderRef, PredicateFault, PredicateNode, RuntimeCheckPlan,
        SnapshotRef,
    },
};
use std::collections::HashMap;

use crate::runtime_state::RuntimeState;

/// Captured value environment for a dynamic predicate check.
///
/// Binders are keyed by their local name within the predicate boundary. Snapshots
/// are keyed by a stable identity derived from their root binder and projection path.
#[derive(Debug, Clone, Default)]
pub struct PredicateCapture {
    /// Current binder values (parameters, `result`, etc.) keyed by local name.
    pub binders: HashMap<String, Value>,
    /// Snapshot values keyed by `snapshot_key`.
    pub snapshots: HashMap<String, Value>,
}

impl PredicateCapture {
    /// Create an empty capture.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a local name to a captured value.
    #[must_use]
    pub fn with_binder(mut self, name: impl Into<String>, value: Value) -> Self {
        self.binders.insert(name.into(), value);
        self
    }

    /// Bind a snapshot value.
    #[must_use]
    pub fn with_snapshot(mut self, snapshot: &SnapshotRef, value: Value) -> Self {
        self.snapshots.insert(snapshot_key(snapshot), value);
        self
    }
}

/// Result of evaluating a predicate at runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateResult {
    /// Predicate evaluated to true.
    True,
    /// Predicate is well-formed and evaluates to false.
    False,
    /// Predicate evaluator encountered a fault (distinct from a false predicate).
    Fault(PredicateFault),
}

/// Intermediate value during predicate evaluation.
///
/// Internal to the evaluator; public callers receive only [`PredicateResult`].
#[derive(Debug, Clone, PartialEq)]
enum EvaluatedValue {
    True,
    False,
    Fault(PredicateFault),
    Value(Value),
}

/// Evaluate a predicate node against a captured environment.
///
/// The evaluator is authority-free: it never dispatches operations, calls
/// handlers, or performs row admission. It only inspects captured values and
/// applies arithmetic/comparison/boolean operators.
#[must_use]
pub fn evaluate_predicate(node: &PredicateNode, capture: &PredicateCapture) -> PredicateResult {
    match eval_node(node, capture) {
        EvaluatedValue::True => PredicateResult::True,
        EvaluatedValue::False => PredicateResult::False,
        EvaluatedValue::Fault(f) => PredicateResult::Fault(f),
        EvaluatedValue::Value(Value::Bool(true)) => PredicateResult::True,
        EvaluatedValue::Value(Value::Bool(false)) => PredicateResult::False,
        EvaluatedValue::Value(_) => PredicateResult::Fault(PredicateFault::TypeMismatch {
            expected: CoreType::Base("Bool".to_string()),
            actual: CoreType::Base("?".to_string()),
        }),
    }
}

fn eval_node(node: &PredicateNode, capture: &PredicateCapture) -> EvaluatedValue {
    match node {
        PredicateNode::BoolLit(value) => {
            if *value {
                EvaluatedValue::True
            } else {
                EvaluatedValue::False
            }
        }
        PredicateNode::IntLit(value) => {
            EvaluatedValue::Value(Value::Int((*value).try_into().unwrap_or(i64::MAX)))
        }
        PredicateNode::StringLit(value) => EvaluatedValue::Value(Value::String(value.clone())),
        PredicateNode::UnitLit => EvaluatedValue::Value(Value::Null),
        PredicateNode::Binder(ref_) | PredicateNode::Result(ref_) | PredicateNode::Message(ref_) => {
            lookup_binder(ref_, capture)
        }
        PredicateNode::Snapshot(snapshot) => lookup_snapshot(snapshot, capture),
        PredicateNode::Field { base, field } => match eval_node(base, capture) {
            EvaluatedValue::Fault(f) => EvaluatedValue::Fault(f),
            EvaluatedValue::Value(v) => field_value(&v, field),
            EvaluatedValue::True | EvaluatedValue::False => EvaluatedValue::Fault(
                PredicateFault::TypeMismatch {
                    expected: CoreType::Base("Record".to_string()),
                    actual: CoreType::Base("Bool".to_string()),
                },
            ),
        },
        PredicateNode::TupleIndex { base, index } => match eval_node(base, capture) {
            EvaluatedValue::Fault(f) => EvaluatedValue::Fault(f),
            EvaluatedValue::Value(v) => tuple_index_value(&v, *index),
            EvaluatedValue::True | EvaluatedValue::False => EvaluatedValue::Fault(
                PredicateFault::TypeMismatch {
                    expected: CoreType::Base("Tuple".to_string()),
                    actual: CoreType::Base("Bool".to_string()),
                },
            ),
        },
        PredicateNode::Not(inner) => match eval_node(inner, capture) {
            EvaluatedValue::True => EvaluatedValue::False,
            EvaluatedValue::False => EvaluatedValue::True,
            EvaluatedValue::Fault(f) => EvaluatedValue::Fault(f),
            EvaluatedValue::Value(Value::Bool(b)) => {
                if b {
                    EvaluatedValue::False
                } else {
                    EvaluatedValue::True
                }
            }
            EvaluatedValue::Value(_) => EvaluatedValue::Fault(PredicateFault::TypeMismatch {
                expected: CoreType::Base("Bool".to_string()),
                actual: CoreType::Base("?".to_string()),
            }),
        },
        PredicateNode::And(left, right) => {
            short_circuit(left, right, capture, true, |a, b| a && b)
        }
        PredicateNode::Or(left, right) => {
            short_circuit(left, right, capture, false, |a, b| a || b)
        }
        PredicateNode::Implies(left, right) => {
            short_circuit(left, right, capture, true, |a, b| (!a) || b)
        }
        PredicateNode::Eq(left, right) => {
            compare(left, right, capture, |cmp| cmp == std::cmp::Ordering::Equal)
        }
        PredicateNode::Ne(left, right) => {
            compare(left, right, capture, |cmp| cmp != std::cmp::Ordering::Equal)
        }
        PredicateNode::Lt(left, right) => {
            compare(left, right, capture, |cmp| cmp == std::cmp::Ordering::Less)
        }
        PredicateNode::Le(left, right) => {
            compare(left, right, capture, |cmp| cmp != std::cmp::Ordering::Greater)
        }
        PredicateNode::Gt(left, right) => {
            compare(left, right, capture, |cmp| cmp == std::cmp::Ordering::Greater)
        }
        PredicateNode::Ge(left, right) => {
            compare(left, right, capture, |cmp| cmp != std::cmp::Ordering::Less)
        }
        PredicateNode::Add(left, right) => arithmetic(left, right, capture, |a, b| a + b),
        PredicateNode::Sub(left, right) => arithmetic(left, right, capture, |a, b| a - b),
        PredicateNode::Mul(left, right) => arithmetic(left, right, capture, |a, b| a * b),
        PredicateNode::Div(left, right) => div_rem(left, right, capture, |a, b| {
            if b == 0 {
                return EvaluatedValue::Fault(PredicateFault::EvaluatorTrap(
                    "division by zero".to_string(),
                ));
            }
            EvaluatedValue::Value(Value::Int(a / b))
        }),
        PredicateNode::Rem(left, right) => div_rem(left, right, capture, |a, b| {
            if b == 0 {
                return EvaluatedValue::Fault(PredicateFault::EvaluatorTrap(
                    "remainder by zero".to_string(),
                ));
            }
            EvaluatedValue::Value(Value::Int(a % b))
        }),
        PredicateNode::PredicateCall { .. } => EvaluatedValue::Fault(PredicateFault::EvaluatorTrap(
            "predicate calls require provider authority; not available in authority-free evaluator"
                .to_string(),
        )),
    }
}

fn lookup_binder(ref_: &PredicateBinderRef, capture: &PredicateCapture) -> EvaluatedValue {
    let local = ref_.id().local();
    capture
        .binders
        .get(local)
        .cloned()
        .map(EvaluatedValue::Value)
        .unwrap_or_else(|| EvaluatedValue::Fault(PredicateFault::MissingBinder(local.to_string())))
}

fn lookup_snapshot(snapshot: &SnapshotRef, capture: &PredicateCapture) -> EvaluatedValue {
    capture
        .snapshots
        .get(&snapshot_key(snapshot))
        .cloned()
        .map(EvaluatedValue::Value)
        .unwrap_or_else(|| {
            EvaluatedValue::Fault(PredicateFault::MissingSnapshot(snapshot_key(snapshot)))
        })
}

fn snapshot_key(snapshot: &SnapshotRef) -> String {
    format!(
        "{}:{}",
        snapshot.root().local(),
        snapshot
            .path()
            .iter()
            .map(|n: &CoreName| n.as_str())
            .collect::<Vec<_>>()
            .join(".")
    )
}

/// Builds a [`PredicateCapture`] for a dynamic [`RuntimeCheckPlan`].
///
/// Parameter binders are mapped from `arguments` in order. The result binder,
/// if present, is mapped from `result`. Snapshot values are resolved from the
/// runtime state registry when present.
pub fn build_predicate_capture(
    runtime_state: &RuntimeState,
    plan: &RuntimeCheckPlan,
    arguments: &[Value],
    result: Option<&Value>,
) -> PredicateCapture {
    let mut capture = PredicateCapture::new();
    let mut arg_iter = arguments.iter();
    for binder in plan.environment_binders() {
        match binder.kind() {
            PredicateBinderKind::Parameter | PredicateBinderKind::Lexical => {
                if let Some(arg) = arg_iter.next() {
                    capture = capture.with_binder(binder.id().local(), arg.clone());
                }
            }
            PredicateBinderKind::Result => {
                if let Some(res) = result {
                    capture = capture.with_binder(binder.id().local(), res.clone());
                }
            }
            _ => {}
        }
    }
    // Resolve any binders already captured in the runtime state (e.g., snapshots).
    for binder in plan.environment_binders() {
        if let Some(value) = runtime_state.predicate_binder_value(binder.id()) {
            capture = capture.with_binder(binder.id().local(), value);
        }
    }
    for snapshot in plan.snapshot_refs() {
        if let Some(value) = runtime_state.predicate_snapshot_value(snapshot) {
            capture = capture.with_snapshot(snapshot, value);
        }
    }
    capture
}

/// Evaluates a dynamic [`RuntimeCheckPlan`] against a captured boundary environment.
///
/// Returns `PredicateResult::True` if the predicate holds, `False` if it is
/// well-formed but false, and `Fault` if the evaluator encounters a fault.
pub fn evaluate_runtime_check_plan(
    runtime_state: &RuntimeState,
    plan: &RuntimeCheckPlan,
    arguments: &[Value],
    result: Option<&Value>,
) -> PredicateResult {
    let capture = build_predicate_capture(runtime_state, plan, arguments, result);
    evaluate_predicate(plan.predicate_node(), &capture)
}

fn field_value(value: &Value, field: &CoreName) -> EvaluatedValue {
    match value {
        Value::Record(fields) => fields
            .iter()
            .find_map(|(n, v)| {
                if n == field.as_str() {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .map(EvaluatedValue::Value)
            .unwrap_or_else(|| {
                EvaluatedValue::Fault(PredicateFault::EvaluatorTrap(format!(
                    "field '{}' not found in record",
                    field.as_str()
                )))
            }),
        _ => EvaluatedValue::Fault(PredicateFault::TypeMismatch {
            expected: CoreType::Base("Record".to_string()),
            actual: value_type(value),
        }),
    }
}

fn tuple_index_value(value: &Value, index: usize) -> EvaluatedValue {
    match value {
        // Ash represents tuple-shaped values as records at runtime. We support
        // tuple indexing by selecting the `index`-th field after sorting the
        // record keys lexicographically. This is stable for tuple records created
        // by the compiler, which use positional field names such as "0", "1", … .
        Value::Record(fields) => {
            let mut ordered: Vec<(&String, &Value)> = fields.iter().collect();
            ordered.sort_by(|(a, _), (b, _)| a.cmp(b));
            ordered
                .get(index)
                .map(|(_, v)| EvaluatedValue::Value((*v).clone()))
                .unwrap_or_else(|| {
                    EvaluatedValue::Fault(PredicateFault::EvaluatorTrap(format!(
                        "tuple index {index} out of bounds (length {})",
                        ordered.len()
                    )))
                })
        }
        _ => EvaluatedValue::Fault(PredicateFault::TypeMismatch {
            expected: CoreType::Base("Tuple".to_string()),
            actual: value_type(value),
        }),
    }
}

fn value_type(value: &Value) -> CoreType {
    match value {
        Value::Bool(_) => CoreType::Base("Bool".to_string()),
        Value::Int(_) => CoreType::Base("Int".to_string()),
        Value::String(_) => CoreType::Base("String".to_string()),
        Value::Null => CoreType::Base("Unit".to_string()),
        Value::Record(_) => CoreType::Base("Record".to_string()),
        Value::Variant { .. } => CoreType::Base("Variant".to_string()),
        _ => CoreType::Base("?".to_string()),
    }
}

fn short_circuit(
    left: &PredicateNode,
    right: &PredicateNode,
    capture: &PredicateCapture,
    default: bool,
    combine: impl FnOnce(bool, bool) -> bool,
) -> EvaluatedValue {
    let left_value = match eval_node(left, capture) {
        EvaluatedValue::True => true,
        EvaluatedValue::False => false,
        EvaluatedValue::Fault(f) => return EvaluatedValue::Fault(f),
        EvaluatedValue::Value(Value::Bool(b)) => b,
        EvaluatedValue::Value(_) => {
            return EvaluatedValue::Fault(PredicateFault::TypeMismatch {
                expected: CoreType::Base("Bool".to_string()),
                actual: CoreType::Base("?".to_string()),
            });
        }
    };
    if left_value == default {
        return if combine(left_value, default) {
            EvaluatedValue::True
        } else {
            EvaluatedValue::False
        };
    }
    let right_value = match eval_node(right, capture) {
        EvaluatedValue::True => true,
        EvaluatedValue::False => false,
        EvaluatedValue::Fault(f) => return EvaluatedValue::Fault(f),
        EvaluatedValue::Value(Value::Bool(b)) => b,
        EvaluatedValue::Value(_) => {
            return EvaluatedValue::Fault(PredicateFault::TypeMismatch {
                expected: CoreType::Base("Bool".to_string()),
                actual: CoreType::Base("?".to_string()),
            });
        }
    };
    if combine(left_value, right_value) {
        EvaluatedValue::True
    } else {
        EvaluatedValue::False
    }
}

fn compare(
    left: &PredicateNode,
    right: &PredicateNode,
    capture: &PredicateCapture,
    decide: impl FnOnce(std::cmp::Ordering) -> bool,
) -> EvaluatedValue {
    match (eval_node(left, capture), eval_node(right, capture)) {
        (EvaluatedValue::Fault(f), _) | (_, EvaluatedValue::Fault(f)) => EvaluatedValue::Fault(f),
        (EvaluatedValue::Value(left_value), EvaluatedValue::Value(right_value)) => {
            match compare_values(&left_value, &right_value) {
                Some(ordering) => {
                    if decide(ordering) {
                        EvaluatedValue::True
                    } else {
                        EvaluatedValue::False
                    }
                }
                None => EvaluatedValue::Fault(PredicateFault::TypeMismatch {
                    expected: value_type(&left_value),
                    actual: value_type(&right_value),
                }),
            }
        }
        _ => EvaluatedValue::Fault(PredicateFault::EvaluatorTrap(
            "comparison requires value operands".to_string(),
        )),
    }
}

fn compare_values(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
        (Value::String(a), Value::String(b)) => a.partial_cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
        _ => None,
    }
}

fn arithmetic(
    left: &PredicateNode,
    right: &PredicateNode,
    capture: &PredicateCapture,
    op: impl FnOnce(i64, i64) -> i64,
) -> EvaluatedValue {
    match (eval_node(left, capture), eval_node(right, capture)) {
        (EvaluatedValue::Fault(f), _) | (_, EvaluatedValue::Fault(f)) => EvaluatedValue::Fault(f),
        (EvaluatedValue::Value(Value::Int(a)), EvaluatedValue::Value(Value::Int(b))) => {
            EvaluatedValue::Value(Value::Int(op(a, b)))
        }
        _ => EvaluatedValue::Fault(PredicateFault::TypeMismatch {
            expected: CoreType::Base("Int".to_string()),
            actual: CoreType::Base("?".to_string()),
        }),
    }
}

fn div_rem(
    left: &PredicateNode,
    right: &PredicateNode,
    capture: &PredicateCapture,
    op: impl FnOnce(i64, i64) -> EvaluatedValue,
) -> EvaluatedValue {
    match (eval_node(left, capture), eval_node(right, capture)) {
        (EvaluatedValue::Fault(f), _) | (_, EvaluatedValue::Fault(f)) => EvaluatedValue::Fault(f),
        (EvaluatedValue::Value(Value::Int(a)), EvaluatedValue::Value(Value::Int(b))) => op(a, b),
        _ => EvaluatedValue::Fault(PredicateFault::TypeMismatch {
            expected: CoreType::Base("Int".to_string()),
            actual: CoreType::Base("?".to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::core_ash_contract::{CoreBoundaryId, PredicateBinderId, PredicateBinderRef};

    fn sample_binder_ref(local: &str, boundary: &str) -> PredicateBinderRef {
        PredicateBinderRef::new(PredicateBinderId::new(
            CoreBoundaryId::from(boundary),
            local,
        ))
    }

    fn bool_binder(
        name: &str,
        boundary: &str,
        value: bool,
    ) -> (PredicateBinderRef, PredicateCapture) {
        let ref_ = sample_binder_ref(name, boundary);
        let capture = PredicateCapture::new().with_binder(name, Value::Bool(value));
        (ref_, capture)
    }

    #[test]
    fn bool_literal_true() {
        assert_eq!(
            evaluate_predicate(&PredicateNode::BoolLit(true), &PredicateCapture::new()),
            PredicateResult::True
        );
    }

    #[test]
    fn bool_literal_false() {
        assert_eq!(
            evaluate_predicate(&PredicateNode::BoolLit(false), &PredicateCapture::new()),
            PredicateResult::False
        );
    }

    #[test]
    fn and_short_circuits_false() {
        let node = PredicateNode::And(
            Box::new(PredicateNode::BoolLit(false)),
            Box::new(PredicateNode::BoolLit(true)),
        );
        assert_eq!(
            evaluate_predicate(&node, &PredicateCapture::new()),
            PredicateResult::False
        );
    }

    #[test]
    fn or_short_circuits_true() {
        let node = PredicateNode::Or(
            Box::new(PredicateNode::BoolLit(true)),
            Box::new(PredicateNode::BoolLit(false)),
        );
        assert_eq!(
            evaluate_predicate(&node, &PredicateCapture::new()),
            PredicateResult::True
        );
    }

    #[test]
    fn compare_ints_eq() {
        let node = PredicateNode::Eq(
            Box::new(PredicateNode::IntLit(2)),
            Box::new(PredicateNode::IntLit(2)),
        );
        assert_eq!(
            evaluate_predicate(&node, &PredicateCapture::new()),
            PredicateResult::True
        );
    }

    #[test]
    fn binder_lookup() {
        let (ref_, capture) = bool_binder("x", "b", true);
        assert_eq!(
            evaluate_predicate(&PredicateNode::Binder(ref_), &capture),
            PredicateResult::True
        );
    }

    #[test]
    fn missing_binder_fault() {
        let ref_ = sample_binder_ref("missing", "b");
        assert!(matches!(
            evaluate_predicate(&PredicateNode::Binder(ref_), &PredicateCapture::new()),
            PredicateResult::Fault(PredicateFault::MissingBinder(_))
        ));
    }
}
