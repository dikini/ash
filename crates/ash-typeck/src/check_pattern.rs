//! Pattern type checking for Ash (TASK-128)
//!
//! Provides type checking for patterns in match expressions, including
//! variable binding extraction and type compatibility checking.

#![allow(clippy::result_large_err)]

use crate::solver::TypeError;
use crate::type_env::TypeEnv as CanonicalTypeEnv;
use crate::type_env::{
    PatternCanonicalConstructor, PatternCanonicalType, PatternCanonicalization,
    PatternCanonicalizationBlockedReason, type_expr_to_type,
};
use crate::types::{Type, TypeVar};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{TypeBody, TypeExpr, VariantPayload};
use ash_parser::surface::{Literal, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use std::collections::{HashMap, HashSet};

pub use ash_core::ast::{TypeDef, VariantDef};

/// Bindings from pattern variables to their types
pub type Bindings = HashMap<String, Type>;

/// Type-aware irrefutability result for a checked pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct Irrefutability {
    /// Whether the pattern is total for the scrutinee type.
    pub outcome: IrrefutabilityOutcome,
    /// Typed bindings introduced by the pattern when it is well-typed.
    pub bindings: Bindings,
}

/// Structured outcome of an irrefutability check.
#[derive(Debug, Clone, PartialEq)]
pub enum IrrefutabilityOutcome {
    /// Every value of the scrutinee type matches the pattern.
    Irrefutable,
    /// Some well-typed value of the scrutinee type does not match the pattern.
    Refutable {
        /// A representative non-match witness.
        witness: IrrefutabilityWitness,
    },
    /// The pattern cannot match any well-typed value of the scrutinee type.
    Impossible {
        /// Typed reason the pattern is impossible.
        reason: IrrefutabilityImpossibleReason,
    },
    /// The checker lacks type shape or constructor-universe information.
    Blocked {
        /// Typed reason the check could not produce a closed answer.
        reason: IrrefutabilityBlockedReason,
    },
}

/// Representative witness for a refutable pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum IrrefutabilityWitness {
    /// A pattern-shaped witness using wildcards for irrelevant fields.
    Pattern(Box<Pattern>),
    /// A list with fewer elements than the fixed pattern prefix.
    ShortList {
        /// Minimum element count demanded by the pattern prefix.
        minimum_len: usize,
    },
    /// A literal pattern can be missed by another value of the same non-singleton type.
    NonLiteralValue {
        /// Literal that the witness must differ from.
        literal: Literal,
    },
    /// Conservative human-readable witness for product or future refined cases.
    Description(String),
}

/// Typed reason an irrefutability check is impossible.
#[derive(Debug, Clone, PartialEq)]
pub enum IrrefutabilityImpossibleReason {
    /// A pattern binds the same name more than once.
    DuplicateBinder {
        /// Duplicate binding name.
        name: String,
    },
    /// Existing pattern type compatibility rejected the pattern.
    PatternTypeError(TypeError),
    /// A constructor-specific pattern named a constructor outside the canonical universe.
    UnknownConstructor {
        /// Offending constructor name.
        name: String,
        /// Scrutinee type boundary.
        scrutinee_type: Type,
    },
}

/// Typed reason an irrefutability check is blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrrefutabilityBlockedReason {
    /// Canonical constructor-universe production was blocked.
    Canonicalization {
        /// Source type passed to canonicalization.
        source_type: Type,
        /// Canonicalization blocked reason.
        reason: PatternCanonicalizationBlockedReason,
    },
    /// A tuple or record pattern needs a known product shape.
    ProductShapeUnavailable {
        /// Scrutinee type whose product shape is not known.
        scrutinee_type: Type,
        /// Pattern product shape requested.
        pattern_shape: String,
    },
    /// A variant pattern needs a closed constructor universe.
    ConstructorUniverseUnavailable {
        /// Scrutinee type whose constructor universe is not known.
        scrutinee_type: Type,
    },
}

/// Type environment for pattern checking
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable types in scope
    vars: HashMap<String, Type>,
    /// Type definitions (for variant checking)
    type_defs: HashMap<String, TypeDef>,
    /// Type definition insertion order for rebuilding the canonical type environment.
    type_def_order: Vec<String>,
}

