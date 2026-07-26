//! TASK-2001 RED coverage for non-generic nominal-newtype singleton patterns.
//!
//! These are source-file `Engine::parse_file` / `Engine::check` contracts for
//! the existing type-checking boundary only. They intentionally exclude generic
//! newtypes, multi-hop re-exports, proof patterns, Core/CPS lowering, and all
//! runtime representation or execution claims.

use ash_engine::Engine;
use std::path::Path;

fn write_file(path: &Path, source: &str) {
    std::fs::write(path, source).expect("write Ash source fixture");
}

fn check_entry(path: &Path) -> Result<(), ash_engine::EngineError> {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse_file(path)?;
    engine.check(&mut entry)
}

fn public_newtype_fixture(caller_source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("provider.ash"),
        "pub newtype OrderId = OrderId(Int);\n",
    );
    write_file(&dir.path().join("caller.ash"), caller_source);
    dir
}

#[test]
fn task_2001_local_newtype_singleton_match_binds_its_int_representation() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("caller.ash"),
        "newtype OrderId = OrderId(Int);\nfn main() -> Int { match OrderId(7) { OrderId(x) => x + 1 } }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "a local singleton-newtype match arm must bind x as the declared Int representation",
    );
}

#[test]
fn task_2001_direct_public_import_newtype_singleton_match_binds_provider_int() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { match OrderId(7) { OrderId(x) => x + 1 } }\n",
    );

    check_entry(&dir.path().join("caller.ash"))
        .expect("a direct public imported singleton-newtype match arm must bind x as provider Int");
}

#[test]
fn task_2001_one_hop_reexport_newtype_singleton_match_preserves_exact_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("inner.ash"),
        "pub newtype OrderId = OrderId(Int);\n",
    );
    write_file(&dir.path().join("outer.ash"), "pub use inner::{OrderId};\n");
    write_file(
        &dir.path().join("caller.ash"),
        "use outer::{OrderId}\nfn main() -> Int { match OrderId(7) { OrderId(x) => x + 1 } }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "a one-hop public re-export must retain its provider TypeDeclId for singleton matching",
    );
}

#[test]
fn task_2001_newtype_singleton_match_rejects_wrong_constructor_and_tuple_arity() {
    let wrong_constructor = public_newtype_fixture(
        "use provider::{OrderId}\nnewtype CustomerId = CustomerId(Int);\nfn main() -> Int { match OrderId(7) { CustomerId(x) => x } }\n",
    );
    let wrong_constructor_error = check_entry(&wrong_constructor.path().join("caller.ash"))
        .expect_err("a singleton-newtype match must reject a different nominal constructor");
    let wrong_constructor_diagnostic = wrong_constructor_error.to_string();
    assert!(
        wrong_constructor_diagnostic.contains("CustomerId")
            && wrong_constructor_diagnostic.contains("OrderId"),
        "wrong-constructor match diagnostics must retain both nominal identities: {wrong_constructor_error}"
    );

    let wrong_arity = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { match OrderId(7) { OrderId(first, second) => first + second } }\n",
    );
    let wrong_arity_error = check_entry(&wrong_arity.path().join("caller.ash"))
        .expect_err("a singleton-newtype match must accept exactly one representation binder");
    assert!(
        wrong_arity_error
            .to_string()
            .contains("expects 1 positional items, got 2"),
        "tuple-arity match diagnostics must identify the singleton newtype contract: {wrong_arity_error}"
    );
}

#[test]
fn task_2001_newtype_singleton_empty_match_reports_deterministic_missing_witness() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { match OrderId(7) { } }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash"))
        .expect_err("an empty singleton-newtype match must report its missing constructor witness");
    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("non-exhaustive match on type 'OrderId'")
            && diagnostic.contains("OrderId(_0)"),
        "singleton-newtype exhaustiveness must report the exact stable constructor witness: {error}"
    );
}

#[test]
fn task_2001_newtype_singleton_if_let_binds_int_and_remains_accepted_with_warning() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { if let OrderId(x) = OrderId(7) then x + 1 else 0 }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "an irrefutable singleton-newtype if-let must bind x as Int and stay accepted despite its unreachable-else warning",
    );
}
