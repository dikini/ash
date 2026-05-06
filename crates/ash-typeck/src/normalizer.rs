//! Type-expression normalizer API skeleton for Phase 112.
//!
//! This module intentionally provides only the total API surface and structural
//! identity conversion from `CanonicalTypeExpr` to `NormalTypeExpr`. It does not
//! own fixture equation registration, reduction semantics, definitional equality
//! adoption, or associated-family computation; later SPEC-060 tasks build those
//! pieces on this boundary.

use crate::type_env::TypeEnv;
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
};

/// Normalization strategy requested by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizationMode {
    /// Reduce only as much as needed to expose the outermost head.
    WeakHead,
    /// Recursively normalize the entire reachable expression.
    Full,
    /// Use-site selected demand-driven normalization.
    Demand,
}

/// Robustness fuel for total normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalizationFuel {
    remaining: usize,
}

impl NormalizationFuel {
    /// Creates a fuel budget with the given number of normalization steps.
    #[must_use]
    pub const fn new(remaining: usize) -> Self {
        Self { remaining }
    }

    /// Returns the remaining budget.
    #[must_use]
    pub const fn remaining(self) -> usize {
        self.remaining
    }

    fn consume(&mut self, mode: NormalizationMode) -> NormalizationResult<()> {
        if self.remaining == 0 {
            return Err(NormalizationError::FuelExhausted { mode, remaining: 0 });
        }
        self.remaining -= 1;
        Ok(())
    }
}

impl Default for NormalizationFuel {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Options controlling a normalization request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationConfig {
    pub mode: NormalizationMode,
    pub fuel: NormalizationFuel,
    pub trace: bool,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            mode: NormalizationMode::Full,
            fuel: NormalizationFuel::default(),
            trace: false,
        }
    }
}

/// A skeleton trace event emitted by normalization when tracing is enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationTraceEvent {
    pub evidence: NormalizationEvidence,
    pub remaining_fuel: usize,
}

/// Structured evidence for why a normal form was produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationEvidence {
    /// The skeleton normalizer performed structural identity conversion only.
    StructuralIdentity,
    /// A computation head was preserved as neutral because no equation registry
    /// exists in TASK-819.
    NeutralUnsupportedComputation,
    /// A projection was preserved without associated-family computation.
    ProjectionPreserved { rigidity: ProjectionRigidity },
}

/// Successful normalization result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationOutcome {
    pub normal: NormalTypeExpr,
    pub mode: NormalizationMode,
    pub fuel_remaining: usize,
    pub evidence: NormalizationEvidence,
    pub trace: Vec<NormalizationTraceEvent>,
}

/// Normalizer robustness errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationError {
    /// The implementation fuel budget was exhausted. This is distinct from
    /// semantic neutral/stuck normal forms.
    FuelExhausted {
        mode: NormalizationMode,
        remaining: usize,
    },
    /// Placeholder for later cycle-guard classification.
    CycleDetected,
}

/// Result alias for normalization requests.
pub type NormalizationResult<T> = Result<T, NormalizationError>;

/// Environment-aware normalizer.
pub struct Normalizer<'env> {
    _env: &'env TypeEnv,
    config: NormalizationConfig,
}

impl<'env> Normalizer<'env> {
    /// Creates a normalizer with default full-normalization options.
    #[must_use]
    pub fn new(env: &'env TypeEnv) -> Self {
        Self::with_config(env, NormalizationConfig::default())
    }

    /// Creates a normalizer with explicit configuration.
    #[must_use]
    pub fn with_config(env: &'env TypeEnv, config: NormalizationConfig) -> Self {
        Self { _env: env, config }
    }

    /// Normalizes a canonical type expression into a normal-form carrier.
    pub fn normalize(&self, expr: &CanonicalTypeExpr) -> NormalizationResult<NormalizationOutcome> {
        let mut state = NormalizationState {
            mode: self.config.mode,
            fuel: self.config.fuel,
            trace_enabled: self.config.trace,
            trace: Vec::new(),
        };
        let (normal, evidence) = state.normalize_expr(expr)?;
        Ok(NormalizationOutcome {
            normal,
            mode: self.config.mode,
            fuel_remaining: state.fuel.remaining(),
            evidence,
            trace: state.trace,
        })
    }

    /// Returns this normalizer's configuration.
    #[must_use]
    pub const fn config(&self) -> &NormalizationConfig {
        &self.config
    }
}

struct NormalizationState {
    mode: NormalizationMode,
    fuel: NormalizationFuel,
    trace_enabled: bool,
    trace: Vec<NormalizationTraceEvent>,
}

impl NormalizationState {
    fn normalize_expr(
        &mut self,
        expr: &CanonicalTypeExpr,
    ) -> NormalizationResult<(NormalTypeExpr, NormalizationEvidence)> {
        self.fuel.consume(self.mode)?;

        let (normal, evidence) = match expr {
            CanonicalTypeExpr::Primitive(name) => (
                NormalTypeExpr::Primitive(name.clone()),
                NormalizationEvidence::StructuralIdentity,
            ),
            CanonicalTypeExpr::Var(name) => (
                NormalTypeExpr::Var(name.clone()),
                NormalizationEvidence::StructuralIdentity,
            ),
            CanonicalTypeExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
            } => (
                NormalTypeExpr::NominalApp {
                    origin: origin.clone(),
                    visible_name: visible_name.clone(),
                    args: self.normalize_args(args)?,
                    kind: kind.clone(),
                },
                NormalizationEvidence::StructuralIdentity,
            ),
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                kind,
                rigidity,
            } => {
                let reason = match rigidity {
                    ProjectionRigidity::Rigid => Some(NormalFormBlockReason::RigidProjection),
                    ProjectionRigidity::Neutral => Some(NormalFormBlockReason::AbstractScrutinee),
                };
                (
                    NormalTypeExpr::Projection {
                        interface: interface.clone(),
                        member: member.clone(),
                        args: self.normalize_args(args)?,
                        kind: kind.clone(),
                        rigidity: *rigidity,
                        reason,
                    },
                    NormalizationEvidence::ProjectionPreserved {
                        rigidity: *rigidity,
                    },
                )
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => (
                NormalTypeExpr::NeutralComputationApp {
                    head: head.clone(),
                    args: self.normalize_args(args)?,
                    kind: kind.clone(),
                    reason: Some(NormalFormBlockReason::Unsupported),
                },
                NormalizationEvidence::NeutralUnsupportedComputation,
            ),
        };

        self.record(evidence.clone());
        Ok((normal, evidence))
    }

    fn normalize_args(
        &mut self,
        args: &[CanonicalTypeExpr],
    ) -> NormalizationResult<Vec<NormalTypeExpr>> {
        args.iter()
            .map(|arg| self.normalize_expr(arg).map(|(normal, _)| normal))
            .collect()
    }

    fn record(&mut self, evidence: NormalizationEvidence) {
        if self.trace_enabled {
            self.trace.push(NormalizationTraceEvent {
                evidence,
                remaining_fuel: self.fuel.remaining(),
            });
        }
    }
}
