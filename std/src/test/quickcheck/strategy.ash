-- QuickCheck v1 strategy value.
--
-- `gen` samples one value for one context. `shrink` returns an ordered finite
-- list of smaller candidates for the same semantic domain.

pub use test::quickcheck::context::{GenContext};

pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,
    shrink: (T) -> List<T>,
};

pub builtin fn no_shrink<T>(value: T) -> List<T>;
