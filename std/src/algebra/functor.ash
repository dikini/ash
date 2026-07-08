use algebra::eq::{Eq}

pub interface Functor<F : * -> *> {
    map(F<A>, (A) -> B) -> F<B>

    law identity(value: F<A>, eq: Eq<F<A>>):
        eq.equiv(map(value, |x| -> x), value)

    law composition(value: F<A>, f: (A) -> B, g: (B) -> C, eq: Eq<F<C>>):
        eq.equiv(map(map(value, f), g), map(value, |x| -> g(f(x))))
}
