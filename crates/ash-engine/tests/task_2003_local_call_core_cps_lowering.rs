//! TASK-2003 RED contract for a checked local callable crossing Core/CPS.
//!
//! The current bounded `PureAnf` bridge rejects calls.  This test fixes the
//! next required local-call shape before production lowering is implemented:
//! the local function is a CPS lambda and the entry tail calls it through the
//! answer continuation.

use ash_core::{
    Expr, Value,
    cps::{Atom, ContRef, Term, Value as CpsValue},
};
use ash_engine::Engine;

const LOCAL_CALL_SOURCE: &str = "fn helper() -> Int { 7 }\nfn main() -> Int { helper() }";
const LOCAL_CALL_HELPER_RETURN_SOURCE: &str =
    "fn helper() -> Int { do { return 7; } }\nfn main() -> Int { helper() }";

#[test]
fn checked_local_call_lowers_to_a_cps_lambda_and_tail_call() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(LOCAL_CALL_SOURCE)
        .expect("local-call source parses");
    engine
        .check(&mut entry)
        .expect("local-call source typechecks before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked local call must lower to a CPS lambda and tail call");

    let Term::LetVal {
        name,
        value:
            CpsValue::Lam {
                params,
                cont,
                body: helper_body,
                ..
            },
        body: entry_body,
    } = term
    else {
        panic!("local callable lowering must bind a CPS lambda before the entry tail call");
    };
    assert_eq!(name, "helper");
    assert!(params.is_empty(), "helper has no source parameters");
    assert!(
        matches!(
            *helper_body,
            Term::Jump {
                cont: ContRef::Var(ref returned_to),
                arg: Atom::Int(7),
                ..
            } if returned_to == &cont
        ),
        "the helper result must jump to its explicit CPS continuation"
    );
    assert!(
        matches!(
            *entry_body,
            Term::Call {
                func: Atom::Var(ref callee),
                ref args,
                cont: ContRef::Label(ref answer),
                ..
            } if callee == "helper" && args.is_empty() && answer == "__answer"
        ),
        "main must tail-call the local CPS lambda through the answer continuation"
    );
}

#[test]
fn explicit_source_return_in_a_local_helper_jumps_to_the_caller_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(LOCAL_CALL_HELPER_RETURN_SOURCE)
        .expect("local helper return source parses");
    engine
        .check(&mut entry)
        .expect("local helper return source typechecks before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("the explicit helper return must lower through checked Core/CPS");

    let Term::LetVal {
        name,
        value:
            CpsValue::Lam {
                params,
                cont,
                body: helper_body,
                ..
            },
        body: entry_body,
    } = term
    else {
        panic!("the explicit helper return must retain the checked local-call CPS shape");
    };
    assert_eq!(name, "helper");
    assert!(params.is_empty(), "helper has no source parameters");
    assert!(
        matches!(
            *helper_body,
            Term::Jump {
                cont: ContRef::Var(ref returned_to),
                arg: Atom::Int(7),
                ..
            } if returned_to == &cont
        ),
        "an explicit source return must jump to the helper's caller continuation, never Term::Return"
    );
    assert!(
        matches!(
            *entry_body,
            Term::Call {
                func: Atom::Var(ref callee),
                ref args,
                cont: ContRef::Label(ref answer),
                ..
            } if callee == "helper" && args.is_empty() && answer == "__answer"
        ),
        "the local caller must supply the explicit __answer continuation"
    );
}

#[tokio::test]
async fn checked_local_call_executes_through_the_production_core_cps_path() {
    let engine = Engine::new().build().expect("engine builds");

    let value = engine
        .run(LOCAL_CALL_SOURCE)
        .await
        .expect("checked local call must execute through Core/CPS admission");

    assert_eq!(value, Value::Int(7));
}

#[test]
fn mutated_legacy_entry_core_cannot_retarget_the_sealed_local_call_route() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(LOCAL_CALL_SOURCE)
        .expect("local-call source parses");
    entry.core = Expr::Literal(Value::Int(99));

    let error = engine
        .admit_entry_to_checked_cps(&mut entry)
        .expect_err("a mutated public legacy Core must not mint local-call admission");
    assert!(
        error
            .to_string()
            .contains("canonical parsed entry provenance"),
        "local-call admission must reject the mutated parsed Core before lowering: {error}"
    );
}

#[test]
fn checked_local_call_rejects_a_file_entry_with_a_type_only_import() {
    let directory = tempfile::tempdir().expect("temporary source directory");
    let provider = directory.path().join("provider.ash");
    let caller = directory.path().join("caller.ash");
    std::fs::write(&provider, "pub type ImportedMarker = ImportedMarker;\n")
        .expect("write imported type module");
    std::fs::write(
        &caller,
        "use provider::{ImportedMarker}\n\nfn helper() -> Int { 7 }\nfn main() -> Int { helper() }\n",
    )
    .expect("write importing local-call entry");

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse_file(&caller)
        .expect("type-only imported local-call source parses");
    engine
        .check(&mut entry)
        .expect("unused imported type does not prevent ordinary typechecking");

    let error = engine
        .admit_entry_to_checked_cps(&mut entry)
        .expect_err("the sealed local-call route must reject retained imported state");
    assert!(
        error
            .to_string()
            .contains("does not admit imported source state"),
        "type-only imports must be rejected before local-call lowering: {error}"
    );
}
