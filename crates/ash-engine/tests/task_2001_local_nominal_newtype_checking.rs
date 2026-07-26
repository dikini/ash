//! TASK-2001 RED coverage for local nominal-newtype program checking.
//!
//! These assertions deliberately exercise the ordinary `Engine::parse` /
//! `Engine::check` path, rather than declaration-registration metadata. They
//! do not claim runtime erasure, imports, generics, recursion, or handler
//! behavior. Local tuple-pattern checking is limited to the nominal wrapper's
//! declared constructor and representation type.

use ash_engine::Engine;

fn check(source: &str) -> Result<ash_engine::Entry, ash_engine::EngineError> {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(source)?;
    engine.check(&mut entry)?;
    Ok(entry)
}

#[test]
fn task_2001_local_newtype_constructor_checks_to_its_nominal_result_type() {
    let entry = check(
        r"
        newtype OrderId = OrderId(Int);
        fn main() -> OrderId { OrderId(7) }
        ",
    )
    .expect("a local newtype constructor must check as its distinct nominal type");

    assert!(
        entry.core_callable_types.contains_key("main"),
        "the normally checked program must retain main's Core type"
    );
}

#[test]
fn task_2001_local_newtype_tuple_pattern_binds_its_representation_type() {
    let entry = check(
        r"
        newtype OrderId = OrderId(Int);
        fn main() -> Int {
            let OrderId(value) = OrderId(7);
            value + 1
        }
        ",
    )
    .expect("an irrefutable local newtype pattern must bind its Int representation");

    assert!(
        entry.core_callable_types.contains_key("main"),
        "the normally checked program must retain main's Core type after using the pattern binding"
    );
}

#[test]
fn task_2001_local_newtype_pattern_rejects_a_different_nominal_constructor() {
    let error = check(
        r"
        newtype OrderId = OrderId(Int);
        newtype CustomerId = CustomerId(Int);
        fn main() -> Int {
            let CustomerId(value) = OrderId(7);
            value
        }
        ",
    )
    .expect_err("a local newtype pattern must not accept a distinct nominal constructor");

    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("CustomerId") && diagnostic.contains("OrderId"),
        "wrong-constructor diagnostics must retain both nominal identities: {error}"
    );
}

#[test]
fn task_2001_local_newtype_pattern_rejects_wrong_tuple_arity() {
    let error = check(
        r"
        newtype OrderId = OrderId(Int);
        fn main() -> Int {
            let OrderId(first, second) = OrderId(7);
            first + second
        }
        ",
    )
    .expect_err("a local newtype pattern must accept exactly one representation binding");

    assert!(
        error
            .to_string()
            .contains("expects 1 positional items, got 2"),
        "tuple-arity diagnostics must identify the one-field newtype wrapper: {error}"
    );
}

#[test]
fn task_2001_local_newtype_constructor_rejects_a_wrong_representation_payload() {
    let error = check(
        r#"
        newtype OrderId = OrderId(Int);
        fn main() -> OrderId { OrderId("not-an-int") }
        "#,
    )
    .expect_err("a newtype constructor must validate its representation payload");

    assert!(
        error
            .to_string()
            .contains("newtype constructor 'OrderId' expects Int but received String"),
        "wrong-payload diagnostic must identify the nominal constructor and representation: {error}"
    );
}

#[test]
fn task_2001_newtype_and_representation_do_not_coerce_in_either_direction() {
    let wrapped_where_int_is_required = check(
        r"
        newtype OrderId = OrderId(Int);
        fn require_int(value: Int) -> Int { value }
        fn main() -> Int { require_int(OrderId(7)) }
        ",
    )
    .expect_err("a nominal newtype value must not coerce to its representation");
    assert!(
        wrapped_where_int_is_required
            .to_string()
            .contains("expected Int but found OrderId"),
        "newtype-to-representation rejection must retain both types: {wrapped_where_int_is_required}"
    );

    let representation_where_wrapped_is_required = check(
        r"
        newtype OrderId = OrderId(Int);
        fn require_order_id(value: OrderId) -> OrderId { value }
        fn main() -> OrderId { require_order_id(7) }
        ",
    )
    .expect_err("a representation value must not coerce to its nominal newtype");
    assert!(
        representation_where_wrapped_is_required
            .to_string()
            .contains("expected OrderId but found Int"),
        "representation-to-newtype rejection must retain both types: {representation_where_wrapped_is_required}"
    );
}

