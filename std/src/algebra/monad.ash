use algebra::applicative::{Applicative}
use algebra::eq::{Eq}

pub interface Monad<M : * -> *> where M: Applicative {
    unit(A) -> M<A>
    bind(M<A>, (A) -> M<B>) -> M<B>

    law left_identity(a: A, f: (A) -> M<B>, eq: Eq<M<B>>):
        eq.equiv(bind(unit(a), f), f(a))

    law right_identity(m: M<A>, eq: Eq<M<A>>):
        eq.equiv(bind(m, |x| -> unit(x)), m)

    law associativity(m: M<A>, f: (A) -> M<B>, g: (B) -> M<C>, eq: Eq<M<C>>):
        eq.equiv(bind(bind(m, f), g), bind(m, |x| -> bind(f(x), g)))
}
