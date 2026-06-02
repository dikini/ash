//! Exhaustiveness checking for pattern matches
//!
//! Provides exhaustiveness analysis to ensure all pattern match cases are covered.
//! Uses a pattern matrix approach for analyzing coverage.

use crate::type_env::{
    PatternCanonicalConstructor, PatternCanonicalType, PatternCanonicalization,
    PatternCanonicalizationBlockedReason, TypeEnv,
};
use crate::types::Type;
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{Pattern, TypeBody, TypeDef, VariantPayload};

/// Coverage result for exhaustiveness checking
#[derive(Debug, Clone, PartialEq)]
pub enum Coverage {
    /// All cases are covered
    Covered,
    /// Some cases are missing
    Missing(Vec<Pattern>),
}

/// Type-aware coverage result for ordinary match expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchCoverage {
    /// All cases are covered.
    Covered,
    /// Some cases are missing.
    Missing(Vec<Pattern>),
    /// Constructor-specific coverage needs a constructor universe that is unavailable.
    Blocked {
        /// Source type passed to pattern canonicalization.
        source_type: Type,
        /// Typed blocked reason.
        reason: PatternCanonicalizationBlockedReason,
    },
    /// The checker intentionally does not claim exhaustive coverage for this shape.
    Unsupported {
        /// Scrutinee type whose coverage was not proven.
        scrutinee_type: Type,
        /// Human-facing reason and guidance.
        reason: String,
    },
}

/// Pattern matrix for exhaustiveness analysis
#[derive(Debug, Clone)]
pub struct PatternMatrix {
    /// Rows of pattern cells
    rows: Vec<Vec<PatternCell>>,
}

/// A single cell in the pattern matrix
#[derive(Debug, Clone)]
pub enum PatternCell {
    /// Wildcard pattern that matches anything
    Wildcard,
    /// Constructor pattern with name and field patterns
    ///
    /// `fields == None` represents a unit-variant pattern written without
    /// destructuring braces (e.g. `None` for `Option::None`).
    /// `fields == Some(_)` represents a constructor pattern with braces
    /// (e.g. `Some { value: x }`).
    Constructor(String, Option<Vec<PatternCell>>),
}

impl PatternMatrix {
    /// Create a new pattern matrix from a list of patterns
    #[must_use]
    pub fn new(patterns: &[Pattern]) -> Self {
        let rows = patterns.iter().map(|p| vec![pattern_to_cell(p)]).collect();
        Self { rows }
    }

