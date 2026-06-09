//! Shared kind vocabulary for Ash type expressions.
//!
//! Kind notation:
//! - `*`           - proper type (Int, String, `List<Int>`)
//! - `Prop`        - proposition kind (proof-relevant propositions)
//! - `* -> *`      - type constructor (List, Option)
//! - `* -> * -> *` - binary type constructor (Result, Pair)

use std::fmt;

/// A kind classifies types and type constructors.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    /// The kind of types: *
    Type,
    /// The kind of propositions: Prop
    Prop,
    /// Function kind: K1 -> K2
    Arrow(Box<Kind>, Box<Kind>),
}

impl Kind {
    /// Create a kind arrow: k1 -> k2.
    #[must_use]
    pub fn arrow(k1: impl Into<Box<Kind>>, k2: impl Into<Box<Kind>>) -> Self {
        Kind::Arrow(k1.into(), k2.into())
    }

    /// Create a kind for an n-ary type constructor.
    #[must_use]
    pub fn n_ary(n: usize) -> Self {
        (0..n).fold(Kind::Type, |acc, _| Kind::arrow(Kind::Type, acc))
    }

    /// Check if this is a proper type kind (*).
    #[must_use]
    pub fn is_type(&self) -> bool {
        matches!(self, Kind::Type)
    }

    /// Get the arity of this kind (number of type arguments).
    #[must_use]
    pub fn arity(&self) -> usize {
        match self {
            Kind::Type | Kind::Prop => 0,
            Kind::Arrow(_, rest) => 1 + rest.arity(),
        }
    }

    /// Check if this is an atomic kind that can be displayed without parentheses.
    #[must_use]
    fn is_atom(&self) -> bool {
        matches!(self, Kind::Type | Kind::Prop)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Type => write!(f, "*"),
            Kind::Prop => write!(f, "Prop"),
            Kind::Arrow(k1, k2) => {
                if k1.is_atom() {
                    write!(f, "{} -> {}", k1, k2)
                } else {
                    write!(f, "({}) -> {}", k1, k2)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_type_is_arity_zero() {
        assert_eq!(Kind::Type.arity(), 0);
        assert!(Kind::Type.is_type());
    }

    #[test]
    fn kind_prop_is_arity_zero_but_not_type() {
        assert_eq!(Kind::Prop.arity(), 0);
        assert!(!Kind::Prop.is_type());
        assert_ne!(Kind::Prop, Kind::Type);
    }

    #[test]
    fn kind_n_ary() {
        assert_eq!(Kind::n_ary(0), Kind::Type);
        assert_eq!(Kind::n_ary(1).arity(), 1);
        assert_eq!(Kind::n_ary(2).arity(), 2);
    }

    #[test]
    fn kind_display() {
        assert_eq!(Kind::Type.to_string(), "*");
        assert_eq!(Kind::Prop.to_string(), "Prop");
        assert_eq!(Kind::n_ary(1).to_string(), "* -> *");
        assert_eq!(Kind::n_ary(2).to_string(), "* -> * -> *");
        assert_eq!(Kind::arrow(Kind::Prop, Kind::Type).to_string(), "Prop -> *");
    }
}