impl TypeEnv {
    /// Create a new empty type environment
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            type_defs: HashMap::new(),
            type_def_order: Vec::new(),
        }
    }

    /// Add a variable binding to the environment
    pub fn bind_var(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    /// Look up a variable's type
    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    /// Add a type definition
    pub fn add_type_def(&mut self, name: String, def: TypeDef) {
        if !self.type_defs.contains_key(&name) {
            self.type_def_order.push(name.clone());
        }
        self.type_defs.insert(name, def);
    }

    /// Look up a type definition
    pub fn lookup_type_def(&self, name: &str) -> Option<&TypeDef> {
        self.type_defs.get(name)
    }

    fn lookup_variant(
        &self,
        variant_name: &str,
        field_patterns: Option<&[(Box<str>, Pattern)]>,
        payload: &VariantPatternPayload,
    ) -> Result<Option<(&TypeDef, &VariantDef)>, TypeError> {
        let named_matches: Vec<(&TypeDef, &VariantDef)> = self
            .type_defs
            .values()
            .flat_map(|type_def| match &type_def.body {
                TypeBody::Enum(variants) => variants
                    .iter()
                    .map(move |variant| (type_def, variant))
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .filter(|(_, variant)| variant.name == variant_name)
            .collect();

        match named_matches.as_slice() {
            [] => Ok(None),
            [(type_def, variant)] => Ok(Some((*type_def, *variant))),
            _ => {
                let requested_fields: Option<Vec<String>> = match payload {
                    VariantPatternPayload::Tuple(items) => Some(
                        items
                            .iter()
                            .enumerate()
                            .map(|(index, _)| tuple_field_name(index))
                            .collect(),
                    ),
                    VariantPatternPayload::Unit => None,
                    VariantPatternPayload::Record(_) => field_patterns.map(|patterns| {
                        patterns
                            .iter()
                            .map(|(field_name, _)| field_name.to_string())
                            .collect()
                    }),
                };
                let mut disambiguated = named_matches.into_iter().filter(|(_, variant)| {
                    requested_fields.as_ref().is_some_and(|requested_fields| {
                        requested_fields.iter().all(|requested| {
                            variant.fields.iter().any(|(name, _)| name == requested)
                        })
                    })
                });

                let first = disambiguated.next();
                let second = disambiguated.next();

                match (first, second) {
                    (Some((type_def, variant)), None) => Ok(Some((type_def, variant))),
                    _ => Err(TypeError::InvalidPattern {
                        message: format!("ambiguous variant: {variant_name}"),
                        span: Span::default(),
                    }),
                }
            }
        }
    }

    fn canonical_env_for_registered_types(&self) -> Result<CanonicalTypeEnv, TypeError> {
        let mut env = CanonicalTypeEnv::new();
        for name in &self.type_def_order {
            if let Some(type_def) = self.type_defs.get(name) {
                env.register_type(type_def).map_err(TypeError::from)?;
            }
        }
        let mut remaining = self
            .type_defs
            .iter()
            .filter(|(name, _)| !self.type_def_order.contains(name))
            .map(|(_, type_def)| type_def)
            .collect::<Vec<_>>();
        remaining.sort_by(|left, right| left.name.cmp(&right.name));
        for type_def in remaining {
            env.register_type(type_def).map_err(TypeError::from)?;
        }
        Ok(env)
    }

    fn lower_type_expr(&self, owner: Option<&TypeDef>, expr: &TypeExpr) -> Result<Type, TypeError> {
        let canonical_env = self.canonical_env_for_registered_types()?;
        let param_mapping = owner
            .map(|type_def| {
                type_def
                    .params
                    .iter()
                    .map(|name| (name.clone(), TypeVar::fresh()))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
        type_expr_to_type(expr, &param_mapping, &canonical_env)
    }

    fn lower_type_expr_for_owner_type(
        &self,
        owner: &TypeDef,
        owner_type: Option<&Type>,
        expr: &TypeExpr,
    ) -> Result<Type, TypeError> {
        let Some(param_substitution) = owner_type_arg_substitution(owner, owner_type) else {
            return self.lower_type_expr(Some(owner), expr);
        };

        let canonical_env = self.canonical_env_for_registered_types()?;
        type_expr_to_type_with_substitution(expr, &param_substitution, &canonical_env)
    }
}

/// Type check a pattern against an expected type
///
/// Returns the bindings from pattern variables to their types.
///
/// # Arguments
/// * `env` - The type environment
/// * `pattern` - The pattern to check
/// * `expected` - The expected type the pattern should match
///
/// # Returns
/// * `Ok(Bindings)` - Variable bindings from the pattern
/// * `Err(TypeError)` - Type error if pattern doesn't match
///
/// # Examples
///
/// ```
/// use ash_typeck::check_pattern::{check_pattern, TypeEnv};
/// use ash_typeck::types::Type;
/// use ash_parser::surface::Pattern;
///
/// let env = TypeEnv::new();
/// let pattern = Pattern::Variable { name: "x".into(), span: ash_parser::token::Span::default() };
/// let expected = Type::Int;
///
/// let bindings = check_pattern(&env, &pattern, &expected).unwrap();
/// assert_eq!(bindings.get("x"), Some(&Type::Int));
/// ```
pub fn check_pattern(
    env: &TypeEnv,
    pattern: &Pattern,
    expected: &Type,
) -> Result<Bindings, TypeError> {
    let mut bindings = Bindings::new();
    check_pattern_inner(env, pattern, expected, &mut bindings)?;
    Ok(bindings)
}

/// Type check a pattern against a TASK-913 canonical ADT constructor universe.
///
/// This entrypoint resolves variant names only within `canonical.constructors`,
/// so aliases can match their canonical ADT while unrelated visible
/// constructors cannot leak in through global name lookup.
pub fn check_pattern_with_canonical_type(
    env: &TypeEnv,
    pattern: &Pattern,
    canonical: &PatternCanonicalType,
) -> Result<Bindings, TypeError> {
    let mut bindings = Bindings::new();
    check_pattern_inner_with_canonical(env, pattern, canonical, &mut bindings)?;
    Ok(bindings)
}

/// Check whether a surface pattern is irrefutable for a scrutinee type.
///
/// This API is intentionally diagnostic-oriented: it distinguishes refutable
/// patterns, impossible/type-error patterns, and checks blocked by missing type
/// shape information. It does not enforce any binder callsite policy by itself.
#[must_use]
pub fn check_irrefutable_pattern(
    env: &TypeEnv,
    pattern: &Pattern,
    scrutinee_type: &Type,
) -> Irrefutability {
    check_irrefutable_pattern_inner_public(env, pattern, scrutinee_type, None)
}

/// Check whether a surface pattern is irrefutable for a canonical ADT boundary.
#[must_use]
pub fn check_irrefutable_pattern_with_canonical_type(
    env: &TypeEnv,
    pattern: &Pattern,
    canonical: &PatternCanonicalType,
) -> Irrefutability {
    check_irrefutable_pattern_inner_public(env, pattern, &canonical.source_type, Some(canonical))
}

/// Check irrefutability using a precomputed pattern canonicalization result.
///
/// Universal patterns remain irrefutable even when canonicalization is blocked.
/// Non-universal constructor patterns report the blocked reason instead of
/// guessing a constructor universe.
#[must_use]
pub fn check_irrefutable_pattern_with_canonicalization(
    env: &TypeEnv,
    pattern: &Pattern,
    scrutinee_type: &Type,
    canonicalization: &PatternCanonicalization,
) -> Irrefutability {
    if let Some(name) = duplicate_binder(pattern) {
        return Irrefutability {
            outcome: IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::DuplicateBinder { name },
            },
            bindings: Bindings::new(),
        };
    }

    if is_universal_pattern(pattern) {
        return check_irrefutable_pattern(env, pattern, scrutinee_type);
    }

    match canonicalization {
        PatternCanonicalization::Matchable(canonical) => {
            check_irrefutable_pattern_with_canonical_type(env, pattern, canonical)
        }
        PatternCanonicalization::Blocked {
            source_type,
            reason,
        } if matches!(pattern, Pattern::Variant { .. }) => Irrefutability {
            outcome: IrrefutabilityOutcome::Blocked {
                reason: IrrefutabilityBlockedReason::Canonicalization {
                    source_type: source_type.clone(),
                    reason: reason.clone(),
                },
            },
            bindings: Bindings::new(),
        },
        PatternCanonicalization::Blocked { .. } => {
            check_irrefutable_pattern(env, pattern, scrutinee_type)
        }
    }
}

fn check_irrefutable_pattern_inner_public(
    env: &TypeEnv,
    pattern: &Pattern,
    scrutinee_type: &Type,
    canonical: Option<&PatternCanonicalType>,
) -> Irrefutability {
    if let Some(name) = duplicate_binder(pattern) {
        return Irrefutability {
            outcome: IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::DuplicateBinder { name },
            },
            bindings: Bindings::new(),
        };
    }

    let mut bindings = Bindings::new();
    let outcome = irrefutable_pattern_inner(env, pattern, scrutinee_type, canonical, &mut bindings);
    if matches!(
        outcome,
        IrrefutabilityOutcome::Impossible { .. } | IrrefutabilityOutcome::Blocked { .. }
    ) {
        bindings.clear();
    }

    Irrefutability { outcome, bindings }
}

fn irrefutable_pattern_inner(
    env: &TypeEnv,
    pattern: &Pattern,
    scrutinee_type: &Type,
    canonical: Option<&PatternCanonicalType>,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    match pattern {
        Pattern::Wildcard => IrrefutabilityOutcome::Irrefutable,
        Pattern::Variable { name, .. } => {
            bindings.insert(name.to_string(), scrutinee_type.clone());
            IrrefutabilityOutcome::Irrefutable
        }
        Pattern::Literal(literal) => {
            match check_pattern_inner(env, pattern, scrutinee_type, bindings) {
                Ok(()) => IrrefutabilityOutcome::Refutable {
                    witness: IrrefutabilityWitness::NonLiteralValue {
                        literal: literal.clone(),
                    },
                },
                Err(error) => IrrefutabilityOutcome::Impossible {
                    reason: IrrefutabilityImpossibleReason::PatternTypeError(error),
                },
            }
        }
        Pattern::List { elements, rest } => {
            irrefutable_list_pattern(env, elements, rest.as_deref(), scrutinee_type, bindings)
        }
        Pattern::Tuple(patterns) => {
            irrefutable_tuple_pattern(env, patterns, scrutinee_type, bindings)
        }
        Pattern::Record(fields) => {
            irrefutable_record_pattern(env, fields, scrutinee_type, bindings)
        }
        Pattern::Variant {
            name,
            fields,
            payload,
        } => irrefutable_variant_pattern(
            env,
            name,
            fields.as_deref(),
            payload,
            scrutinee_type,
            canonical,
            bindings,
        ),
    }
}

fn irrefutable_list_pattern(
    env: &TypeEnv,
    elements: &[Pattern],
    rest: Option<&str>,
    scrutinee_type: &Type,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    let element_type = match scrutinee_type {
        Type::List(element_type) => element_type.as_ref().clone(),
        Type::Var(_) => Type::Var(TypeVar::fresh()),
        _ => {
            return impossible_from_pattern_error(
                env,
                &Pattern::List {
                    elements: elements.to_vec(),
                    rest: rest.map(Into::into),
                },
                scrutinee_type,
            );
        }
    };

    for element in elements {
        match irrefutable_pattern_inner(env, element, &element_type, None, bindings) {
            IrrefutabilityOutcome::Irrefutable | IrrefutabilityOutcome::Refutable { .. } => {}
            other => return other,
        }
    }

    if let Some(rest_name) = rest {
        bindings.insert(rest_name.to_string(), Type::List(Box::new(element_type)));
    }

    if elements.is_empty() {
        return if rest.is_some() {
            IrrefutabilityOutcome::Irrefutable
        } else {
            IrrefutabilityOutcome::Refutable {
                witness: IrrefutabilityWitness::Description("non-empty list".to_string()),
            }
        };
    }

    IrrefutabilityOutcome::Refutable {
        witness: IrrefutabilityWitness::ShortList {
            minimum_len: elements.len(),
        },
    }
}

fn irrefutable_tuple_pattern(
    env: &TypeEnv,
    patterns: &[Pattern],
    scrutinee_type: &Type,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    let Type::Record(fields) = scrutinee_type else {
        if matches!(scrutinee_type, Type::Var(_)) {
            return IrrefutabilityOutcome::Blocked {
                reason: IrrefutabilityBlockedReason::ProductShapeUnavailable {
                    scrutinee_type: scrutinee_type.clone(),
                    pattern_shape: "tuple".to_string(),
                },
            };
        }
        return impossible_from_pattern_error(
            env,
            &Pattern::Tuple(patterns.to_vec()),
            scrutinee_type,
        );
    };

    if patterns.len() != fields.len() {
        return IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::PatternTypeError(
                TypeError::PatternArityMismatch {
                    expected: fields.len(),
                    actual: patterns.len(),
                    span: Span::default(),
                },
            ),
        };
    }

    let mut refutable = None;
    for (index, pattern) in patterns.iter().enumerate() {
        let Some(field_type) = tuple_pattern_field_type(fields, index) else {
            return IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::PatternTypeError(
                    TypeError::PatternArityMismatch {
                        expected: fields.len(),
                        actual: patterns.len(),
                        span: Span::default(),
                    },
                ),
            };
        };

        match irrefutable_pattern_inner(env, pattern, field_type, None, bindings) {
            IrrefutabilityOutcome::Irrefutable => {}
            IrrefutabilityOutcome::Refutable { witness } => {
                refutable.get_or_insert((index, witness));
            }
            other => return other,
        }
    }

    refutable.map_or(IrrefutabilityOutcome::Irrefutable, |(index, witness)| {
        IrrefutabilityOutcome::Refutable {
            witness: lift_tuple_witness(patterns.len(), index, witness),
        }
    })
}

