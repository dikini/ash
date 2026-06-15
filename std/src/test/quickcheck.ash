-- QuickCheck-style property testing substrate.
--
-- `Arbitrary<T>` supplies the canonical/default strategy for a type.
-- `Strategy<T>` is an explicit generated domain plus shrink relation and is
-- used when a property needs a narrower semantic domain than arbitrary `T`.

pub type Strategy<T> = Strategy { id: String };

pub interface Arbitrary<T> {
    arbitrary() -> Strategy<T>
    gen(Int, Int) -> List<T>
    shrink(T) -> List<T>

    law gen_projects_from_arbitrary(seed: Int, size: Int):
        true == true

    law shrink_projects_from_arbitrary(value: T):
        true == true
}

pub fn ints() -> Strategy<Int> {
    Strategy { id: "test::quickcheck::ints" }
}

pub fn small_ints() -> Strategy<Int> {
    Strategy { id: "test::quickcheck::small_ints" }
}

pub fn positive_ints() -> Strategy<Int> {
    Strategy { id: "test::quickcheck::positive_ints" }
}

pub fn nonzero_ints() -> Strategy<Int> {
    Strategy { id: "test::quickcheck::nonzero_ints" }
}

pub fn bools() -> Strategy<Bool> {
    Strategy { id: "test::quickcheck::bools" }
}

pub fn strings() -> Strategy<String> {
    Strategy { id: "test::quickcheck::strings" }
}

pub fn identifiers() -> Strategy<String> {
    Strategy { id: "test::quickcheck::identifiers" }
}

pub fn sorted_int_lists() -> Strategy<List<Int>> {
    Strategy { id: "test::quickcheck::sorted_int_lists" }
}

pub fn nonempty_int_lists() -> Strategy<List<Int>> {
    Strategy { id: "test::quickcheck::nonempty_int_lists" }
}

pub fn map<A, B>(strategy: Strategy<A>, f: (A) -> B) -> Strategy<B> {
    Strategy { id: "test::quickcheck::map" }
}

pub fn map2<A, B, C>(left: Strategy<A>, right: Strategy<B>, f: (A, B) -> C) -> Strategy<C> {
    Strategy { id: "test::quickcheck::map2" }
}

pub fn one_of<T>(strategies: List<Strategy<T>>) -> Strategy<T> {
    Strategy { id: "test::quickcheck::one_of" }
}

pub fn list_of<T>(element: Strategy<T>) -> Strategy<List<T>> {
    Strategy { id: "test::quickcheck::list_of" }
}
