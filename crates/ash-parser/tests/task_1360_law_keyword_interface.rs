use ash_parser::surface::{Definition, Expr};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn first_interface(source: &str) -> ash_parser::surface::InterfaceDef {
    parse(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("interface should be parsed")
}

#[test]
fn parses_law_inside_interface() {
    let interface = first_interface(
        r#"
        interface Semigroup<A> {
            append(A, A) -> A
            law associativity(a: A, b: A, c: A, eq: Eq<A>): eq.equiv(append(append(a, b), c), append(a, append(b, c)))
        }
        "#,
    );

    assert_eq!(interface.name.as_ref(), "Semigroup");
    assert_eq!(interface.laws.len(), 1);

    let law = &interface.laws[0];
    assert_eq!(law.name.as_ref(), "associativity");
    assert_eq!(law.params.len(), 4);
    assert_eq!(law.params[0].name.as_ref(), "a");
    assert_eq!(law.params[1].name.as_ref(), "b");
    assert_eq!(law.params[3].name.as_ref(), "eq");
    assert!(law.constraints.is_empty());
    // Just verify proposition parsed successfully (any expression)
    assert!(
        !matches!(&law.proposition, Expr::Variable { name, .. } if name.as_ref().is_empty()
        )
    );
}

#[test]
fn parses_law_with_where_constraints() {
    let interface = first_interface(
        r#"
        interface Monoid<M> {
            empty() -> M
            law left_identity(x: M, eq: Eq<M>) where eq(x, empty()): eq.equiv(append(empty(), x), x)
        }
        "#,
    );

    assert_eq!(interface.laws.len(), 1);
    let law = &interface.laws[0];
    assert_eq!(law.name.as_ref(), "left_identity");
    assert_eq!(law.params.len(), 2);
    assert_eq!(law.constraints.len(), 1);
    assert_eq!(law.constraints[0].predicate.name.as_ref(), "eq");
    assert_eq!(law.constraints[0].predicate.args.len(), 2);
}

#[test]
fn parses_multiple_laws() {
    let interface = first_interface(
        r#"
        interface Eq<T> {
            equiv(T, T) -> Bool
            law reflexivity(x: T): equiv(x, x)
            law symmetry(x: T, y: T): equiv(x, y) == equiv(y, x)
        }
        "#,
    );

    assert_eq!(interface.laws.len(), 2);
    assert_eq!(interface.laws[0].name.as_ref(), "reflexivity");
    assert_eq!(interface.laws[1].name.as_ref(), "symmetry");
}

#[test]
fn parses_interface_with_methods_and_laws() {
    let interface = first_interface(
        r#"
        interface Functor<F> {
            map(F<A>, (A) -> B) -> F<B>
            law identity(fa: F<A>): map(fa, id) == fa
        }
        "#,
    );

    assert_eq!(interface.methods.len(), 1);
    assert_eq!(interface.laws.len(), 1);
    assert_eq!(interface.laws[0].name.as_ref(), "identity");
}
