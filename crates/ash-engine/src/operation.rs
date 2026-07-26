//! Concrete source operations admitted by the bounded engine entry path.

use ash_core::Expr;

/// One statically resolvable source operation admitted by a bounded task slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcreteOperationDescriptor {
    /// Source module qualifier.
    pub module: &'static str,
    /// Source operation name.
    pub name: &'static str,
    /// Existing provider authority required for admission.
    pub provider: &'static str,
}

impl ConcreteOperationDescriptor {
    /// Whether the source call fields carry exactly this concrete identity.
    #[must_use]
    pub fn matches_call_parts(self, module: Option<&str>, name: &str) -> bool {
        module == Some(self.module) && name == self.name
    }

    /// Whether a lowered legacy call carries exactly this concrete identity.
    #[must_use]
    pub fn matches_legacy_call(self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Call {
                func,
                module: Some(module),
                ..
            } if self.matches_call_parts(Some(module), func)
        )
    }
}

/// The sole statically resolvable operation admitted by TASK-2010.
pub const TIME_SLEEP_OPERATION: ConcreteOperationDescriptor = ConcreteOperationDescriptor {
    module: "time",
    name: "sleep",
    provider: "time",
};
