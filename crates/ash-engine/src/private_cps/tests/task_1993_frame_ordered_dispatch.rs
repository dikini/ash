//! TASK-1993 integration tests: `eval_raise` must consume only the selected frame.

use super::super::eval_term;
use ash_core::cps::*;

fn operation(name: &str) -> EffectOp {
    EffectOp {
        item: EffectItem {
            namespace: "task-1993".to_owned(),
            name: name.to_owned(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_owned()],
        result_type: "String".to_owned(),
    }
}

fn exit_body(body: Term) -> Term {
    Term::LetCont {
        name: "exit".to_owned(),
        param: "value".to_owned(),
        cont_body: Box::new(Term::Return {
            value: Value::Atom(Atom::Var("value".to_owned())),
        }),
        body: Box::new(body),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn clause(op: EffectOp, result: &str) -> HandlerClause {
    HandlerClause {
        op,
        params: vec!["argument".to_owned()],
        resume: "resume".to_owned(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_owned()),
            arg: Atom::String(result.to_owned()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    }
}

fn provider(result: &str) -> Value {
    Value::Lam {
        params: vec!["argument".to_owned()],
        cont: "resume".to_owned(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_owned()),
            arg: Atom::String(result.to_owned()),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        rec_binding: None,
        row: EffectRow::default(),
    }
}

fn raise(op: EffectOp) -> Term {
    exit_body(Term::Raise {
        op,
        args: vec![Atom::String("payload".to_owned())],
        resume: ContRef::Label("exit".to_owned()),
        row: EffectRow::default(),
    })
}

#[test]
fn eval_raise_consumes_only_the_inner_handler_not_an_unbound_outer_provider() {
    let target = operation("target");
    let mut chain = HandlerChain::new();
    chain.push(HandlerFrame::Provider {
        op: target.clone(),
        handler: "outer_provider_must_not_be_resolved".to_owned(),
    });
    chain.push(HandlerFrame::Shallow {
        clause: clause(target.clone(), "inner-handler"),
    });

    assert_eq!(
        eval_term(&raise(target), &Env::new(), &chain),
        Ok(Atom::String("inner-handler".to_owned())),
        "the unselected outer provider must not be looked up or invoked"
    );
}

#[test]
fn eval_raise_consumes_only_the_inner_provider_not_the_outer_handler() {
    let target = operation("target");
    let mut chain = HandlerChain::new();
    chain.push(HandlerFrame::Shallow {
        clause: clause(target.clone(), "outer-handler"),
    });
    chain.push(HandlerFrame::Provider {
        op: target.clone(),
        handler: "inner_provider".to_owned(),
    });
    let env = Env::new().with_binding("inner_provider".to_owned(), provider("inner-provider"));

    assert_eq!(
        eval_term(&raise(target), &env, &chain),
        Ok(Atom::String("inner-provider".to_owned())),
        "handler and provider frames use one shared innermost-first ordering"
    );
}