fn irrefutable_record_pattern(
    env: &TypeEnv,
    field_patterns: &[(Box<str>, Pattern)],
    scrutinee_type: &Type,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    let Type::Record(fields) = scrutinee_type else {
        if matches!(scrutinee_type, Type::Var(_)) {
            return IrrefutabilityOutcome::Blocked {
                reason: IrrefutabilityBlockedReason::ProductShapeUnavailable {
                    scrutinee_type: scrutinee_type.clone(),
                    pattern_shape: "record".to_string(),
                },
            };
        }
        return impossible_from_pattern_error(
            env,
            &Pattern::Record(field_patterns.to_vec()),
            scrutinee_type,
        );
    };

    let mut refutable = None;
    for (field_name, field_pattern) in field_patterns {
        let Some((_, field_type)) = fields
            .iter()
            .find(|(name, _)| name.as_ref() == field_name.as_ref())
        else {
            return IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::PatternTypeError(
                    TypeError::InvalidPattern {
                        message: format!("unknown field: {field_name}"),
                        span: Span::default(),
                    },
                ),
            };
        };

        match irrefutable_pattern_inner(env, field_pattern, field_type, None, bindings) {
            IrrefutabilityOutcome::Irrefutable => {}
            IrrefutabilityOutcome::Refutable { witness } => {
                refutable.get_or_insert((field_name.to_string(), witness));
            }
            other => return other,
        }
    }

    refutable.map_or(
        IrrefutabilityOutcome::Irrefutable,
        |(field_name, witness)| IrrefutabilityOutcome::Refutable {
            witness: lift_record_witness(fields, &field_name, witness),
        },
    )
}

fn irrefutable_variant_pattern(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    scrutinee_type: &Type,
    canonical: Option<&PatternCanonicalType>,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    let owned_canonical;
    let canonical = match canonical {
        Some(canonical) => canonical,
        None => match canonicalize_type_from_pattern_env(env, scrutinee_type) {
            Ok(PatternCanonicalization::Matchable(canonical)) => {
                owned_canonical = canonical;
                &owned_canonical
            }
            Ok(PatternCanonicalization::Blocked { .. })
                if matches!(scrutinee_type, Type::Var(_)) =>
            {
                return IrrefutabilityOutcome::Blocked {
                    reason: IrrefutabilityBlockedReason::ConstructorUniverseUnavailable {
                        scrutinee_type: scrutinee_type.clone(),
                    },
                };
            }
            Ok(PatternCanonicalization::Blocked {
                source_type,
                reason:
                    reason @ (PatternCanonicalizationBlockedReason::RigidAssociatedProjection { .. }
                    | PatternCanonicalizationBlockedReason::ConstructorVariableApplication {
                        ..
                    }
                    | PatternCanonicalizationBlockedReason::NonConcreteTypeArgument),
            }) => {
                return IrrefutabilityOutcome::Blocked {
                    reason: IrrefutabilityBlockedReason::Canonicalization {
                        source_type,
                        reason,
                    },
                };
            }
            Ok(PatternCanonicalization::Blocked { .. }) => {
                return impossible_from_pattern_error(
                    env,
                    &Pattern::Variant {
                        name: variant_name.into(),
                        fields: field_patterns.map(<[_]>::to_vec),
                        payload: payload.clone(),
                    },
                    scrutinee_type,
                );
            }
            Err(error) => {
                return IrrefutabilityOutcome::Impossible {
                    reason: IrrefutabilityImpossibleReason::PatternTypeError(error),
                };
            }
        },
    };

    let Some(constructor) = canonical
        .constructors
        .iter()
        .find(|constructor| constructor.name == variant_name)
    else {
        return IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::UnknownConstructor {
                name: variant_name.to_string(),
                scrutinee_type: canonical.source_type.clone(),
            },
        };
    };

    match irrefutable_canonical_variant_fields(
        env,
        variant_name,
        field_patterns,
        payload,
        constructor,
        bindings,
    ) {
        IrrefutabilityOutcome::Irrefutable => {}
        nested @ IrrefutabilityOutcome::Refutable { .. } => return nested,
        other => return other,
    }

    if canonical.constructors.len() == 1 {
        IrrefutabilityOutcome::Irrefutable
    } else {
        let witness = canonical
            .constructors
            .iter()
            .find(|constructor| constructor.name != variant_name)
            .map(constructor_witness_pattern)
            .map(|pattern| IrrefutabilityWitness::Pattern(Box::new(pattern)))
            .unwrap_or_else(|| {
                IrrefutabilityWitness::Description(format!(
                    "non-{variant_name} constructor witness"
                ))
            });
        IrrefutabilityOutcome::Refutable { witness }
    }
}

fn irrefutable_canonical_variant_fields(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    constructor: &PatternCanonicalConstructor,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    match payload {
        VariantPatternPayload::Unit => {
            if constructor.payload_shape == VariantPayloadShape::Unit {
                return IrrefutabilityOutcome::Irrefutable;
            }
            IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::PatternTypeError(
                    TypeError::InvalidPattern {
                        message: format!("variant {variant_name} does not have unit payload"),
                        span: Span::default(),
                    },
                ),
            }
        }
        VariantPatternPayload::Record(record_fields) => irrefutable_record_variant_fields(
            env,
            variant_name,
            field_patterns.unwrap_or(record_fields.as_slice()),
            constructor,
            bindings,
        ),
        VariantPatternPayload::Tuple(items) => {
            irrefutable_tuple_variant_fields(env, variant_name, items, constructor, bindings)
        }
    }
}

fn irrefutable_record_variant_fields(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: &[(Box<str>, Pattern)],
    constructor: &PatternCanonicalConstructor,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    if constructor.payload_shape != VariantPayloadShape::Record {
        return IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::PatternTypeError(TypeError::InvalidPattern {
                message: format!("variant {variant_name} does not have record payload"),
                span: Span::default(),
            }),
        };
    }

    let mut refutable = None;
    for (field_name, field_pattern) in field_patterns {
        let Some((_, field_type)) = constructor
            .fields
            .iter()
            .find(|(name, _)| name == field_name.as_ref())
        else {
            return IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::PatternTypeError(
                    TypeError::InvalidPattern {
                        message: format!("unknown field: {field_name}"),
                        span: Span::default(),
                    },
                ),
            };
        };

        match irrefutable_pattern_inner(env, field_pattern, field_type, None, bindings) {
            IrrefutabilityOutcome::Irrefutable => {}
            IrrefutabilityOutcome::Refutable { witness } => {
                refutable.get_or_insert((field_name.to_string(), witness));
            }
            other => return other,
        }
    }

    refutable.map_or(
        IrrefutabilityOutcome::Irrefutable,
        |(field_name, witness)| IrrefutabilityOutcome::Refutable {
            witness: lift_variant_witness(constructor, &field_name, witness),
        },
    )
}

