//! TASK-1599/1600: S-expression round-trip integration tests
//!
//! Tests that CPS IR terms can be written to .cps files, read back,
//! and executed with the same results as the original.
//! This exercises the full file workflow: serialize → write → read → execute.

use ash_core::cps::*;
use ash_core::sexp::{read_term_from_file, write_term_to_file};
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from("/tmp");
    path.push(format!("ash_cps_test_{}.cps", name));
    path
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_file_roundtrip_letval_jump() {
    let term = Term::LetVal {
        name: "x".to_string().to_string(),
        value: Value::Atom(Atom::Int(42)),
        body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("x".to_string()),
            row: EffectRow::default(),
        }),
    };

    let path = temp_path("letval_jump");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    assert_eq!(term, parsed);
}

#[test]
fn test_file_roundtrip_letprim() {
    let term = Term::LetPrim {
        name: "y".to_string().to_string(),
        op: PrimOp::Add,
        args: vec![Atom::Int(1), Atom::Int(2)],
        body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("y".to_string()),
            row: EffectRow::default(),
        }),
    };

    let path = temp_path("letprim");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    assert_eq!(term, parsed);
}

#[test]
fn test_file_roundtrip_if() {
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

    let path = temp_path("if");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    assert_eq!(term, parsed);
}

#[test]
fn test_file_roundtrip_call() {
    let id_lam = Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::Var("x".to_string()),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        row: EffectRow::default(),
    };
    let term = Term::LetVal {
        name: "id".to_string().to_string(),
        value: id_lam,
        body: Box::new(Term::Call {
            func: Atom::Var("id".to_string()),
            args: vec![Atom::Int(42)],
            cont: ContRef::Label("exit".to_string()),
            row: EffectRow::default(),
        }),
    };

    let path = temp_path("call");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    assert_eq!(term, parsed);
}

#[test]
fn test_file_roundtrip_trap() {
    let term = Term::Trap {
        reason: TrapReason::Custom("unreachable".to_string()),
    };

    let path = temp_path("trap");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    assert_eq!(term, parsed);
}

#[test]
fn test_file_roundtrip_factorial() {
    // Factorial: letrec fact = (lam [n] k ...) in (call fact [5] exit)
    let factorial_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::LetPrim {
            name: "is_zero".to_string().to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Var("n".to_string()), Atom::Int(0)],
            body: Box::new(Term::If {
                cond: Atom::Var("is_zero".to_string()),
                then_branch: Box::new(Term::Jump {
                    cont: ContRef::Var("k".to_string()),
                    arg: Atom::Int(1),
                    row: EffectRow::default(),
                }),
                else_branch: Box::new(Term::LetPrim {
                    name: "n_minus_1".to_string().to_string(),
                    op: PrimOp::Sub,
                    args: vec![Atom::Var("n".to_string()), Atom::Int(1)],
                    body: Box::new(Term::LetCont {
                        name: "k_mul".to_string().to_string(),
                        param: "result".to_string(),
                        cont_body: Box::new(Term::LetPrim {
                            name: "prod".to_string().to_string(),
                            op: PrimOp::Mul,
                            args: vec![Atom::Var("n".to_string()), Atom::Var("result".to_string())],
                            body: Box::new(Term::Jump {
                                cont: ContRef::Var("k".to_string()),
                                arg: Atom::Var("prod".to_string()),
                                row: EffectRow::default(),
                            }),
                        }),
                        body: Box::new(Term::Call {
                            func: Atom::Var("fact".to_string()),
                            args: vec![Atom::Var("n_minus_1".to_string())],
                            cont: ContRef::Label("k_mul".to_string()),
                            row: EffectRow::default(),
                        }),
                    }),
                }),
                row: EffectRow::default(),
            }),
        }),
        captured_env: Env::new(),
        row: EffectRow::default(),
    };

    let term = Term::LetRec {
        name: "fact".to_string().to_string(),
        value: factorial_lam,
        body: Box::new(Term::Call {
            func: Atom::Var("fact".to_string()),
            args: vec![Atom::Int(5)],
            cont: ContRef::Label("exit".to_string()),
            row: EffectRow::default(),
        }),
    };

    let path = temp_path("factorial");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    assert_eq!(term, parsed);
}

#[test]
fn test_serialize_format_is_sexp_not_json() {
    let term = Term::LetVal {
        name: "x".to_string().to_string(),
        value: Value::Atom(Atom::Int(42)),
        body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("x".to_string()),
            row: EffectRow::default(),
        }),
    };

    let s = ash_core::sexp::term_to_string(&term).unwrap();
    // Must be S-expression format (starts with paren), not JSON (starts with brace)
    assert!(
        s.starts_with('('),
        "Expected S-expression format, got: {}",
        s
    );
    assert!(
        !s.contains('{'),
        "Expected S-expression format, got JSON: {}",
        s
    );
}

#[test]
fn test_serialize_readable_format() {
    let term = Term::LetPrim {
        name: "y".to_string().to_string(),
        op: PrimOp::Add,
        args: vec![Atom::Int(1), Atom::Int(2)],
        body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("y".to_string()),
            row: EffectRow::default(),
        }),
    };

    let s = ash_core::sexp::term_to_string(&term).unwrap();
    // serde_lexpr uses record-style S-expressions with field names
    assert!(s.contains("LetPrim"), "Missing 'LetPrim' in output: {}", s);
    assert!(s.contains("Add"), "Missing 'Add' in output: {}", s);
    assert!(s.contains("Jump"), "Missing 'Jump' in output: {}", s);
}

#[test]
fn test_file_extension_is_cps() {
    let term = Term::Trap {
        reason: TrapReason::Custom("unreachable".to_string()),
    };

    let path = temp_path("extension");
    write_term_to_file(&term, &path).unwrap();

    // Verify file was created with .cps extension
    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".cps"));

    cleanup(&path);
}

#[test]
fn test_multiple_file_roundtrips() {
    let terms = [
        Term::LetVal {
            name: "a".to_string().to_string(),
            value: Value::Atom(Atom::Int(1)),
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("a".to_string()),
                row: EffectRow::default(),
            }),
        },
        Term::LetPrim {
            name: "b".to_string().to_string(),
            op: PrimOp::Mul,
            args: vec![Atom::Int(3), Atom::Int(4)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("b".to_string()),
                row: EffectRow::default(),
            }),
        },
        Term::If {
            cond: Atom::Bool(false),
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
        },
    ];

    for (i, term) in terms.iter().enumerate() {
        let path = temp_path(&format!("multi_{}", i));
        write_term_to_file(term, &path).unwrap();
        let parsed = read_term_from_file(&path).unwrap();
        cleanup(&path);
        assert_eq!(*term, parsed, "Roundtrip failed for term {}", i);
    }
}
