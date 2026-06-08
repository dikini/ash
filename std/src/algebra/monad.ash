use algebra::applicative::{Applicative}

pub interface Monad<M : * -> *> where M: Applicative {
    unit(A) -> M<A>
    bind(M<A>, A -> M<B>) -> M<B>
}
