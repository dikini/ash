//! TASK-1858/TASK-1859 handler/provider operational semantics tests.

use ash_core::cps::*;
use ash_interp::cps::{CpsError, eval_term};

fn op(name: &str) -> EffectOp {
    EffectOp {
        item: EffectItem {
            namespace: "operation".to_string(),
            name: name.to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "String".to_string(),
    }
}

fn exit_body(body: Term) -> Term {
    Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Return {
            value: Value::Atom(Atom::Var("v".to_string())),
        }),
        body: Box::new(body),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn handler_clause(op: EffectOp, result: &str, resume: &str) -> HandlerClause {
    HandlerClause {
        op,
        params: vec!["path".to_string()],
        resume: resume.to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var(resume.to_string()),
            arg: Atom::String(result.to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    }
}

fn provider_handler(result: &str) -> Value {
    Value::Lam {
        params: vec!["path".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::String(result.to_string()),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        rec_binding: None,
        row: EffectRow::default(),
    }
}

#[test]
fn task_1859_handle_dispatch_resumes_with_handler_result() {
    let read = op("PosixFs::read");
    let term = exit_body(Term::Handle {
        clause: handler_clause(read.clone(), "handled", "resume"),
        body: Box::new(Term::Raise {
            op: read,
            args: vec![Atom::String("config.toml".to_string())],
            resume: ContRef::Label("exit".to_string()),
            row: EffectRow::default(),
        }),
        cont: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    });

    assert_eq!(
        eval_term(&term, &Env::new(), &HandlerChain::new()),
        Ok(Atom::String("handled".to_string()))
    );
}

#[test]
fn task_1859_provider_frame_dispatches_raise() {
    let read = op("PosixFs::read");
    let env = Env::new().with_binding("provider_read".to_string(), provider_handler("provided"));
    let mut chain = HandlerChain::new();
    chain.push(HandlerFrame::Provider {
        op: read.clone(),
        handler: "provider_read".to_string(),
    });
    let term = exit_body(Term::Raise {
        op: read,
        args: vec![Atom::String("config.toml".to_string())],
        resume: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    });

    assert_eq!(
        eval_term(&term, &env, &chain),
        Ok(Atom::String("provided".to_string()))
    );
}

#[test]
fn task_1858_inner_provider_shadows_outer_handler_for_same_operation() {
    let read = op("PosixFs::read");
    let env = Env::new().with_binding("provider_read".to_string(), provider_handler("provided"));
    let mut chain = HandlerChain::new();
    chain.push(HandlerFrame::Provider {
        op: read.clone(),
        handler: "provider_read".to_string(),
    });

    let term = exit_body(Term::Handle {
        clause: handler_clause(read.clone(), "handled", "resume"),
        body: Box::new(Term::Raise {
            op: read,
            args: vec![Atom::String("config.toml".to_string())],
            resume: ContRef::Label("exit".to_string()),
            row: EffectRow::default(),
        }),
        cont: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    });

    assert_eq!(
        eval_term(&term, &env, &chain),
        Ok(Atom::String("handled".to_string())),
        "the handle form installs an inner handler that shadows an outer provider"
    );
}

#[test]
fn task_1858_inner_provider_frame_shadows_outer_handler_frame_for_same_operation() {
    let read = op("PosixFs::read");
    let env = Env::new().with_binding("provider_read".to_string(), provider_handler("provided"));
    let mut chain = HandlerChain::new();
    chain.push(HandlerFrame::Shallow {
        clause: handler_clause(read.clone(), "handled", "resume"),
    });
    chain.push(HandlerFrame::Provider {
        op: read.clone(),
        handler: "provider_read".to_string(),
    });
    let term = exit_body(Term::Raise {
        op: read,
        args: vec![Atom::String("config.toml".to_string())],
        resume: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    });

    assert_eq!(
        eval_term(&term, &env, &chain),
        Ok(Atom::String("provided".to_string())),
        "frame-stack order must not skip an inner provider to use an outer handler"
    );
}

#[test]
fn task_1860_raise_without_handler_or_provider_is_unhandled_effect() {
    let read = op("PosixFs::read");
    let term = exit_body(Term::Raise {
        op: read.clone(),
        args: vec![Atom::String("config.toml".to_string())],
        resume: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    });

    assert!(matches!(
        eval_term(&term, &Env::new(), &HandlerChain::new()),
        Err(CpsError::UnhandledEffect(inner)) if inner == read
    ));
}
