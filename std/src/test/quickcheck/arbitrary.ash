-- Default QuickCheck evidence for a type.
use test::quickcheck::strategy::{Strategy};

pub interface Arbitrary<T> {
    arbitrary() -> Strategy<T>
}