fn irrefutable_tuple_variant_fields(
    env: &TypeEnv,
    variant_name: &str,
    items: &[Pattern],
    constructor: &PatternCanonicalConstructor,
    bindings: &mut Bindings,
) -> IrrefutabilityOutcome {
    if constructor.payload_shape != VariantPayloadShape::Tuple {
        return IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::PatternTypeError(TypeError::InvalidPattern {
                message: format!("variant {variant_name} does not have tuple payload"),
                span: Span::default(),
            }),
        };
    }

    if items.len() != constructor.fields.len() {
        return IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::PatternTypeError(TypeError::InvalidPattern {
                message: format!(
                    "tuple variant {variant_name} expects {} positional items, got {}",
                    constructor.fields.len(),
                    items.len()
                ),
                span: Span::default(),
            }),
        };
    }

    let mut refutable = None;
    for (index, item) in items.iter().enumerate() {
        let Some((_, field_type)) = constructor
            .fields
            .iter()
            .find(|(name, _)| name == &tuple_field_name(index))
            .or_else(|| constructor.fields.get(index))
        else {
            return IrrefutabilityOutcome::Impossible {
                reason: IrrefutabilityImpossibleReason::PatternTypeError(
                    TypeError::InvalidPattern {
                        message: format!(
                            "tuple variant {variant_name} is missing positional slot {index}"
                        ),
                        span: Span::default(),
                    },
                ),
            };
        };

        match irrefutable_pattern_inner(env, item, field_type, None, bindings) {
            IrrefutabilityOutcome::Irrefutable => {}
            IrrefutabilityOutcome::Refutable { witness } => {
                refutable.get_or_insert((index, witness));
            }
            other => return other,
        }
    }

    refutable.map_or(IrrefutabilityOutcome::Irrefutable, |(index, witness)| {
        IrrefutabilityOutcome::Refutable {
            witness: lift_variant_witness(constructor, &tuple_field_name(index), witness),
        }
    })
}

fn impossible_from_pattern_error(
    env: &TypeEnv,
    pattern: &Pattern,
    scrutinee_type: &Type,
) -> IrrefutabilityOutcome {
    match check_pattern(env, pattern, scrutinee_type) {
        Ok(_) => IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::PatternTypeError(TypeError::InvalidPattern {
                message: "pattern shape is impossible for irrefutability classification"
                    .to_string(),
                span: Span::default(),
            }),
        },
        Err(error) => IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::PatternTypeError(error),
        },
    }
}

fn is_universal_pattern(pattern: &Pattern) -> bool {
    matches!(pattern, Pattern::Wildcard | Pattern::Variable { .. })
}

fn duplicate_binder(pattern: &Pattern) -> Option<String> {
    fn visit(pattern: &Pattern, seen: &mut HashSet<String>) -> Option<String> {
        match pattern {
            Pattern::Variable { name, .. } => {
                let name = name.to_string();
                if seen.insert(name.clone()) {
                    None
                } else {
                    Some(name)
                }
            }
            Pattern::Wildcard | Pattern::Literal(_) => None,
            Pattern::Tuple(patterns) => patterns.iter().find_map(|pattern| visit(pattern, seen)),
            Pattern::Record(fields) => fields.iter().find_map(|(_, pattern)| visit(pattern, seen)),
            Pattern::List { elements, rest } => {
                for element in elements {
                    if let Some(duplicate) = visit(element, seen) {
                        return Some(duplicate);
                    }
                }
                rest.as_ref().and_then(|name| {
                    let name = name.to_string();
                    if seen.insert(name.clone()) {
                        None
                    } else {
                        Some(name)
                    }
                })
            }
            Pattern::Variant {
                fields, payload, ..
            } => match payload {
                VariantPatternPayload::Tuple(items) => {
                    items.iter().find_map(|pattern| visit(pattern, seen))
                }
                VariantPatternPayload::Unit => None,
                VariantPatternPayload::Record(record_fields) => {
                    let fields = fields.as_deref().unwrap_or(record_fields);
                    fields.iter().find_map(|(_, pattern)| visit(pattern, seen))
                }
            },
        }
    }

    visit(pattern, &mut HashSet::new())
}

fn tuple_pattern_field_type(fields: &[(Box<str>, Type)], index: usize) -> Option<&Type> {
    let decimal = index.to_string();
    let underscored = format!("_{index}");
    fields
        .iter()
        .find(|(name, _)| name.as_ref() == decimal || name.as_ref() == underscored)
        .map(|(_, ty)| ty)
}

fn constructor_witness_pattern(constructor: &PatternCanonicalConstructor) -> Pattern {
    let fields = match constructor.payload_shape {
        VariantPayloadShape::Unit => None,
        VariantPayloadShape::Record => Some(
            constructor
                .fields
                .iter()
                .map(|(field_name, _)| (field_name.clone().into_boxed_str(), Pattern::Wildcard))
                .collect(),
        ),
        VariantPayloadShape::Tuple => Some(
            constructor
                .fields
                .iter()
                .enumerate()
                .map(|(index, _)| (tuple_field_name(index).into_boxed_str(), Pattern::Wildcard))
                .collect(),
        ),
    };

    let payload = match constructor.payload_shape {
        VariantPayloadShape::Unit => VariantPatternPayload::Unit,
        VariantPayloadShape::Record => {
            VariantPatternPayload::Record(fields.clone().unwrap_or_default())
        }
        VariantPayloadShape::Tuple => VariantPatternPayload::Tuple(
            constructor
                .fields
                .iter()
                .map(|_| Pattern::Wildcard)
                .collect(),
        ),
    };

    Pattern::Variant {
        name: constructor.name.clone().into_boxed_str(),
        fields,
        payload,
    }
}

fn lift_tuple_witness(
    len: usize,
    index: usize,
    witness: IrrefutabilityWitness,
) -> IrrefutabilityWitness {
    witness_to_pattern(witness).map_or_else(
        || IrrefutabilityWitness::Description(format!("tuple field {index} does not match")),
        |field_pattern| {
            let mut fields = vec![Pattern::Wildcard; len];
            if let Some(slot) = fields.get_mut(index) {
                *slot = field_pattern;
            }
            IrrefutabilityWitness::Pattern(Box::new(Pattern::Tuple(fields)))
        },
    )
}

fn lift_record_witness(
    fields: &[(Box<str>, Type)],
    field_name: &str,
    witness: IrrefutabilityWitness,
) -> IrrefutabilityWitness {
    witness_to_pattern(witness).map_or_else(
        || IrrefutabilityWitness::Description(format!("record field {field_name} does not match")),
        |field_pattern| {
            IrrefutabilityWitness::Pattern(Box::new(Pattern::Record(
                fields
                    .iter()
                    .map(|(name, _)| {
                        if name.as_ref() == field_name {
                            (name.clone(), field_pattern.clone())
                        } else {
                            (name.clone(), Pattern::Wildcard)
                        }
                    })
                    .collect(),
            )))
        },
    )
}

