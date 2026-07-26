//! TASK-2001 RED coverage for public nominal newtypes imported through ordinary files.
//!
//! These cases use the normal file resolver and `Engine::parse_file` / `Engine::check`
//! boundary. They deliberately distinguish a public nominal wrapper from a transparent
//! ordinary alias; they do not assert runtime representation, generic newtypes, or broader
//! pattern matching support.

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
fn task_2001_imported_public_newtype_tuple_pattern_binds_provider_representation_type() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { let OrderId(value) = OrderId(7); value + 1 }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "a public named-imported newtype pattern must bind its provider representation type",
    );
}

#[test]
fn task_2001_imported_newtype_tuple_pattern_rejects_private_provider_import() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("provider.ash"),
        "newtype PrivateOrderId = PrivateOrderId(Int);\n",
    );
    write_file(
        &dir.path().join("caller.ash"),
        "use provider::{PrivateOrderId}\nfn main() -> Int { let PrivateOrderId(value) = PrivateOrderId(7); value }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash")).expect_err(
        "a private provider newtype pattern must not enter an importing module's namespaces",
    );
    assert!(
        error.to_string().contains("PrivateOrderId"),
        "private provider-pattern diagnostic must identify the inaccessible newtype: {error}"
    );
}

#[test]
fn task_2001_imported_newtype_tuple_pattern_rejects_different_local_nominal_constructor() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nnewtype CustomerId = CustomerId(Int);\nfn main() -> Int { let CustomerId(value) = OrderId(7); value }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash")).expect_err(
        "an imported provider newtype pattern must not accept a different local nominal constructor",
    );
    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("CustomerId") && diagnostic.contains("OrderId"),
        "wrong-constructor diagnostics must retain both nominal identities: {error}"
    );
}

#[test]
fn task_2001_imported_newtype_tuple_pattern_rejects_wrong_constructor_arity() {
    let dir = public_newtype_fixture(
        "use provider::{OrderId}\nfn main() -> Int { let OrderId(first, second) = OrderId(7); first + second }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash")).expect_err(
        "an imported provider newtype pattern must accept exactly one representation binding",
    );
    assert!(
        error
            .to_string()
            .contains("expects 1 positional items, got 2"),
        "tuple-arity diagnostics must identify the one-field imported newtype wrapper: {error}"
    );
}

#[test]
fn task_2001_reexported_newtype_tuple_pattern_preserves_provider_identity_and_representation() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("inner.ash"),
        "pub newtype OrderId = OrderId(Int);\n",
    );
    write_file(&dir.path().join("outer.ash"), "pub use inner::{OrderId};\n");
    write_file(
        &dir.path().join("caller.ash"),
        "use outer::{OrderId}\nfn main() -> Int { let OrderId(value) = OrderId(7); value }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "a public re-export must retain the inner provider TypeDeclId and bind its Int representation",
    );
}

#[test]
fn task_2001_two_hop_reexported_newtype_tuple_pattern_remains_closed_until_admitted() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("inner.ash"),
        "pub newtype OrderId = OrderId(Int);\n",
    );
    write_file(
        &dir.path().join("middle.ash"),
        "pub use inner::{OrderId};\n",
    );
    write_file(
        &dir.path().join("outer.ash"),
        "pub use middle::{OrderId};\n",
    );
    write_file(
        &dir.path().join("caller.ash"),
        "use outer::{OrderId}\nfn main() -> Int { let OrderId(value) = OrderId(7); value }\n",
    );

    let error = check_entry(&dir.path().join("caller.ash")).expect_err(
        "a two-hop public facade must not admit a nominal-newtype tuple pattern before that topology is proved",
    );
    assert!(
        error.to_string().contains("OrderId"),
        "the closed multi-hop pattern diagnostic must retain the visible nominal name: {error}"
    );
}

#[test]
fn task_2001_same_spelled_reexport_facade_preserves_matching_direct_import_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(
        &dir.path().join("inner.ash"),
        "pub newtype OrderId = OrderId(Int);\n",
    );
    write_file(&dir.path().join("outer.ash"), "pub use inner::{OrderId};\n");
    write_file(
        &dir.path().join("caller.ash"),
        "use inner::{OrderId}\nuse outer::{OrderId}\nfn main() -> Int { let OrderId(value) = OrderId(7); value }\n",
    );

    check_entry(&dir.path().join("caller.ash")).expect(
        "a same-spelled facade must remain admissible when it resolves to the direct import's exact TypeDeclId",
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
