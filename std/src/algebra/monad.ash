use algebra::applicative::{Applicative}

pub interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, Int -> M<Int>) -> M<Int>
}

pub fn unit_option(value: Int) -> Option<Int> {
    option::pure(value)
}

pub fn bind_option(value: Option<Int>, f: (Int) -> Option<Int>) -> Option<Int> {
    option::and_then(value, f)
}

pub fn unit_result<E>(value: Int) -> Result<Int, E> {
    result::pure(value)
}

pub fn bind_result<E>(value: Result<Int, E>, f: (Int) -> Result<Int, E>) -> Result<Int, E> {
    result::and_then(value, f)
}