fn lift_variant_witness(
    constructor: &PatternCanonicalConstructor,
    field_name: &str,
    witness: IrrefutabilityWitness,
) -> IrrefutabilityWitness {
    witness_to_pattern(witness).map_or_else(
        || {
            IrrefutabilityWitness::Description(format!(
                "variant {} field {field_name} does not match",
                constructor.name
            ))
        },
        |field_pattern| {
            let payload = match constructor.payload_shape {
                VariantPayloadShape::Unit => VariantPatternPayload::Unit,
                VariantPayloadShape::Record => VariantPatternPayload::Record(
                    constructor
                        .fields
                        .iter()
                        .map(|(name, _)| {
                            if name == field_name {
                                (name.clone().into_boxed_str(), field_pattern.clone())
                            } else {
                                (name.clone().into_boxed_str(), Pattern::Wildcard)
                            }
                        })
                        .collect(),
                ),
                VariantPayloadShape::Tuple => VariantPatternPayload::Tuple(
                    constructor
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, (name, _))| {
                            if name == field_name || tuple_field_name(index) == field_name {
                                field_pattern.clone()
                            } else {
                                Pattern::Wildcard
                            }
                        })
                        .collect(),
                ),
            };
            let fields = match &payload {
                VariantPatternPayload::Unit => None,
                VariantPatternPayload::Record(fields) => Some(fields.clone()),
                VariantPatternPayload::Tuple(items) => Some(
                    items
                        .iter()
                        .enumerate()
                        .map(|(index, pattern)| {
                            (tuple_field_name(index).into_boxed_str(), pattern.clone())
                        })
                        .collect(),
                ),
            };

            IrrefutabilityWitness::Pattern(Box::new(Pattern::Variant {
                name: constructor.name.clone().into_boxed_str(),
                fields,
                payload,
            }))
        },
    )
}

fn witness_to_pattern(witness: IrrefutabilityWitness) -> Option<Pattern> {
    match witness {
        IrrefutabilityWitness::Pattern(pattern) => Some(*pattern),
        IrrefutabilityWitness::NonLiteralValue { literal } => non_literal_witness_pattern(literal),
        IrrefutabilityWitness::ShortList { minimum_len } if minimum_len > 0 => {
            Some(Pattern::List {
                elements: Vec::new(),
                rest: None,
            })
        }
        IrrefutabilityWitness::ShortList { .. } | IrrefutabilityWitness::Description(_) => None,
    }
}

fn non_literal_witness_pattern(literal: Literal) -> Option<Pattern> {
    match literal {
        Literal::Int(value) => Some(Pattern::Literal(Literal::Int(value.saturating_add(1)))),
        Literal::Float(value) => Some(Pattern::Literal(Literal::Float(
            ordered_float::OrderedFloat(value.0 + 1.0),
        ))),
        Literal::String(value) => Some(Pattern::Literal(Literal::String(
            format!("{value}#").into_boxed_str(),
        ))),
        Literal::Bool(value) => Some(Pattern::Literal(Literal::Bool(!value))),
        Literal::Null | Literal::List(_) => None,
    }
}

fn canonicalize_type_from_pattern_env(
    env: &TypeEnv,
    scrutinee_type: &Type,
) -> Result<PatternCanonicalization, TypeError> {
    let canonical_env = env.canonical_env_for_registered_types()?;
    Ok(canonical_env.canonicalize_type_for_pattern(scrutinee_type))
}

fn owner_type_arg_substitution(
    owner: &TypeDef,
    owner_type: Option<&Type>,
) -> Option<HashMap<String, Type>> {
    let Type::Constructor { name, args, .. } = owner_type? else {
        return None;
    };

    if name.name != owner.name || args.len() != owner.params.len() {
        return None;
    }

    Some(
        owner
            .params
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect(),
    )
}

fn type_expr_to_type_with_substitution(
    expr: &TypeExpr,
    substitutions: &HashMap<String, Type>,
    type_env: &CanonicalTypeEnv,
) -> Result<Type, TypeError> {
    if !type_expr_mentions_substituted_param(expr, substitutions) {
        return type_expr_to_type(expr, &HashMap::new(), type_env);
    }

    match expr {
        TypeExpr::Named(name) => substitutions
            .get(name)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| type_expr_to_type(expr, &HashMap::new(), type_env)),
        TypeExpr::Constructor { name, args } if name == "Fn" => {
            let mut arg_types = args
                .iter()
                .map(|arg| type_expr_to_type_with_substitution(arg, substitutions, type_env))
                .collect::<Result<Vec<_>, _>>()?;
            let ret = arg_types
                .pop()
                .ok_or_else(|| TypeError::ConstructorArityMismatch {
                    name: "Fn".to_string(),
                    expected_arity: 1,
                    found_arity: 0,
                    span: Span::default(),
                })?;
            Ok(Type::Fn(arg_types, Box::new(ret)))
        }
        TypeExpr::Constructor { name, args } => {
            let (qualified, _) = type_env.resolve_type(name)?;
            type_env.check_type_constructor_arity(&qualified, args.len())?;
            let arg_types = args
                .iter()
                .map(|arg| type_expr_to_type_with_substitution(arg, substitutions, type_env))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Constructor {
                name: qualified,
                args: arg_types,
                kind: crate::Kind::Type,
            })
        }
        TypeExpr::Tuple(items) => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                type_expr_to_type_with_substitution(item, substitutions, type_env)
                    .map(|ty| (format!("_{index}").into_boxed_str(), ty))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Type::Record),
        TypeExpr::Record(fields) => fields
            .iter()
            .map(|(name, field_type)| {
                type_expr_to_type_with_substitution(field_type, substitutions, type_env)
                    .map(|ty| (name.clone().into_boxed_str(), ty))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Type::Record),
        TypeExpr::Associated { .. } => type_expr_to_type(expr, &HashMap::new(), type_env),
    }
}

fn type_expr_mentions_substituted_param(
    expr: &TypeExpr,
    substitutions: &HashMap<String, Type>,
) -> bool {
    match expr {
        TypeExpr::Named(name) => substitutions.contains_key(name),
        TypeExpr::Constructor { args, .. } | TypeExpr::Tuple(args) => args
            .iter()
            .any(|arg| type_expr_mentions_substituted_param(arg, substitutions)),
        TypeExpr::Record(fields) => fields
            .iter()
            .any(|(_, ty)| type_expr_mentions_substituted_param(ty, substitutions)),
        TypeExpr::Associated { base, .. } => {
            type_expr_mentions_substituted_param(base, substitutions)
        }
    }
}

fn check_pattern_inner_with_canonical(
    env: &TypeEnv,
    pattern: &Pattern,
    canonical: &PatternCanonicalType,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match pattern {
        Pattern::Variant {
            name,
            fields,
            payload,
        } => check_variant_pattern_with_canonical(
            env,
            name,
            fields.as_deref(),
            payload,
            canonical,
            bindings,
        ),
        _ => check_pattern_inner(env, pattern, &canonical.source_type, bindings),
    }
}

fn check_variant_pattern_with_canonical(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    canonical: &PatternCanonicalType,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    let variant = canonical
        .constructors
        .iter()
        .find(|constructor| constructor.name == variant_name)
        .ok_or_else(|| TypeError::UnknownVariantForCanonicalType {
            variant: variant_name.to_string(),
            canonical_type: Box::new(canonical.canonical_type.clone()),
            source_type: Box::new(canonical.source_type.clone()),
            span: Span::default(),
        })?;

    check_canonical_variant_fields(
        env,
        variant_name,
        field_patterns,
        payload,
        variant,
        bindings,
    )
}

/// Inner recursive pattern checking function
fn check_pattern_inner(
    env: &TypeEnv,
    pattern: &Pattern,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match pattern {
        // Wildcard matches anything, no bindings
        Pattern::Wildcard => Ok(()),

        // Variable binds to the expected type
        Pattern::Variable { name, .. } => {
            bindings.insert(name.to_string(), expected.clone());
            Ok(())
        }

        // Literal must match the literal type
        Pattern::Literal(lit) => {
            let lit_type = literal_to_type(lit);
            if types_compatible(expected, &lit_type) {
                Ok(())
            } else {
                Err(TypeError::PatternMismatch {
                    expected: Box::new(expected.clone()),
                    actual: Box::new(lit_type),
                    span: Span::default(),
                })
            }
        }

        // Variant pattern: check variant exists and field patterns match
        Pattern::Variant {
            name,
            fields,
            payload,
        } => check_variant_pattern(env, name, fields.as_deref(), payload, expected, bindings),

        // Tuple pattern: check element count and types
        Pattern::Tuple(patterns) => check_tuple_pattern(env, patterns, expected, bindings),

        // Record pattern: check field names and types
        Pattern::Record(field_patterns) => {
            check_record_pattern(env, field_patterns, expected, bindings)
        }

        // List pattern: check element patterns
        Pattern::List { elements, rest } => check_list_pattern(
            env,
            elements,
            rest.as_ref().map(|v| v.as_ref()),
            expected,
            bindings,
        ),
    }
}

