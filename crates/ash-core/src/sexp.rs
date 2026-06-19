//! CPS IR S-expression serialization via serde-lexpr
//!
//! Provides a human-readable S-expression format for CPS IR terms and values.
//! Uses serde-lexpr for serialization/deserialization.
//!
//! Format examples:
//!   (letval x (int 42) (jump (label exit) (var x)))
//!   (letprim y add ((int 1) (int 2)) (jump (label exit) (var y)))
//!   (lam (x) k (jump (var k) (var x)) (row))

use crate::cps::*;
use serde_lexpr;
use std::io;
use std::path::Path;

/// Error type for S-expression serialization/deserialization
#[derive(Debug, Clone, PartialEq)]
pub enum SexpError {
    Io(String),
    Serde(String),
}

impl std::fmt::Display for SexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexpError::Io(s) => write!(f, "io error: {}", s),
            SexpError::Serde(s) => write!(f, "serde error: {}", s),
        }
    }
}

impl std::error::Error for SexpError {}

impl From<io::Error> for SexpError {
    fn from(e: io::Error) -> Self {
        SexpError::Io(e.to_string())
    }
}

impl From<serde_lexpr::Error> for SexpError {
    fn from(e: serde_lexpr::Error) -> Self {
        SexpError::Serde(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// High-level API
// ---------------------------------------------------------------------------

/// Serialize a CPS IR term to an S-expression string
pub fn term_to_string(t: &Term) -> Result<String, SexpError> {
    let s = serde_lexpr::to_string(t)?;
    Ok(s)
}

/// Parse a CPS IR term from an S-expression string
pub fn string_to_term(s: &str) -> Result<Term, SexpError> {
    let t: Term = serde_lexpr::from_str(s)?;
    Ok(t)
}

/// Serialize a CPS IR value to an S-expression string
pub fn value_to_string(v: &Value) -> Result<String, SexpError> {
    let s = serde_lexpr::to_string(v)?;
    Ok(s)
}

/// Parse a CPS IR value from an S-expression string
pub fn string_to_value(s: &str) -> Result<Value, SexpError> {
    let v: Value = serde_lexpr::from_str(s)?;
    Ok(v)
}

/// Write a CPS IR term to a file in S-expression format
pub fn write_term_to_file(t: &Term, path: &Path) -> Result<(), SexpError> {
    let s = term_to_string(t)?;
    std::fs::write(path, s)?;
    Ok(())
}

/// Read a CPS IR term from a file in S-expression format
pub fn read_term_from_file(path: &Path) -> Result<Term, SexpError> {
    let s = std::fs::read_to_string(path)?;
    string_to_term(&s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_atom_int() {
        let atom = Atom::Int(42);
        let s = serde_lexpr::to_string(&atom).unwrap();
        let parsed: Atom = serde_lexpr::from_str(&s).unwrap();
        assert_eq!(atom, parsed);
    }

    #[test]
    fn test_roundtrip_atom_var() {
        let atom = Atom::Var("x".to_string());
        let s = serde_lexpr::to_string(&atom).unwrap();
        let parsed: Atom = serde_lexpr::from_str(&s).unwrap();
        assert_eq!(atom, parsed);
    }

    #[test]
    fn test_roundtrip_term_letval_jump() {
        let term = Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(42)),
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("x".to_string()),
                row: EffectRow::default(),
            }),
        };
        let s = term_to_string(&term).unwrap();
        let parsed = string_to_term(&s).unwrap();
        assert_eq!(term, parsed);
    }

    #[test]
    fn test_roundtrip_term_letprim() {
        let term = Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Add,
            args: vec![Atom::Int(1), Atom::Int(2)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        };
        let s = term_to_string(&term).unwrap();
        let parsed = string_to_term(&s).unwrap();
        assert_eq!(term, parsed);
    }

    #[test]
    fn test_roundtrip_term_if() {
        let term = Term::If {
            cond: Atom::Bool(true),
            then_branch: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Int(1),
                row: EffectRow::default(),
            }),
            else_branch: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Int(0),
                row: EffectRow::default(),
            }),
            row: EffectRow::default(),
        };
        let s = term_to_string(&term).unwrap();
        let parsed = string_to_term(&s).unwrap();
        assert_eq!(term, parsed);
    }

    #[test]
    fn test_roundtrip_term_call() {
        let id_lam = Value::Lam {
            params: vec!["x".to_string()],
            cont: "k".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("k".to_string()),
                arg: Atom::Var("x".to_string()),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        };
        let term = Term::LetVal {
            name: "id".to_string(),
            value: id_lam,
            body: Box::new(Term::Call {
                func: Atom::Var("id".to_string()),
                args: vec![Atom::Int(42)],
                cont: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
        };
        let s = term_to_string(&term).unwrap();
        let parsed = string_to_term(&s).unwrap();
        assert_eq!(term, parsed);
    }

    #[test]
    fn test_roundtrip_term_trap() {
        let term = Term::Trap {
            reason: TrapReason::Custom("unreachable".to_string()),
        };
        let s = term_to_string(&term).unwrap();
        let parsed = string_to_term(&s).unwrap();
        assert_eq!(term, parsed);
    }

    #[test]
    fn test_roundtrip_value_lam() {
        let lam = Value::Lam {
            params: vec!["x".to_string()],
            cont: "k".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("k".to_string()),
                arg: Atom::Var("x".to_string()),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        };
        let s = value_to_string(&lam).unwrap();
        let parsed = string_to_value(&s).unwrap();
        assert_eq!(lam, parsed);
    }

    #[test]
    fn test_file_roundtrip() {
        let term = Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Add,
            args: vec![Atom::Int(1), Atom::Int(2)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        };

        let path = std::path::PathBuf::from("/tmp/test_cps_roundtrip.cps");
        write_term_to_file(&term, &path).unwrap();
        let parsed = read_term_from_file(&path).unwrap();
        assert_eq!(term, parsed);

        // Clean up
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_serialize_format_readable() {
        let term = Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(42)),
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("x".to_string()),
                row: EffectRow::default(),
            }),
        };
        let s = term_to_string(&term).unwrap();
        // Should be readable S-expression format, not JSON
        assert!(s.starts_with('('));
        assert!(!s.contains('{'));
    }
}