    /// Check if the matrix has any rows
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the number of rows in the matrix
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Convert an AST pattern to a pattern cell
pub fn pattern_to_cell(pattern: &Pattern) -> PatternCell {
    match pattern {
        Pattern::Wildcard | Pattern::Variable { .. } => PatternCell::Wildcard,
        Pattern::Variant { name, fields } => {
            let field_cells = fields
                .as_ref()
                .map(|f| f.iter().map(|(_, p)| pattern_to_cell(p)).collect());
            PatternCell::Constructor(name.clone(), field_cells)
        }
        Pattern::Tuple(patterns) => PatternCell::Constructor(
            "tuple".to_string(),
            Some(patterns.iter().map(pattern_to_cell).collect()),
        ),
        Pattern::Literal(_) => PatternCell::Constructor("literal".to_string(), None),
        Pattern::Record(fields) => PatternCell::Constructor(
            "record".to_string(),
            Some(fields.iter().map(|(_, p)| pattern_to_cell(p)).collect()),
        ),
        Pattern::List(patterns, rest) => {
            let mut cells: Vec<PatternCell> = patterns.iter().map(pattern_to_cell).collect();
            if rest.is_some() {
                cells.push(PatternCell::Wildcard);
            }
            PatternCell::Constructor("list".to_string(), Some(cells))
        }
    }
}

/// Check if patterns cover all cases for a type
pub fn check_exhaustive(patterns: &[Pattern], type_def: &TypeDef) -> Coverage {
    let matrix = PatternMatrix::new(patterns);

    match find_uncovered(&matrix, type_def) {
        None => Coverage::Covered,
        Some(witnesses) => Coverage::Missing(witnesses),
    }
}

/// Check if patterns cover all cases for a canonical pattern constructor universe.
pub fn check_exhaustive_canonical(
    patterns: &[Pattern],
    canonical: &PatternCanonicalType,
) -> Coverage {
    let matrix = PatternMatrix::new(patterns);

    match find_uncovered_canonical(&matrix, &canonical.constructors) {
        None => Coverage::Covered,
        Some(witnesses) => Coverage::Missing(witnesses),
    }
}

/// Check ordinary match exhaustiveness against the scrutinee type.
///
/// This API is deliberately conservative outside ordinary ADTs. Wildcard and
/// variable arms are universal for every well-typed scrutinee, but
/// refutable non-wildcard arms require a SPEC-068 canonical constructor universe.
#[must_use]
pub fn check_match_exhaustive(
    env: &TypeEnv,
    patterns: &[Pattern],
    scrutinee_type: &Type,
) -> MatchCoverage {
    check_match_exhaustive_inner(env, patterns, scrutinee_type)
}

/// Find uncovered patterns for a type
fn find_uncovered(matrix: &PatternMatrix, type_def: &TypeDef) -> Option<Vec<Pattern>> {
    let variants = match &type_def.body {
        TypeBody::Enum(variants) => variants,
        _ => return None,
    };

    // Check if there's a wildcard pattern (covers everything)
    let has_wildcard = matrix
        .rows
        .iter()
        .any(|row| matches!(row.first(), Some(PatternCell::Wildcard)));

    if has_wildcard {
        return None;
    }

    // Find missing variants
    let mut missing = Vec::new();
    for variant in variants {
        let is_covered = matrix.rows.iter().any(|row| match row.first() {
            Some(PatternCell::Constructor(name, pattern_fields)) if name == &variant.name => {
                match pattern_fields {
                    // Unit-variant patterns only cover variants that have zero fields.
                    None => variant.fields.is_empty(),
                    // Braced constructor patterns cover (conservatively) the variant.
                    Some(_) => true,
                }
            }
            _ => false,
        });

        if !is_covered {
            let witness_fields = match &variant.payload {
                VariantPayload::Unit => None,
                VariantPayload::Record(fields) => Some(
                    fields
                        .iter()
                        .map(|(field_name, _)| (field_name.clone(), Pattern::Wildcard))
                        .collect(),
                ),
                VariantPayload::Tuple(items) => Some(
                    items
                        .iter()
                        .enumerate()
                        .map(|(index, _)| (tuple_field_name(index), Pattern::Wildcard))
                        .collect(),
                ),
            };

            missing.push(Pattern::Variant {
                name: variant.name.clone(),
                fields: witness_fields,
            });
        }
    }

    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

/// Find uncovered patterns for a canonical constructor universe.
fn find_uncovered_canonical(
    matrix: &PatternMatrix,
    constructors: &[PatternCanonicalConstructor],
) -> Option<Vec<Pattern>> {
    let has_wildcard = matrix
        .rows
        .iter()
        .any(|row| matches!(row.first(), Some(PatternCell::Wildcard)));

    if has_wildcard {
        return None;
    }

    let mut missing = Vec::new();
    for constructor in constructors {
        let is_covered = matrix.rows.iter().any(|row| match row.first() {
            Some(PatternCell::Constructor(name, pattern_fields)) if name == &constructor.name => {
                match pattern_fields {
                    None => matches!(constructor.payload_shape, VariantPayloadShape::Unit),
                    Some(_) => true,
                }
            }
            _ => false,
        });

        if !is_covered {
            let witness_fields = match constructor.payload_shape {
                VariantPayloadShape::Unit => None,
                VariantPayloadShape::Record => Some(
                    constructor
                        .fields
                        .iter()
                        .map(|(field_name, _)| (field_name.clone(), Pattern::Wildcard))
                        .collect(),
                ),
                VariantPayloadShape::Tuple => Some(
                    constructor
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, _)| (tuple_field_name(index), Pattern::Wildcard))
                        .collect(),
                ),
            };

            missing.push(Pattern::Variant {
                name: constructor.name.clone(),
                fields: witness_fields,
            });
        }
    }

    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

fn check_match_exhaustive_inner(
    env: &TypeEnv,
    patterns: &[Pattern],
    scrutinee_type: &Type,
) -> MatchCoverage {
    if patterns
        .iter()
        .any(|pattern| is_universal_for_type(pattern, scrutinee_type))
    {
        return MatchCoverage::Covered;
    }

    match env.canonicalize_type_for_pattern(scrutinee_type) {
        PatternCanonicalization::Matchable(canonical) => {
            check_canonical_type_coverage(env, patterns, &canonical)
        }
        PatternCanonicalization::Blocked {
            source_type,
            reason,
        } => match (&reason, scrutinee_type) {
            (PatternCanonicalizationBlockedReason::NonAdt, Type::List(_))
                if patterns.iter().any(contains_list_pattern) =>
            {
                unsupported_list_coverage(scrutinee_type)
            }
            _ if patterns.iter().any(contains_variant_pattern) => MatchCoverage::Blocked {
                source_type,
                reason,
            },
            _ => unsupported_open_coverage(scrutinee_type),
        },
    }
}

fn check_canonical_type_coverage(
    env: &TypeEnv,
    patterns: &[Pattern],
    canonical: &PatternCanonicalType,
) -> MatchCoverage {
    let mut missing = Vec::new();

    for constructor in &canonical.constructors {
        let rows = patterns
            .iter()
            .filter_map(|pattern| constructor_field_patterns(pattern, constructor))
            .collect::<Vec<_>>();

        if constructor.fields.is_empty() {
            if rows.is_empty() {
                missing.push(constructor_witness_pattern(constructor, Vec::new()));
            }
            continue;
        }

        if rows.is_empty() {
            missing.push(constructor_witness_pattern(
                constructor,
                wildcard_fields(constructor.fields.len()),
            ));
            continue;
        }

        if let Some(field_witnesses) = product_missing_witness(env, &rows, &constructor.fields) {
            missing.push(constructor_witness_pattern(constructor, field_witnesses));
        }
    }

    if missing.is_empty() {
        MatchCoverage::Covered
    } else {
        MatchCoverage::Missing(missing)
    }
}

fn product_missing_witness(
    env: &TypeEnv,
    rows: &[Vec<Pattern>],
    fields: &[(String, Type)],
) -> Option<Vec<Pattern>> {
    if fields.is_empty() {
        return rows.is_empty().then(Vec::new);
    }

    if rows.iter().any(|row| {
        row.len() == fields.len()
            && row
                .iter()
                .zip(fields)
                .all(|(pattern, (_, ty))| single_pattern_covers_type(env, pattern, ty))
    }) {
        return None;
    }

    let (_, first_type) = &fields[0];
    if let PatternCanonicalization::Matchable(canonical) =
        env.canonicalize_type_for_pattern(first_type)
    {
        for constructor in &canonical.constructors {
            let mut expanded_fields = constructor.fields.clone();
            expanded_fields.extend(fields[1..].iter().cloned());

            let specialized_rows = rows
                .iter()
                .filter_map(|row| {
                    let first = row.first()?;
                    let rest = row.iter().skip(1).cloned();
                    if is_universal_for_type(first, first_type) {
                        let mut specialized = wildcard_fields(constructor.fields.len());
                        specialized.extend(rest);
                        Some(specialized)
                    } else {
                        constructor_field_patterns(first, constructor).map(|mut fields| {
                            fields.extend(rest);
                            fields
                        })
                    }
                })
                .collect::<Vec<_>>();

            if let Some(mut missing) =
                product_missing_witness(env, &specialized_rows, &expanded_fields)
            {
                let rest_missing = missing.split_off(constructor.fields.len());
                let first_missing = constructor_witness_pattern(constructor, missing);
                let mut witness = vec![first_missing];
                witness.extend(rest_missing);
                return Some(witness);
            }
        }

        return None;
    }

    let rest_fields = &fields[1..];
    let universal_first_rows = rows
        .iter()
        .filter(|row| {
            row.first()
                .is_some_and(|first| is_universal_for_type(first, first_type))
        })
        .map(|row| row.iter().skip(1).cloned().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    if !universal_first_rows.is_empty()
        && product_missing_witness(env, &universal_first_rows, rest_fields).is_none()
    {
        return None;
    }

    let mut witness = vec![default_witness_for_type(first_type)];
    witness.extend(wildcard_fields(rest_fields.len()));
    Some(witness)
}

fn single_pattern_covers_type(env: &TypeEnv, pattern: &Pattern, ty: &Type) -> bool {
    matches!(
        check_match_exhaustive_inner(env, std::slice::from_ref(pattern), ty),
        MatchCoverage::Covered
    )
}

fn unsupported_list_coverage(scrutinee_type: &Type) -> MatchCoverage {
    MatchCoverage::Unsupported {
        scrutinee_type: scrutinee_type.clone(),
        reason: "list pattern coverage is conservative for variable-length lists; add a wildcard/default arm to make the match exhaustive".to_string(),
    }
}

fn unsupported_open_coverage(scrutinee_type: &Type) -> MatchCoverage {
    MatchCoverage::Unsupported {
        scrutinee_type: scrutinee_type.clone(),
        reason: "refutable non-wildcard patterns over a non-ADT or open scrutinee need an ADT constructor universe; add a wildcard/default arm to make the match exhaustive".to_string(),
    }
}

fn constructor_field_patterns(
    pattern: &Pattern,
    constructor: &PatternCanonicalConstructor,
) -> Option<Vec<Pattern>> {
    let Pattern::Variant { name, fields } = pattern else {
        return None;
    };
    if name != &constructor.name {
        return None;
    }

    match constructor.payload_shape {
        VariantPayloadShape::Unit => fields.is_none().then(Vec::new),
        VariantPayloadShape::Record => fields.as_ref().and_then(|fields| {
            constructor
                .fields
                .iter()
                .map(|(expected_name, _)| {
                    fields
                        .iter()
                        .find(|(name, _)| name == expected_name)
                        .map(|(_, pattern)| pattern.clone())
                })
                .collect()
        }),
        VariantPayloadShape::Tuple => fields.as_ref().and_then(|fields| {
            (0..constructor.fields.len())
                .map(|index| {
                    let expected_name = tuple_field_name(index);
                    fields
                        .iter()
                        .find(|(name, _)| name == &expected_name)
                        .map(|(_, pattern)| pattern.clone())
                        .or_else(|| fields.get(index).map(|(_, pattern)| pattern.clone()))
                })
                .collect()
        }),
    }
}

fn constructor_witness_pattern(
    constructor: &PatternCanonicalConstructor,
    field_patterns: Vec<Pattern>,
) -> Pattern {
    let fields = match constructor.payload_shape {
        VariantPayloadShape::Unit => None,
        VariantPayloadShape::Record => Some(
            constructor
                .fields
                .iter()
                .enumerate()
                .map(|(index, (field_name, _))| {
                    (
                        field_name.clone(),
                        field_patterns
                            .get(index)
                            .cloned()
                            .unwrap_or(Pattern::Wildcard),
                    )
                })
                .collect(),
        ),
        VariantPayloadShape::Tuple => Some(
            constructor
                .fields
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    (
                        tuple_field_name(index),
                        field_patterns
                            .get(index)
                            .cloned()
                            .unwrap_or(Pattern::Wildcard),
                    )
                })
                .collect(),
        ),
    };