/// Convert a literal to its type
fn literal_to_type(lit: &Literal) -> Type {
    match lit {
        Literal::Int(_) => Type::Int,
        Literal::Float(_) => Type::Float,
        Literal::String(_) => Type::String,
        Literal::Bool(_) => Type::Bool,
        Literal::Null => Type::Null,
        Literal::List(_) => Type::List(Box::new(Type::Var(TypeVar::fresh()))),
    }
}

/// Check if two types are compatible
fn types_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        // Same types are compatible
        (t1, t2) if t1 == t2 => true,
        // Type variables are compatible with anything
        (Type::Var(_), _) => true,
        (_, Type::Var(_)) => true,
        // Lists are compatible if elements are
        (Type::List(e1), Type::List(a1)) => types_compatible(e1, a1),
        // Otherwise not compatible
        _ => false,
    }
}

/// Check a variant pattern against a type
fn check_variant_pattern(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    if matches!(expected, Type::Constructor { .. }) {
        let canonical_result = match canonicalize_type_from_pattern_env(env, expected)? {
            PatternCanonicalization::Matchable(canonical) => {
                let mut canonical_bindings = bindings.clone();
                Some(
                    check_variant_pattern_with_canonical(
                        env,
                        variant_name,
                        field_patterns,
                        payload,
                        &canonical,
                        &mut canonical_bindings,
                    )
                    .map(|()| canonical_bindings),
                )
            }
            PatternCanonicalization::Blocked { .. } => None,
        };

        if let Some(Err(error)) = canonical_result.as_ref()
            && !is_canonical_result_deferred_to_registered_variant(error)
        {
            return Err(error.clone());
        }

        if let Some(registered_bindings) = try_registered_variant_pattern(
            env,
            variant_name,
            field_patterns,
            payload,
            expected,
            bindings,
        )? {
            *bindings = registered_bindings;
            return Ok(());
        }

        if let Some(result) = canonical_result {
            *bindings = result?;
            return Ok(());
        }
    }

    if !matches!(expected, Type::Var(_)) {
        return Err(TypeError::PatternMismatch {
            expected: Box::new(expected.clone()),
            actual: Box::new(Type::Var(TypeVar::fresh())),
            span: Span::default(),
        });
    }

    if let Some((owner, variant_def)) = env.lookup_variant(variant_name, field_patterns, payload)? {
        return check_variant_fields(
            env,
            field_patterns,
            payload,
            owner,
            None,
            variant_def,
            bindings,
        );
    }

    Err(TypeError::UnknownVariant(
        variant_name.to_string(),
        Span::default(),
    ))
}

fn is_canonical_result_deferred_to_registered_variant(error: &TypeError) -> bool {
    matches!(error, TypeError::PatternMismatch { .. })
}

fn try_registered_variant_pattern(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    expected: &Type,
    bindings: &Bindings,
) -> Result<Option<Bindings>, TypeError> {
    let Some((owner, variant_def)) = env.lookup_variant(variant_name, field_patterns, payload)?
    else {
        return Ok(None);
    };

    if let Type::Constructor { name, .. } = expected
        && owner.name != name.name
    {
        return Ok(None);
    }

    let mut registered_bindings = bindings.clone();
    check_variant_fields(
        env,
        field_patterns,
        payload,
        owner,
        Some(expected),
        variant_def,
        &mut registered_bindings,
    )?;
    Ok(Some(registered_bindings))
}

fn check_variant_fields(
    env: &TypeEnv,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    owner: &TypeDef,
    owner_type: Option<&Type>,
    variant_def: &VariantDef,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    let variant_name = variant_def.name.as_str();
    match (payload, &variant_def.payload) {
        (VariantPatternPayload::Unit, VariantPayload::Unit) => Ok(()),
        (VariantPatternPayload::Record(record_fields), VariantPayload::Record(_)) => {
            check_record_variant_fields(
                env,
                variant_name,
                field_patterns.unwrap_or(record_fields.as_slice()),
                &variant_def.fields,
                owner,
                owner_type,
                bindings,
            )
        }
        (VariantPatternPayload::Tuple(items), VariantPayload::Tuple(_)) => {
            check_tuple_variant_fields(
                env,
                variant_name,
                items,
                &variant_def.fields,
                owner,
                owner_type,
                bindings,
            )
        }
        _ => Err(TypeError::InvalidPattern {
            message: format!("variant {variant_name} payload shape mismatch"),
            span: Span::default(),
        }),
    }
}

fn check_canonical_variant_fields(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    payload: &VariantPatternPayload,
    variant: &PatternCanonicalConstructor,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match (payload, variant.payload_shape.clone()) {
        (VariantPatternPayload::Unit, VariantPayloadShape::Unit) => Ok(()),
        (VariantPatternPayload::Record(record_fields), VariantPayloadShape::Record) => {
            check_record_variant_fields_from_types(
                env,
                variant_name,
                field_patterns.unwrap_or(record_fields.as_slice()),
                &variant.fields,
                bindings,
            )
        }
        (VariantPatternPayload::Tuple(items), VariantPayloadShape::Tuple) => {
            check_tuple_variant_fields_from_types(
                env,
                variant_name,
                items,
                &variant.fields,
                bindings,
            )
        }
        _ => Err(TypeError::InvalidPattern {
            message: format!("variant {variant_name} payload shape mismatch"),
            span: Span::default(),
        }),
    }
}

fn check_record_variant_fields(
    env: &TypeEnv,
    _variant_name: &str,
    field_patterns: &[(Box<str>, Pattern)],
    variant_fields: &[(String, ash_core::ast::TypeExpr)],
    owner: &TypeDef,
    owner_type: Option<&Type>,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    for (field_name, field_pattern) in field_patterns {
        let field_type = variant_fields
            .iter()
            .find(|(name, _)| name == field_name.as_ref())
            .map(|(_, ty)| env.lower_type_expr_for_owner_type(owner, owner_type, ty))
            .transpose()?
            .ok_or_else(|| TypeError::InvalidPattern {
                message: format!("unknown field: {field_name}"),
                span: Span::default(),
            })?;
        check_pattern_inner(env, field_pattern, &field_type, bindings)?;
    }

    Ok(())
}

fn check_record_variant_fields_from_types(
    env: &TypeEnv,
    _variant_name: &str,
    field_patterns: &[(Box<str>, Pattern)],
    variant_fields: &[(String, Type)],
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    for (field_name, field_pattern) in field_patterns {
        let field_type = variant_fields
            .iter()
            .find(|(name, _)| name == field_name.as_ref())
            .map(|(_, ty)| ty)
            .ok_or_else(|| TypeError::InvalidPattern {
                message: format!("unknown field: {field_name}"),
                span: Span::default(),
            })?;
        check_pattern_inner(env, field_pattern, field_type, bindings)?;
    }

    Ok(())
}

fn check_tuple_variant_fields(
    env: &TypeEnv,
    variant_name: &str,
    items: &[Pattern],
    variant_fields: &[(String, ash_core::ast::TypeExpr)],
    owner: &TypeDef,
    owner_type: Option<&Type>,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    if items.len() != variant_fields.len() {
        return Err(TypeError::InvalidPattern {
            message: format!(
                "tuple variant {variant_name} expects {} positional items, got {}",
                variant_fields.len(),
                items.len()
            ),
            span: Span::default(),
        });
    }

    for (index, pattern) in items.iter().enumerate() {
        let expected_name = tuple_field_name(index);
        let field_expr = variant_fields
            .iter()
            .find(|(name, _)| name == &expected_name)
            .map(|(_, ty)| ty)
            .or_else(|| variant_fields.get(index).map(|(_, ty)| ty))
            .ok_or_else(|| TypeError::InvalidPattern {
                message: format!("tuple variant {variant_name} is missing positional slot {index}"),
                span: Span::default(),
            })?;
        let field_type = env.lower_type_expr_for_owner_type(owner, owner_type, field_expr)?;
        check_pattern_inner(env, pattern, &field_type, bindings)?;
    }

    Ok(())
}

