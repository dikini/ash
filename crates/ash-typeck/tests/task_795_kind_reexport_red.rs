use ash_core::kind::Kind as CoreKind;
use ash_typeck::Kind;

#[test]
fn ash_typeck_kind_is_the_shared_core_kind_contract() {
    let from_typeck: Kind = Kind::n_ary(1);
    let from_core: CoreKind = CoreKind::n_ary(1);

    assert_eq!(from_typeck, from_core);
}