    Pattern::Variant {
        name: constructor.name.clone(),
        fields,
    }
}

fn default_witness_for_type(ty: &Type) -> Pattern {
    match ty {
        Type::List(_) => Pattern::List(Vec::new(), None),
        _ => Pattern::Wildcard,
    }
}

fn wildcard_fields(len: usize) -> Vec<Pattern> {
    vec![Pattern::Wildcard; len]
}

fn is_universal_for_type(pattern: &Pattern, _ty: &Type) -> bool {
    matches!(pattern, Pattern::Wildcard | Pattern::Variable { .. })
}

fn contains_variant_pattern(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Variant { .. } => true,
        Pattern::Tuple(items) | Pattern::List(items, _) => {
            items.iter().any(contains_variant_pattern)
        }
        Pattern::Record(fields) => fields
            .iter()
            .any(|(_, pattern)| contains_variant_pattern(pattern)),
        Pattern::Wildcard | Pattern::Variable { .. } | Pattern::Literal(_) => false,
    }
}

fn contains_list_pattern(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::List(_, _) => true,
        Pattern::Tuple(items) => items.iter().any(contains_list_pattern),
        Pattern::Record(fields) => fields
            .iter()
            .any(|(_, pattern)| contains_list_pattern(pattern)),
        Pattern::Variant { fields, .. } => fields.as_ref().is_some_and(|fields| {
            fields
                .iter()
                .any(|(_, pattern)| contains_list_pattern(pattern))
        }),
        Pattern::Wildcard | Pattern::Variable { .. } | Pattern::Literal(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{TypeExpr, VariantDef, VariantPayload, Visibility};

    /// Create a test Option type with Some and None variants
    fn make_option_type() -> TypeDef {
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
        }
    }

    #[test]
    fn test_exhaustive_full_coverage() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![("value".to_string(), Pattern::Wildcard)]),
            },
            Pattern::Variant {
                name: "None".to_string(),
                fields: None,
            },
        ];

        assert_eq!(check_exhaustive(&patterns, &option_type), Coverage::Covered);
    }

    #[test]
    fn test_non_exhaustive_some_requires_field_patterns() {
        let option_type = make_option_type();
        let patterns = vec![
            // `Option::Some` has a payload field (`value`), so a pattern that
            // omits the `{ ... }` field destructuring should not count as
            // covering `Some`.
            Pattern::Variant {
                name: "Some".to_string(),
                fields: None,
            },
            Pattern::Variant {
                name: "None".to_string(),
                fields: None,
            },
        ];

        match check_exhaustive(&patterns, &option_type) {
            Coverage::Missing(missing) => {
                assert!(
                    missing.iter().any(|p| {
                        matches!(
                            p,
                            Pattern::Variant {
                                name,
                                fields: Some(_)
                            } if name == "Some"
                        )
                    }),
                    "Expected missing coverage for `Some` with fields"
                );
            }
            other => panic!("Expected Missing coverage, got {other:?}"),
        }
    }

    #[test]
    fn test_non_exhaustive_missing_variant() {
        let option_type = make_option_type();
        let patterns = vec![Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        }];

        match check_exhaustive(&patterns, &option_type) {
            Coverage::Missing(missing) => {
                assert_eq!(missing.len(), 2);
                let names: Vec<&str> = missing
                    .iter()
                    .filter_map(|p| match p {
                        Pattern::Variant { name, .. } => Some(name.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    names.contains(&"Some") && names.contains(&"None"),
                    "Expected missing coverage for both `Some` and `None`"
                );
            }
            _ => panic!("Expected Missing coverage"),
        }
    }

    #[test]
    fn test_exhaustive_with_wildcard() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant {
                name: "Some".to_string(),
                fields: None,
            },
            Pattern::Wildcard,
        ];

        assert_eq!(check_exhaustive(&patterns, &option_type), Coverage::Covered);
    }

    #[test]
    fn test_exhaustive_with_variable() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant {
                name: "Some".to_string(),
                fields: None,
            },
            Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
        ];

        assert_eq!(check_exhaustive(&patterns, &option_type), Coverage::Covered);
    }

    #[test]
    fn test_empty_pattern_list() {
        let option_type = make_option_type();
        let patterns: Vec<Pattern> = vec![];

        match check_exhaustive(&patterns, &option_type) {
            Coverage::Missing(missing) => {
                assert_eq!(missing.len(), 2);
                // Should be missing both Some and None
            }
            _ => panic!("Expected Missing coverage for empty pattern list"),
        }
    }

    #[test]
    fn test_pattern_matrix_creation() {
        let patterns = vec![
            Pattern::Variant {
                name: "Some".to_string(),
                fields: None,
            },
            Pattern::Wildcard,
        ];

        let matrix = PatternMatrix::new(&patterns);
        assert_eq!(matrix.row_count(), 2);
    }

    #[test]
    fn test_pattern_to_cell_variant() {
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        };

        match pattern_to_cell(&pattern) {
            PatternCell::Constructor(name, fields) => {
                assert_eq!(name, "Some");
                assert!(fields.is_none());
            }
            _ => panic!("Expected Constructor cell"),
        }
    }

    #[test]
    fn test_pattern_to_cell_wildcard() {
        let pattern = Pattern::Wildcard;
        assert!(matches!(pattern_to_cell(&pattern), PatternCell::Wildcard));
    }

    #[test]
    fn test_pattern_to_cell_variable() {
        let pattern = Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        };
        assert!(matches!(pattern_to_cell(&pattern), PatternCell::Wildcard));
    }
}