fn check_tuple_variant_fields_from_types(
    env: &TypeEnv,
    variant_name: &str,
    items: &[Pattern],
    variant_fields: &[(String, Type)],
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    if items.len() != variant_fields.len() {
        return Err(TypeError::InvalidPattern {
            message: format!(
                "tuple variant {variant_name} expects {} positional items, got {}",
                variant_fields.len(),
                items.len()
            ),
            span: Span::default(),
        });
    }

    for (index, pattern) in items.iter().enumerate() {
        let expected_name = tuple_field_name(index);
        let field_type = variant_fields
            .iter()
            .find(|(name, _)| name == &expected_name)
            .map(|(_, ty)| ty)
            .or_else(|| variant_fields.get(index).map(|(_, ty)| ty))
            .ok_or_else(|| TypeError::InvalidPattern {
                message: format!("tuple variant {variant_name} is missing positional slot {index}"),
                span: Span::default(),
            })?;
        check_pattern_inner(env, pattern, field_type, bindings)?;
    }

    Ok(())
}

/// Check a tuple pattern against a type
fn check_tuple_pattern(
    env: &TypeEnv,
    patterns: &[Pattern],
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::Record(fields) => {
            // Tuples are represented as records with numeric field names
            if patterns.len() != fields.len() {
                return Err(TypeError::PatternArityMismatch {
                    expected: fields.len(),
                    actual: patterns.len(),
                    span: Span::default(),
                });
            }

            for (i, pattern) in patterns.iter().enumerate() {
                let field_idx = format!("{i}");
                let field_type = fields
                    .iter()
                    .find(|(n, _)| n.as_ref() == field_idx)
                    .map(|(_, t)| t)
                    .ok_or(TypeError::PatternArityMismatch {
                        expected: fields.len(),
                        actual: patterns.len(),
                        span: Span::default(),
                    })?;
                check_pattern_inner(env, pattern, field_type, bindings)?;
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh types for each element
            for pattern in patterns {
                let fresh_type = Type::Var(TypeVar::fresh());
                check_pattern_inner(env, pattern, &fresh_type, bindings)?;
            }
            Ok(())
        }
        _ => Err(TypeError::PatternMismatch {
            expected: Box::new(expected.clone()),
            actual: Box::new(Type::Record(
                patterns
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (Box::from(format!("{i}")), Type::Var(TypeVar::fresh())))
                    .collect(),
            )),
            span: Span::default(),
        }),
    }
}

/// Check a record pattern against a type
fn check_record_pattern(
    env: &TypeEnv,
    field_patterns: &[(Box<str>, Pattern)],
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::Record(fields) => {
            for (field_name, field_pattern) in field_patterns {
                let field_type = fields
                    .iter()
                    .find(|(n, _)| n.as_ref() == field_name.as_ref())
                    .map(|(_, t)| t)
                    .ok_or_else(|| TypeError::InvalidPattern {
                        message: format!("unknown field: {field_name}"),
                        span: Span::default(),
                    })?;
                check_pattern_inner(env, field_pattern, field_type, bindings)?;
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh types for each field
            for (field_name, field_pattern) in field_patterns {
                let _ = field_name;
                let fresh_type = Type::Var(TypeVar::fresh());
                check_pattern_inner(env, field_pattern, &fresh_type, bindings)?;
            }
            Ok(())
        }
        _ => Err(TypeError::PatternMismatch {
            expected: Box::new(expected.clone()),
            actual: Box::new(Type::Record(
                field_patterns
                    .iter()
                    .map(|(n, _)| (n.clone(), Type::Var(TypeVar::fresh())))
                    .collect(),
            )),
            span: Span::default(),
        }),
    }
}

