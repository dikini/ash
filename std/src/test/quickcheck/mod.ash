-- QuickCheck-style property testing substrate.
--
-- Canonical APIs live in submodules. Root exports are alpha convenience aliases;
-- reference material should prefer `test::quickcheck::<submodule>` paths.

pub mod context;
pub mod strategy;
pub mod arbitrary;
pub mod int;
pub mod bool;
pub mod string;
pub mod list;
pub mod combinator;
pub mod prelude;

pub use context::{
    GenContext,
    seed,
    size,
    split,
    variant,
    indexed,
    resize,
    choose_int,
    choose_bool,
};
pub use strategy::{Strategy, no_shrink};
pub use arbitrary::{Arbitrary};

pub use int::{
    ints,
    small as small_ints,
    positive as positive_ints,
    nonzero as nonzero_ints,
    positive,
    nonzero,
};
pub use bool::{bools};
pub use string::{strings, identifiers};
pub use list::{list_of, nonempty_ints as nonempty_int_lists, sorted_ints as sorted_int_lists, nonempty_ints, sorted_ints};
pub use combinator::{
    Weighted,
    weighted,
    map,
    map_with_shrink,
    map2,
    with_shrink,
    constant,
    one_of,
    one_of_weighted,
    append_shrink,
    prepend_shrink,
    RecursiveConfig,
    recursive,
    recursive_with,
    recursive_config,
    default_recursive_config,
};
