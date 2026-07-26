//! TASK-2001 RED coverage for public nominal newtypes imported through ordinary files.
//!
//! These cases use the normal file resolver and `Engine::parse_file` / `Engine::check`
//! boundary. They deliberately distinguish a public nominal wrapper from a transparent
//! ordinary alias; they do not assert runtime representation, generic newtypes, or imported
//! newtype-pattern support.

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
fn task_2001_named_imported_public_newtype_constructor_retains_its_nominal_type() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn require(value: OrderId) -> OrderId { value }\nfn main() -> OrderId { require(OrderId(7)) }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "a public named-imported newtype constructor must produce its imported nominal type",
    );
}

#[test]
fn task_2001_named_imported_newtype_constructor_rejects_wrong_representation_payload() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> OrderId { OrderId(\"not-an-int\") }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash"))
        .expect_err("an imported newtype constructor must validate its representation payload");
    assert!(
        error
            .to_string()
            .contains("newtype constructor 'OrderId' expects Int but received String"),
        "wrong imported-newtype payload diagnostic must identify the constructor and representation: {error}"
    );
}

#[test]
fn task_2001_imported_newtype_and_representation_do_not_coerce_in_either_direction() {
    let wrapped_as_int = public_newtype_fixture(
        "use provider::{OrderId}\nfn require_int(value: Int) -> Int { value }\nfn main() -> Int { require_int(OrderId(7)) }\n",
    );
    let wrapped_as_int_error = check_entry(&wrapped_as_int.path().join("caller.ash"))
        .expect_err("an imported newtype must not coerce to its Int representation");
    assert!(
        wrapped_as_int_error
            .to_string()
            .contains("expected Int but found OrderId"),
        "newtype-to-representation diagnostic must retain both types: {wrapped_as_int_error}"
    );

    let int_as_wrapped = public_newtype_fixture(
        "use provider::{OrderId}\nfn require_order(value: OrderId) -> OrderId { value }\nfn main() -> OrderId { require_order(7) }\n",
    );
    let int_as_wrapped_error = check_entry(&int_as_wrapped.path().join("caller.ash"))
        .expect_err("an Int must not coerce to an imported newtype");
    assert!(
        int_as_wrapped_error
            .to_string()
            .contains("expected OrderId but found Int"),
        "representation-to-newtype diagnostic must retain both types: {int_as_wrapped_error}"
    );
}

#[test]
fn task_2001_imported_newtype_tuple_pattern_remains_out_of_scope() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { let imported_value = OrderId(7); let OrderId(value) = imported_value; value + 1 }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash")).expect_err(
        "an imported newtype tuple pattern must remain unavailable on the normal checking path",
    );
    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("canonicalization of OrderId blocked")
            && diagnostic.contains("UnknownType"),
        "the imported pattern boundary must fail deterministically during type checking: {error}"
    );
}

#[test]
fn task_2001_private_newtype_cannot_be_named_or_constructed_through_a_named_import() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("provider.ash"),
        "newtype PrivateOrderId = PrivateOrderId(Int);\n",
    );
    write_file(
        &dir.path().join("caller.ash"),
        "use provider::{PrivateOrderId}\nfn main() -> PrivateOrderId { PrivateOrderId(7) }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash")).expect_err(
        "a private newtype must not enter an importing module's type or value namespace",
    );
    assert!(
        error.to_string().contains("PrivateOrderId"),
        "private-import diagnostic must identify the inaccessible newtype: {error}"
    );
}

#[test]
fn task_2001_imported_ordinary_type_alias_remains_transparent() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("provider.ash"),
        "pub type Counter = Int;\n",
    );
    write_file(
        &dir.path().join("caller.ash"),
        "use provider::{Counter}\nfn require_int(value: Int) -> Int { value }\nfn main() -> Counter { require_int(7) }\n",
    );

    check_entry(&dir.path().join("caller.ash"))
        .expect("an imported ordinary alias must remain transparent to its Int representation");
}