#[test]
fn task_2001_two_local_wrappers_of_the_same_representation_remain_distinct() {
    let error = check(
        r"
        newtype OrderId = OrderId(Int);
        newtype CustomerId = CustomerId(Int);
        fn require_customer(value: CustomerId) -> CustomerId { value }
        fn main() -> CustomerId { require_customer(OrderId(7)) }
        ",
    )
    .expect_err("two wrappers with Int representations must remain nominally distinct");

    assert!(
        error
            .to_string()
            .contains("expected CustomerId but found OrderId"),
        "cross-newtype rejection must retain both nominal identities: {error}"
    );
}

#[test]
fn task_2001_bodyless_representation_cannot_back_a_newtype() {
    // Ordinary `type Token;` is not canonical surface syntax: type definitions
    // have a body. `builtin type Token;` is the current parser-supported opaque
    // bodyless declaration and is the relevant inhabitation boundary here.
    let engine = Engine::new().build().expect("engine builds");
    assert!(
        engine
            .parse("type Token;\nnewtype Wrap = Wrap(Token);\nfn main() -> Wrap { Wrap(Token) }")
            .is_err(),
        "the parser must continue to reject a bodyless ordinary type declaration"
    );

    let error = check(
        r"
        builtin type Token;
        newtype Wrap = Wrap(Token);
        fn main() -> Wrap { Wrap(Token) }
        ",
    )
    .expect_err("a bodyless opaque representation must be rejected for a newtype");

    assert!(
        error
            .to_string()
            .contains("newtype representation 'Token' is not inhabited"),
        "inhabitation rejection must name the representation: {error}"
    );
}

#[test]
fn task_2016_direct_recursive_local_newtype_is_rejected_during_normal_checking() {
    let error = check(
        r"
        newtype Loop = Loop(Loop);
        fn main() -> Loop { Loop(Loop) }
        ",
    )
    .expect_err("a direct recursive local newtype must be rejected before body checking");

    assert!(
        error
            .to_string()
            .contains("recursive newtype representation is not supported"),
        "recursive-newtype diagnostic must deterministically identify the unsupported boundary: {error}"
    );
}

#[test]
fn task_2016_rejects_a_newtype_that_collides_with_an_ordinary_local_type() {
    let error = check(
        r"
        type Token = Token(Int);
        newtype Token = Token(Int);
        fn main() -> Token { Token(7) }
        ",
    )
    .expect_err(
        "a newtype and ordinary local type with the same name and constructor must not overwrite each other",
    );

    assert!(
        error
            .to_string()
            .contains("conflicting local type declaration 'Token'"),
        "collision diagnostics must name the conflicting local spelling deterministically: {error}"
    );
}

#[test]
fn task_2016_newtype_cannot_reuse_an_ordinary_local_type_and_constructor_name() {
    let error = check(
        r"
        type Collision = Collision(Int);
        newtype Collision = Collision(Int);
        fn main() -> Collision { Collision(7) }
        ",
    )
    .expect_err("a local newtype must not overwrite an ordinary local type or its constructor");

    assert!(
        error
            .to_string()
            .contains("newtype 'Collision' conflicts with existing local type or constructor"),
        "collision diagnostic must reject the colliding nominal name and constructor: {error}"
    );
}

#[test]
fn task_2016_ordinary_local_type_cannot_reuse_a_newtype_name_and_constructor() {
    let error = check(
        r"
        newtype Collision = Collision(Int);
        type Collision = Collision(Int);
        fn main() -> Collision { Collision(7) }
        ",
    )
    .expect_err("an ordinary local type must not overwrite a local newtype or its constructor");

    assert!(
        error
            .to_string()
            .contains("local type 'Collision' conflicts with existing newtype or constructor"),
        "collision diagnostic must reject an ordinary declaration that reuses a newtype name: {error}"
    );
}

#[test]
fn task_2016_newtype_cannot_shadow_a_primitive_type_name() {
    let error = check(
        r#"
        newtype Int = Wrapped(String);
        fn main() -> Int { Wrapped("seven") }
        "#,
    )
    .expect_err("a local newtype must not shadow the primitive Int type");

    assert!(
        error
            .to_string()
            .contains("newtype 'Int' conflicts with existing primitive or prelude type"),
        "primitive-name collision must produce a deterministic diagnostic: {error}"
    );
}
