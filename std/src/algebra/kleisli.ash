use algebra::monad::{unit_option, bind_option, unit_result, bind_result}

pub fn id_option(value: Int) -> Option<Int> {
    unit_option(value)
}

pub fn compose_option(
    value: Int,
    f: (Int) -> Option<Int>,
    g: (Int) -> Option<Int>
) -> Option<Int> {
    bind_option(f(value), g)
}

pub fn id_result<E>(value: Int) -> Result<Int, E> {
    unit_result(value)
}

pub fn compose_result<E>(
    value: Int,
    f: (Int) -> Result<Int, E>,
    g: (Int) -> Result<Int, E>
) -> Result<Int, E> {
    bind_result(f(value), g)
}
