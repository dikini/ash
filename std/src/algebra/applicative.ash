pub interface Applicative<F : * -> *> {
    pure(Int) -> F<Int>
    apply(F<Int -> Int>, F<Int>) -> F<Int>
}

pub impl Applicative<Option> {
    pure(value) = option::pure(value)
    apply(functions, value) = option::apply(functions, value)
}

pub impl <E : *> Applicative<Result<_, E>> {
    pure(value) = result::pure(value)
    apply(functions, value) = result::apply(functions, value)
}
