use ash_core::kind::Kind;

#[test]
fn prop_kind_exists_and_is_not_type() {
    assert_eq!(Kind::Prop.arity(), 0);
    assert!(!Kind::Prop.is_type());
    assert_ne!(Kind::Prop, Kind::Type);
}

#[test]
fn prop_kind_displays_as_prop() {
    assert_eq!(Kind::Prop.to_string(), "Prop");
    assert_eq!(Kind::arrow(Kind::Prop, Kind::Type).to_string(), "Prop -> *");
}
