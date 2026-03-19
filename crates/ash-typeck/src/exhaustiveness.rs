//! Exhaustiveness checking for pattern matches
//!
//! Provides exhaustiveness analysis to ensure all pattern match cases are covered.
//! Uses a pattern matrix approach for analyzing coverage.

use crate::check_pattern::TypeDef;
use ash_core::ast::Pattern;

/// Coverage result for exhaustiveness checking
#[derive(Debug, Clone, PartialEq)]
pub enum Coverage {
    /// All cases are covered
    Covered,
    /// Some cases are missing
    Missing(Vec<Pattern>),
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
    Constructor(String, Vec<PatternCell>),
}

impl PatternMatrix {
    /// Create a new pattern matrix from a list of patterns
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
        Pattern::Wildcard | Pattern::Variable(_) => PatternCell::Wildcard,
        Pattern::Variant { name, fields } => {
            let field_cells = fields
                .as_ref()
                .map(|f| f.iter().map(|(_, p)| pattern_to_cell(p)).collect())
                .unwrap_or_default();
            PatternCell::Constructor(name.clone(), field_cells)
        }
        Pattern::Tuple(patterns) => PatternCell::Constructor(
            "tuple".to_string(),
            patterns.iter().map(pattern_to_cell).collect(),
        ),
        Pattern::Literal(_) => PatternCell::Constructor("literal".to_string(), vec![]),
        Pattern::Record(fields) => PatternCell::Constructor(
            "record".to_string(),
            fields.iter().map(|(_, p)| pattern_to_cell(p)).collect(),
        ),
        Pattern::List(patterns, rest) => {
            let mut cells: Vec<PatternCell> = patterns.iter().map(pattern_to_cell).collect();
            if rest.is_some() {
                cells.push(PatternCell::Wildcard);
            }
            PatternCell::Constructor("list".to_string(), cells)
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

/// Find uncovered patterns for a type
fn find_uncovered(matrix: &PatternMatrix, type_def: &TypeDef) -> Option<Vec<Pattern>> {
    // Get all covered variant names from the matrix
    let covered: Vec<String> = matrix
        .rows
        .iter()
        .filter_map(|row| match row.first() {
            Some(PatternCell::Constructor(name, _)) => Some(name.clone()),
            _ => None,
        })
        .collect();

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
    for variant in &type_def.variants {
        if !covered.contains(&variant.name) {
            missing.push(Pattern::Variant {
                name: variant.name.clone(),
                fields: None,
            });
        }
    }

    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check_pattern::VariantDef;
    use crate::types::Type;

    /// Create a test Option type with Some and None variants
    fn make_option_type() -> TypeDef {
        TypeDef {
            name: "Option".to_string(),
            variants: vec![
                VariantDef {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), Type::Var(crate::types::TypeVar(0)))],
                },
                VariantDef {
                    name: "None".to_string(),
                    fields: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_exhaustive_full_coverage() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant {
                name: "Some".to_string(),
                fields: None,
            },
            Pattern::Variant {
                name: "None".to_string(),
                fields: None,
            },
        ];

        assert_eq!(check_exhaustive(&patterns, &option_type), Coverage::Covered);
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
                assert_eq!(missing.len(), 1);
                // Should be missing None
                match &missing[0] {
                    Pattern::Variant { name, .. } => {
                        assert_eq!(name, "None");
                    }
                    _ => panic!("Expected Variant pattern for None"),
                }
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
            Pattern::Variable("x".to_string()),
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
                assert!(fields.is_empty());
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
        let pattern = Pattern::Variable("x".to_string());
        assert!(matches!(pattern_to_cell(&pattern), PatternCell::Wildcard));
    }
}
