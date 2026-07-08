use algebra::functor::{Functor}
use algebra::eq::{Eq}

pub interface Applicative<F : * -> *> where F: Functor {
    pure(A) -> F<A>
    apply(F<(A) -> B>, F<A>) -> F<B>

    law identity(value: F<A>, eq: Eq<F<A>>):
        eq.equiv(apply(pure(|x| -> x), value), value)

    law homomorphism(x: A, f: (A) -> B, eq: Eq<F<B>>):
        eq.equiv(apply(pure(f), pure(x)), pure(f(x)))

    law interchange(u: F<(A) -> B>, y: A, eq: Eq<F<B>>):
        eq.equiv(apply(u, pure(y)), apply(pure(|f| -> f(y)), u))

    law composition(u: F<(B) -> C>, v: F<(A) -> B>, w: F<A>, eq: Eq<F<C>>):
        eq.equiv(
            apply(apply(apply(pure(|f| -> |g| -> |x| -> f(g(x))), u), v), w),
            apply(u, apply(v, w))
        )
}
