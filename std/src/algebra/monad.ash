pub interface Monad<M : * -> *> {
    unit(Int) -> M<Int>
    bind(M<Int>, Int -> M<Int>) -> M<Int>
}

pub impl Monad<Option> {
    unit(value) = option::pure(value)
    bind(value, f) = option::and_then(value, f)
}

pub impl <E : *> Monad<Result<_, E>> {
    unit(value) = result::pure(value)
    bind(value, f) = result::and_then(value, f)
}
