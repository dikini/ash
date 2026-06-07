pub interface Applicative<F : * -> *> {
    pure(Int) -> F<Int>
    apply(F<Int -> Int>, F<Int>) -> F<Int>
}

pub fn pure_option(value: Int) -> Option<Int> {
    option::pure(value)
}

pub fn apply_option(functions: Option<Int -> Int>, value: Option<Int>) -> Option<Int> {
    option::apply(functions, value)
}

pub fn pure_result<E>(value: Int) -> Result<Int, E> {
    result::pure(value)
}

pub fn apply_result<E>(functions: Result<Int -> Int, E>, value: Result<Int, E>) -> Result<Int, E> {
    result::apply(functions, value)
}