/// Check a list pattern against a type
fn check_list_pattern(
    env: &TypeEnv,
    elements: &[Pattern],
    rest: Option<&str>,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::List(elem_type) => {
            for element in elements {
                check_pattern_inner(env, element, elem_type, bindings)?;
            }
            if let Some(rest_name) = rest {
                // Rest binding gets the list type
                bindings.insert(rest_name.to_string(), expected.clone());
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh type for elements
            let elem_type = Type::Var(TypeVar::fresh());
            for element in elements {
                check_pattern_inner(env, element, &elem_type, bindings)?;
            }
            if let Some(rest_name) = rest {
                let list_type = Type::List(Box::new(elem_type));
                bindings.insert(rest_name.to_string(), list_type);
            }
            Ok(())
        }
        _ => Err(TypeError::PatternMismatch {
            expected: Box::new(expected.clone()),
            actual: Box::new(Type::List(Box::new(Type::Var(TypeVar::fresh())))),
            span: Span::default(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{TypeBody, TypeExpr, VariantPayload, Visibility};

    fn option_env() -> TypeEnv {
        let mut env = TypeEnv::new();
        env.add_type_def(
            "Option".to_string(),
            TypeDef {
                name: "Option".to_string(),
                params: vec![],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![("value".to_string(), TypeExpr::Named("Int".to_string()))],
                        payload: VariantPayload::Record(vec![(
                            "value".to_string(),
                            TypeExpr::Named("Int".to_string()),
                        )]),
                    },
                    VariantDef {
                        name: "None".to_string(),
                        fields: vec![],
                        payload: VariantPayload::Unit,
                    },
                ]),
                visibility: Visibility::Public,
                builtin: false,
            },
        );
        env
    }

    fn generic_option_env() -> TypeEnv {
        let mut env = TypeEnv::new();
        env.add_type_def(
            "Option".to_string(),
            TypeDef {
                name: "Option".to_string(),
                params: vec!["T".to_string()],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![("value".to_string(), TypeExpr::Named("T".to_string()))],
                        payload: VariantPayload::Record(vec![(
                            "value".to_string(),
                            TypeExpr::Named("T".to_string()),
                        )]),
                    },
                    VariantDef {
                        name: "None".to_string(),
                        fields: vec![],
                        payload: VariantPayload::Unit,
                    },
                ]),
                visibility: Visibility::Public,
                builtin: false,
            },
        );
        env
    }

    // ============================================================
    // Wildcard Pattern Tests
    // ============================================================

    #[test]
    fn test_wildcard_matches_any_type() {
        let env = TypeEnv::new();
        let pattern = Pattern::Wildcard;

        // Wildcard should match any type with no bindings
        let bindings = check_pattern(&env, &pattern, &Type::Int).unwrap();
        assert!(bindings.is_empty());

        let bindings = check_pattern(&env, &pattern, &Type::String).unwrap();
        assert!(bindings.is_empty());

        let bindings = check_pattern(&env, &pattern, &Type::Bool).unwrap();
        assert!(bindings.is_empty());
    }

    // ============================================================
    // Variable Pattern Tests
    // ============================================================

    #[test]
    fn test_variable_binds_to_expected_type() {
        let env = TypeEnv::new();
        let pattern = Pattern::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Int).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variable_binds_different_types() {
        let env = TypeEnv::new();

        let pattern = Pattern::Variable {
            name: "s".into(),
            span: ash_parser::token::Span::default(),
        };
        let bindings = check_pattern(&env, &pattern, &Type::String).unwrap();
        assert_eq!(bindings.get("s"), Some(&Type::String));

        let pattern = Pattern::Variable {
            name: "b".into(),
            span: ash_parser::token::Span::default(),
        };
        let bindings = check_pattern(&env, &pattern, &Type::Bool).unwrap();
        assert_eq!(bindings.get("b"), Some(&Type::Bool));
    }

    // ============================================================
    // Literal Pattern Tests
    // ============================================================

    #[test]
    fn test_literal_int_matches_int() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::Int(42));

        let bindings = check_pattern(&env, &pattern, &Type::Int).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_literal_int_mismatch_error() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::Int(42));

        let result = check_pattern(&env, &pattern, &Type::String);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::PatternMismatch { .. }
        ));
    }

    #[test]
    fn test_literal_string_matches_string() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::String("hello".into()));

        let bindings = check_pattern(&env, &pattern, &Type::String).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_literal_bool_matches_bool() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::Bool(true));

        let bindings = check_pattern(&env, &pattern, &Type::Bool).unwrap();
        assert!(bindings.is_empty());
    }

    // ============================================================
    // Variant Pattern Tests
    // ============================================================

    #[test]
    fn test_variant_pattern_with_fields() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variant_pattern_rejects_non_adt_expected_type() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };

        let result = check_pattern(&env, &pattern, &Type::Int);
        assert!(matches!(result, Err(TypeError::PatternMismatch { .. })));
    }

    #[test]
    fn test_variant_pattern_none() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "None".into(),
            fields: None,
            payload: VariantPatternPayload::Unit,
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_variant_pattern_type_var() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };

        let type_var = Type::Var(TypeVar::fresh());
        let bindings = check_pattern(&env, &pattern, &type_var).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    // ============================================================
    // Tuple Pattern Tests
    // ============================================================

    #[test]
    fn test_tuple_pattern_matches() {
        let env = TypeEnv::new();
        // (a, b) pattern
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable {
                name: "a".into(),
                span: ash_parser::token::Span::default(),
            },
            Pattern::Variable {
                name: "b".into(),
                span: ash_parser::token::Span::default(),
            },
        ]);

        // Tuple represented as record with numeric fields
        let tuple_type = Type::Record(vec![
            (Box::from("0"), Type::Int),
            (Box::from("1"), Type::String),
        ]);

        let bindings = check_pattern(&env, &pattern, &tuple_type).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::String));
    }

    #[test]
    fn test_tuple_pattern_arity_mismatch() {
        let env = TypeEnv::new();
        // (a, b, c) pattern against 2-element tuple
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable {
                name: "a".into(),
                span: ash_parser::token::Span::default(),
            },
            Pattern::Variable {
                name: "b".into(),
                span: ash_parser::token::Span::default(),
            },
            Pattern::Variable {
                name: "c".into(),
                span: ash_parser::token::Span::default(),
            },
        ]);

        let tuple_type = Type::Record(vec![
            (Box::from("0"), Type::Int),
            (Box::from("1"), Type::String),
        ]);

        let result = check_pattern(&env, &pattern, &tuple_type);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::PatternArityMismatch {
                expected: 2,
                actual: 3,
                ..
            }
        ));
    }

    #[test]
    fn test_tuple_pattern_type_var() {
        let env = TypeEnv::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable {
                name: "a".into(),
                span: ash_parser::token::Span::default(),
            },
            Pattern::Variable {
                name: "b".into(),
                span: ash_parser::token::Span::default(),
            },
        ]);

        // Against a type variable
        let type_var = Type::Var(TypeVar::fresh());
        let bindings = check_pattern(&env, &pattern, &type_var).unwrap();
        assert!(bindings.contains_key("a"));
        assert!(bindings.contains_key("b"));
    }

    // ============================================================
    // Record Pattern Tests
    // ============================================================

    #[test]
    fn test_record_pattern_matches() {
        let env = TypeEnv::new();
        // { name: n, age: a } pattern
        let pattern = Pattern::Record(vec![
            (
                Box::from("name"),
                Pattern::Variable {
                    name: "n".into(),
                    span: ash_parser::token::Span::default(),
                },
            ),
            (
                Box::from("age"),
                Pattern::Variable {
                    name: "a".into(),
                    span: ash_parser::token::Span::default(),
                },
            ),
        ]);

        let record_type = Type::Record(vec![
            (Box::from("name"), Type::String),
            (Box::from("age"), Type::Int),
        ]);

        let bindings = check_pattern(&env, &pattern, &record_type).unwrap();
        assert_eq!(bindings.get("n"), Some(&Type::String));
        assert_eq!(bindings.get("a"), Some(&Type::Int));
    }

    #[test]
    fn test_record_pattern_unknown_field() {
        let env = TypeEnv::new();
        // { unknown: x } pattern
        let pattern = Pattern::Record(vec![(
            Box::from("unknown"),
            Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
        )]);

        let record_type = Type::Record(vec![(Box::from("name"), Type::String)]);

        let result = check_pattern(&env, &pattern, &record_type);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::InvalidPattern { .. }
        ));
    }

    // ============================================================
    // List Pattern Tests
    // ============================================================

    #[test]
    fn test_list_pattern_matches() {
        let env = TypeEnv::new();
        // [a, b] pattern
        let pattern = Pattern::List {
            elements: vec![
                Pattern::Variable {
                    name: "a".into(),
                    span: ash_parser::token::Span::default(),
                },
                Pattern::Variable {
                    name: "b".into(),
                    span: ash_parser::token::Span::default(),
                },
            ],
            rest: None,
        };

        let list_type = Type::List(Box::new(Type::Int));

        let bindings = check_pattern(&env, &pattern, &list_type).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::Int));
    }

    #[test]
    fn test_list_pattern_with_rest() {
        let env = TypeEnv::new();
        // [first, ..rest] pattern
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable {
                name: "first".into(),
                span: ash_parser::token::Span::default(),
            }],
            rest: Some(Box::from("rest")),
        };

        let list_type = Type::List(Box::new(Type::Int));

        let bindings = check_pattern(&env, &pattern, &list_type).unwrap();
        assert_eq!(bindings.get("first"), Some(&Type::Int));
        assert_eq!(bindings.get("rest"), Some(&list_type));
    }

    #[test]
    fn test_list_pattern_mismatch() {
        let env = TypeEnv::new();
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable {
                name: "a".into(),
                span: ash_parser::token::Span::default(),
            }],
            rest: None,
        };

        let result = check_pattern(&env, &pattern, &Type::Int);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::PatternMismatch { .. }
        ));
    }

    // ============================================================
    // Integration Tests
    // ============================================================

    #[test]
    fn test_some_value_against_option_int() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_none_against_option_no_bindings() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "None".into(),
            fields: None,
            payload: VariantPatternPayload::Unit,
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_nested_pattern() {
        let mut env = TypeEnv::new();
        env.add_type_def(
            "OptionTuple".to_string(),
            TypeDef {
                name: "OptionTuple".to_string(),
                params: vec![],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![(
                            "value".to_string(),
                            TypeExpr::Record(vec![
                                ("0".to_string(), TypeExpr::Named("Int".to_string())),
                                ("1".to_string(), TypeExpr::Named("String".to_string())),
                            ]),
                        )],
                        payload: VariantPayload::Record(vec![(
                            "value".to_string(),
                            TypeExpr::Record(vec![
                                ("0".to_string(), TypeExpr::Named("Int".to_string())),
                                ("1".to_string(), TypeExpr::Named("String".to_string())),
                            ]),
                        )]),
                    },
                    VariantDef {
                        name: "None".to_string(),
                        fields: vec![],
                        payload: VariantPayload::Unit,
                    },
                ]),
                visibility: Visibility::Public,
                builtin: false,
            },
        );
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Tuple(vec![
                    Pattern::Variable {
                        name: "a".into(),
                        span: ash_parser::token::Span::default(),
                    },
                    Pattern::Variable {
                        name: "b".into(),
                        span: ash_parser::token::Span::default(),
                    },
                ]),
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Tuple(vec![
                    Pattern::Variable {
                        name: "a".into(),
                        span: ash_parser::token::Span::default(),
                    },
                    Pattern::Variable {
                        name: "b".into(),
                        span: ash_parser::token::Span::default(),
                    },
                ]),
            )]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::String));
    }

    #[test]
    fn test_variant_pattern_uses_field_shape_to_disambiguate_constructors() {
        let mut env = option_env();
        env.add_type_def(
            "Maybe".to_string(),
            TypeDef {
                name: "Maybe".to_string(),
                params: vec![],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![("other".to_string(), TypeExpr::Named("Bool".to_string()))],
                        payload: VariantPayload::Record(vec![(
                            "other".to_string(),
                            TypeExpr::Named("Bool".to_string()),
                        )]),
                    },
                    VariantDef {
                        name: "Never".to_string(),
                        fields: vec![],
                        payload: VariantPayload::Unit,
                    },
                ]),
                visibility: Visibility::Public,
                builtin: false,
            },
        );

        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variant_pattern_accepts_expected_constructor_type() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };
        let expected = Type::Constructor {
            name: crate::QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: crate::Kind::Type,
        };

        let bindings = check_pattern(&env, &pattern, &expected).unwrap();

        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variant_pattern_substitutes_expected_constructor_type_params() {
        let env = generic_option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
            payload: VariantPatternPayload::Record(vec![(
                "value".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            )]),
        };
        let expected = Type::Constructor {
            name: crate::QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: crate::Kind::Type,
        };

        let bindings = check_pattern(&env, &pattern, &expected).unwrap();

        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }
}
